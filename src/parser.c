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

    struct ast_valof_expr* current_valof;

    struct ast_program* program;
};

enum operator_precedence {
    PREC_LOWEST,

    PREC_CALL,

    PREC_HIGHEST
};

static struct ast_generic_stmt* parse_statement(struct parser_context* ctx);
static struct ast_generic_expr* parse_expression(struct parser_context* ctx, enum operator_precedence precedence, enum token_kind end);

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

static struct ast_funccall_expr* parse_function_call(struct parser_context* ctx, struct ast_generic_expr* callee) {
    struct ast_funccall_expr* call = malloc(sizeof(struct ast_funccall_expr));
    ast_funccall_init(call, &ctx->cur_tok.loc, callee);
    parser_consume(ctx, TOKEN_LPAREN, "expect `(` for function call");

    while(ctx->cur_tok.kind != TOKEN_RPAREN) {
        ptr_list_add(&call->params, parse_expression(ctx, PREC_LOWEST, TOKEN_COMMA)); 

        if(ctx->cur_tok.kind != TOKEN_RPAREN)
            parser_consume(ctx, TOKEN_COMMA, "expect `,` between function call parameters");
    }

    parser_advance(ctx);

    return call;
}

static struct ast_generic_expr* parse_prefix_expression(struct parser_context* ctx) {
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
    case TOKEN_VALOF: {
        expr = malloc(sizeof(struct ast_valof_expr));
        ast_valof_init(AST_CAST_EXPR(expr, valof), &ctx->cur_tok.loc);
        parser_advance(ctx);

        struct ast_valof_expr* outer = ctx->current_valof;
        ctx->current_valof = AST_CAST_EXPR(expr, valof);

        AST_CAST_EXPR(expr, valof)->body = parse_statement(ctx);

        ctx->current_valof = outer;
    } break;
    case TOKEN_IDENT:
        expr = malloc(sizeof(struct ast_ident_expr));
        ast_ident_expr_init(AST_CAST_EXPR(expr, ident), &ctx->cur_tok.loc, ctx->cur_tok.val.string);
        parser_advance(ctx);
        break;
    default:
        print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected token, expect expression");
    }
    
    return expr;
}

static struct ast_generic_expr* parse_infix_expression(struct parser_context* ctx, struct ast_generic_expr* left) {
    struct ast_generic_expr* expr = NULL;

    switch(ctx->cur_tok.kind) {
    case TOKEN_LPAREN:
        expr = AST_AS_GENERIC_EXPR(parse_function_call(ctx, left));
        break;
    default:
        print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected token, expect infix operator");
    }

    return expr;
}

static enum operator_precedence get_operator_precedence(enum token_kind kind) {
    switch(kind) {
    case TOKEN_LPAREN:
        return PREC_CALL;
    default:
        return PREC_LOWEST;
    }
}

static struct ast_generic_expr* parse_expression(struct parser_context* ctx, enum operator_precedence precedence, enum token_kind end) {
    struct ast_generic_expr* expr = parse_prefix_expression(ctx);

    while(ctx->cur_tok.kind != end && precedence < get_operator_precedence(ctx->cur_tok.kind)) {
        expr = parse_infix_expression(ctx, expr);
    }

    return expr;
}

#define SKIP_SEMICOLON() if(ctx->cur_tok.kind == TOKEN_SEMICOLON) \
    parser_advance(ctx);

static struct ast_generic_stmt* parse_statement(struct parser_context* ctx) {
    struct ast_generic_stmt* stmt;
    
    switch(ctx->cur_tok.kind) {
    case TOKEN_LBRACE:
        stmt = malloc(sizeof(struct ast_block_stmt));
        ast_block_stmt_init(AST_CAST_STMT(stmt, block), &ctx->cur_tok.loc);

        parser_advance(ctx);

        while(ctx->cur_tok.kind != TOKEN_RBRACE)
            ptr_list_add(&AST_CAST_STMT(stmt, block)->stmts, parse_statement(ctx));
        
        parser_advance(ctx);
        break;
    case TOKEN_RESULTIS:
        stmt = malloc(sizeof(struct ast_resultis_stmt));
        ast_resultis_stmt_init(AST_CAST_STMT(stmt, resultis), &ctx->cur_tok.loc);
        
        parser_advance(ctx);

        AST_CAST_STMT(stmt, resultis)->expr = parse_expression(ctx, PREC_LOWEST, TOKEN_SEMICOLON);
        SKIP_SEMICOLON();

        if(!ctx->current_valof) {
            print_compiler_error(ctx->ctx, ERROR_DEFAULT, &stmt->loc, "encountered `resultis` statement outside of `valof` expression");
            break;
        }

        if(!ctx->current_valof->type)
            ctx->current_valof->type = AST_CAST_STMT(stmt, resultis)->expr->type;
        else {
            struct ast_typecast_expr* cast = malloc(sizeof(struct ast_typecast_expr));
            ast_typecast_init(cast, stmt->loc, ctx->current_valof->type, AST_CAST_STMT(stmt, resultis)->expr);
            AST_CAST_STMT(stmt, resultis)->expr = AST_AS_GENERIC_EXPR(cast);
        }
        break;
    default: {
            stmt = malloc(sizeof(struct ast_expr_stmt));

            struct ast_generic_expr* expr = parse_expression(ctx, PREC_LOWEST, TOKEN_SEMICOLON);
            ast_expr_stmt_init(AST_CAST_STMT(stmt, expr), &expr->loc, expr);
            SKIP_SEMICOLON();
        }
    }

