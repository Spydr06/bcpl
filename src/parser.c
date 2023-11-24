#include "parser.h"

#include "ast.h"
#include "context.h"
#include "token.h"

#include <assert.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>

struct parser_context {
    struct context* ctx;
    struct source_file* file;

    struct token cur_tok;
    struct token last_tok;

    struct ast_program* program;
};

inline static void parser_advance(struct parser_context* ctx) {
    ctx->last_tok = ctx->cur_tok;
    next_token(ctx->file, &ctx->cur_tok, &ctx->last_tok, &ctx->ctx->tags);

    if(ctx->cur_tok.kind == LEX_ERROR)
        print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "error parsing token: %s", ctx->cur_tok.val.string);
}

inline static void parser_consume(struct parser_context* ctx, enum token_kind expect, const char* error) {
    if(ctx->cur_tok.kind != expect) {
        if(ctx->cur_tok.kind == LEX_EOF)
            print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected end of file, %s", error);
        else
            print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected token, %s", error);
    }

    parser_advance(ctx);
}

static ast_type_index_t parse_type_ident(struct parser_context* ctx) {
    for(enum ast_type_kind kind = BUILTIN_PRIMITIVE_TYPE_START; kind < BUILTIN_PRIMITIVE_TYPE_END; kind++)
        if(strcmp(primitive_types[kind], ctx->cur_tok.val.string) == 0) {
            parser_advance(ctx);
            return ast_builtin_type(&ctx->ctx->ast, kind);
        }
    
    print_compiler_error(ctx->ctx, ERROR_DEFAULT, &ctx->cur_tok.loc, "undefined type `%s`, custom types are not supported yet", ctx->cur_tok.val.string);
    parser_advance(ctx);
    return TYPE_NOT_FOUND;
}

static ast_type_index_t parse_type(struct parser_context* ctx) {
    switch(ctx->cur_tok.kind) {
    case TOKEN_IDENT:
        return parse_type_ident(ctx);
    default:
        print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected token, expect data type");
        return TYPE_NOT_FOUND;
    }
}

static struct ast_generic_expr* parse_expression(struct parser_context* ctx) {
    struct ast_generic_expr* expr = NULL;
    switch(ctx->cur_tok.kind) {
    case TOKEN_INTEGER:
        expr = malloc(sizeof(struct ast_intlit_expr));
        ast_intlit_init(AST_CAST_EXPR(expr, intlit), &ctx->cur_tok.loc, ctx->cur_tok.val.integer);
        parser_advance(ctx);
        break;
    case TOKEN_FLOAT:
        expr = malloc(sizeof(struct ast_floatlit_expr));
        ast_floatlit_init(AST_CAST_EXPR(expr, floatlit), &ctx->cur_tok.loc, ctx->cur_tok.val.floatp);
        parser_advance(ctx);
        break;
    case TOKEN_TRUE:
        expr = malloc(sizeof(struct ast_generic_expr));
        ast_true_init(expr, &ctx->cur_tok.loc);
        parser_advance(ctx);
        break;
    case TOKEN_FALSE:
        expr = malloc(sizeof(struct ast_generic_expr));
        ast_false_init(expr, &ctx->cur_tok.loc);
        parser_advance(ctx);
        break;
    case TOKEN_CHAR:
        expr = malloc(sizeof(struct ast_charlit_expr));
        ast_charlit_init(AST_CAST_EXPR(expr, charlit), &ctx->cur_tok.loc, ctx->cur_tok.val.integer > CHAR_MAX, (wchar_t) ctx->cur_tok.val.integer);
        parser_advance(ctx);
        break;
    case TOKEN_STRING:
        expr = malloc(sizeof(struct ast_stringlit_expr));
        ast_stringlit_init(AST_CAST_EXPR(expr, stringlit), &ctx->cur_tok.loc, ctx->cur_tok.val.string);
        parser_advance(ctx);
        break;
    default:
        print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected token, expect expression");
    }

    return expr;
}