    return stmt;
}

#undef SKIP_SEMICOLON

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

        struct ast_generic_expr* value = parse_expression(ctx, PREC_LOWEST, TOKEN_SEMICOLON);
        if(ast_generic_decl_type(decl) == TYPE_NOT_FOUND)
            ast_generic_decl_set_type(decl, value->type);
        
        if(ast_generic_decl_type(decl) != value->type) {
            struct ast_typecast_expr* cast = malloc(sizeof(struct ast_typecast_expr));
            ast_typecast_init(cast, value->loc, ast_generic_decl_type(decl), value);
            ast_generic_decl_set_expr(decl, AST_AS_GENERIC_EXPR(cast));
        }
        else
            ast_generic_decl_set_expr(decl, value);

        if(ctx->cur_tok.kind == TOKEN_SEMICOLON)
            parser_advance(ctx);

        ptr_list_add(&section->declarations, (const void*) decl);
    }

    parser_advance(ctx);
}

static struct ast_param* parse_function_param(struct parser_context* ctx) {
    struct ast_param* param = malloc(sizeof(struct ast_param));
    ast_param_init(param, &ctx->cur_tok.loc, ctx->cur_tok.val.string);
    
    parser_consume(ctx, TOKEN_IDENT, "expect identifier for function parameter");
    
    if(ctx->cur_tok.kind == TOKEN_OF) {
        parser_advance(ctx);
        param->type = parse_type(ctx);
    }

    if(ctx->cur_tok.kind == TOKEN_EQ) {
        parser_advance(ctx);
        param->default_value = parse_expression(ctx, PREC_LOWEST, TOKEN_COMMA);

        if(!param->type) 
            param->type = param->default_value->type;
        else if(param->type != param->default_value->type) {
            struct ast_typecast_expr* cast = malloc(sizeof(struct ast_typecast_expr));
            ast_typecast_init(cast, param->default_value->loc, param->type, param->default_value);
            param->default_value = AST_AS_GENERIC_EXPR(cast);
        }
    }

    if(!param->type && !param->default_value)
        print_compiler_error(ctx->ctx, ERROR_DEFAULT, &param->loc, "function parameter `%s` has neither an explicit type nor a default value", param->ident);

    return param;
}

static void parse_function_decl(struct parser_context* ctx, struct ast_section* section) {
    bool tailcall_recursive = ctx->cur_tok.kind == TOKEN_AND;
    parser_advance(ctx);

    struct ast_function_decl* decl = malloc(sizeof(struct ast_function_decl));
    ast_function_decl_init(decl, &ctx->cur_tok.loc, ctx->cur_tok.val.string, tailcall_recursive);

    parser_consume(ctx, TOKEN_IDENT, tailcall_recursive ? "expect identifier afgter `and`" : "expect identifier after `let`");

    // parse argument list
    if(ctx->cur_tok.kind == TOKEN_LPAREN) {
        parser_advance(ctx);

        while(ctx->cur_tok.kind != TOKEN_RPAREN) {
            struct ast_param* param = parse_function_param(ctx);
            if(decl->params->size != decl->required_params && !param->default_value)
                print_compiler_error(ctx->ctx, ERROR_DEFAULT, &param->loc, "function parameter `%s` without default value appears after paramers with default value", param->ident);
            
            ast_function_decl_add_param(decl, param);

            if(ctx->cur_tok.kind != TOKEN_RPAREN)
                parser_consume(ctx, TOKEN_COMMA, "expect `,` between function parameters");
        }

        parser_consume(ctx, TOKEN_RPAREN, "expect `)` after function parameters");
    }

    switch(ctx->cur_tok.kind) {
    case TOKEN_BE:
        parser_advance(ctx);
        ast_function_decl_set_stmt(decl, parse_statement(ctx));
        break;
    case TOKEN_EQ:
        parser_advance(ctx);
        ast_generic_decl_set_expr(AST_AS_GENERIC_DECL(decl), parse_expression(ctx, PREC_LOWEST, TOKEN_SEMICOLON));
        if(ctx->cur_tok.kind == TOKEN_SEMICOLON)
            parser_advance(ctx);
        break;
    default:
        print_compiler_error(ctx->ctx, ERROR_FATAL, &ctx->cur_tok.loc, "unexpected token, expect either `=` or `be` after `%s` declaration", tailcall_recursive ? "and" : "let");
        break;
    }

    ptr_list_add(&section->declarations, (const void*) decl);
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