static void parse_require(struct parser_context* ctx, struct ast_section* section) {
    parser_consume(ctx, TOKEN_REQUIRE, "expect `require`");
    
    do {
        string_list_add(&section->required, ctx->cur_tok.val.string);
        parser_consume(ctx, TOKEN_IDENT, "expect identifier after `require`");
    } while(ctx->cur_tok.kind == TOKEN_COMMA AND_THEN(parser_advance(ctx)));
}

static void parse_global_decl(struct parser_context* ctx, struct ast_section* section) {
    enum ast_decl_kind decl_kind;

    const char* error_str;
    switch(ctx->cur_tok.kind) {
    case TOKEN_GLOBAL:
        error_str = "expect `{` after `global`";
        decl_kind = DECL_GLOBAL;
        break;
    case TOKEN_STATIC:
        error_str = "expect `{` after `static`";
        decl_kind = DECL_STATIC;
        break;
    case TOKEN_MANIFEST:
        error_str = "expect `{` after `manifest`";
        decl_kind = DECL_MANIFEST;
        break;
    default:
        print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpecterrd token, expect one of `global`, `static`, `manifest`");
    }

    parser_advance(ctx);
    parser_consume(ctx, TOKEN_LBRACE, error_str);

    while(ctx->cur_tok.kind != TOKEN_RBRACE) {
        struct ast_generic_decl* decl; 
        switch(decl_kind) {
        case DECL_GLOBAL:
            decl = malloc(sizeof(struct ast_global_decl));
            ast_global_decl_init(AST_CAST_DECL(decl, global), &ctx->cur_tok.loc, ctx->cur_tok.val.string);
            break;
        case DECL_MANIFEST:
            decl = malloc(sizeof(struct ast_manifest_decl));
            ast_manifest_decl_init(AST_CAST_DECL(decl, manifest), &ctx->cur_tok.loc, ctx->cur_tok.val.string);
            break;
        case DECL_STATIC:
            decl = malloc(sizeof(struct ast_static_decl));
            ast_static_decl_init(AST_CAST_DECL(decl, static), &ctx->cur_tok.loc, ctx->cur_tok.val.string);
            break;
        default:
            assert(false);
        }

        parser_consume(ctx, TOKEN_IDENT, "expect identifier");
    
        if(ctx->cur_tok.kind == TOKEN_OF) {
            parser_advance(ctx);
            ast_generic_decl_set_type(decl, parse_type(ctx)); 
        }

        parser_consume(ctx, TOKEN_EQ, "expect `=`");

        ast_generic_decl_set_expr(decl, parse_expression(ctx));
    }

    parser_advance(ctx);
}

static void parse_function_decl(struct parser_context* ctx, struct ast_section* section) {

}

static void parse_section(struct parser_context* ctx) {
    struct ast_section* section = malloc(sizeof(struct ast_section));
    ast_section_init(section, &ctx->cur_tok.loc);

    parser_consume(ctx, TOKEN_SECTION, "expect `section`");

    section->ident = ctx->cur_tok.val.string;
    parser_consume(ctx, TOKEN_IDENT, "expect identifier after `section`");

    ptr_list_add(&ctx->ctx->ast.sections, section);

    bool had_decls = false;
    while(1) {
        switch(ctx->cur_tok.kind) {
        case TOKEN_SECTION:
        case LEX_EOF:
            return;
        case TOKEN_REQUIRE:
            if(had_decls)
                print_compiler_error(ctx->ctx, ERROR_WARNING, &ctx->cur_tok.loc, "encountered `require` after declarations");
            parse_require(ctx, section);
            break;
        case TOKEN_MANIFEST:
        case TOKEN_GLOBAL:
        case TOKEN_STATIC:
            had_decls = true;
            parse_global_decl(ctx, section);
            break;
        case TOKEN_LET:
        case TOKEN_AND:
            had_decls = true;
            parse_function_decl(ctx, section);
            break;
        default:
            print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected token, expect declaration");
        }
    } 
}

void parse_file(struct context *ctx, struct source_file* file) {
    struct parser_context parser_ctx = {
        .ctx = ctx,
        .file = file,
        .cur_tok = (struct token){.kind=LEX_EOF},
    };

    parser_advance(&parser_ctx);
    while(parser_ctx.cur_tok.kind != LEX_EOF) {
        parse_section(&parser_ctx);
    }
}

