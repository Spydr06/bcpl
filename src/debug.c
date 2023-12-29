#include "debug.h"
#include "ast.h"
#include <ctype.h>
#include <wchar.h>
#include <wctype.h>

static const char* const token_kind_strs[] = {
    [1] = "IDENT",
    "INTEGER",
    "FLOAT",
    "STRING",
    "CHAR",
    "LPAREN",
    "RPAREN",
    "LBRACE",
    "RBRACE",
    "LBRACKET",
    "RBRACKET",
    "SEMICOLON",
    "COMMA",
    "DOT",
    "COLON",
    "ASSIGN",
    "COND",
    "QMARK",
    "EMARK",
    "AT",
    "PLUS",
    "MINUS",
    "STAR",
    "SLASH",
    "EQ",
    "NE",
    "GT",
    "GE",
    "LT",
    "LE",
    "NOT",
    "LOGAND",
    "LOGOR",
    "XOR",
    "LSHIFT",
    "RSHIFT",
    "TRUE",
    "FALSE",
    "LET",
    "AND",
    "VALOF",
    "RESULTIS",
    "RETURN",
    "FINISH",
    "SKIP",
    "REPEAT",
    "BREAK",
    "IF",
    "UNLESS",
    "WHILE",
    "FOR",
    "UNTIL",
    "SWITCHON",
    "MATCH",
    "EVERY",
    "CASE",
    "DEFAULT",
    "DO",
    "TO",
    "BY",
    "OF",
    "BE",
    "SECTION",
    "REQUIRE",
    "GLOBAL",
    "MANIFEST",
    "STATIC",
    "MOD",
    "ABS",

    "LEX_ERROR",
    [0] = "LEX_EOF"
};

void dbg_print_token(struct token* t) {
    printf("%s:%u:%zu->%u ", t->loc.file->path, t->loc.line, t->loc.offset, t->loc.width);
    switch(t->kind) {
        case TOKEN_INTEGER:
            printf("NUMBER %lu\n", t->val.integer);
            break;
        case TOKEN_FLOAT:
            printf("NAME %s\n", t->val.string);
            break;
        case TOKEN_STRING:
            printf("STRINGCONST %s\n", t->val.string);
            break;
        case TOKEN_IDENT:
            printf("IDENT %s\n", t->val.string);
            break;
        case LEX_ERROR:
            printf("LEX_ERROR %s\n", t->val.string);
            break;
        default:
            puts(token_kind_strs[t->kind]);
            break;
    } 
}

#define PRINT_INDENT_(_indent) do {                                         \
        for(size_t _i = 0; _i < (_indent) - 1; _i++) fputs("| ", stdout);   \
        fputc('|', stdout);                                                 \
    } while(0)

#define PRINT_INDENT(_indent) do { \
        fputs("\033[90m", stdout); \
        PRINT_INDENT_(_indent);    \
        fputs("\033[0m", stdout);  \
    } while(0)

#define PRINT_JCT(_indent, ...) do{                          \
        fputs("\033[90m", stdout);                      \
        PRINT_INDENT_(_indent); fputs("\\\n", stdout);  \
        PRINT_INDENT_(_indent);                         \
        printf("\033[1m + \033[35m" __VA_ARGS__);       \
        fputs("\033[0m", stdout);                       \
    } while(0)

#define PRINT_END(_indent) do {                         \
        fputs("\033[90m", stdout);                      \
        PRINT_INDENT_(_indent); fputs(" '\n", stdout);  \
        fputs("\033[0m", stdout);                       \
    } while(0)

void dbg_print_ast_expr(const struct ast_generic_expr* expr, uint32_t indent);

void dbg_print_ast_stmt(const struct ast_generic_stmt* stmt, uint32_t indent) {
    if(!stmt) {
        PRINT_JCT(indent, "(null)\n");
        PRINT_END(indent);
        return;
    }

    switch(stmt->kind) {
    case STMT_EXPR:
        PRINT_JCT(indent, "expression\n");
        dbg_print_ast_expr(AST_CAST_STMT(stmt, expr)->expr, indent + 1);
        break;
    case STMT_BLOCK:
        PRINT_JCT(indent, "block\n");
        for(size_t i = 0; i < AST_CAST_STMT(stmt, block)->stmts->size; i++)
            dbg_print_ast_stmt(AST_CAST_STMT(stmt, block)->stmts->data[i], indent + 1);
        break;
    case STMT_RESULTIS:
        PRINT_JCT(indent, "resultis\n");
        dbg_print_ast_expr(AST_CAST_STMT(stmt, resultis)->expr, indent + 1);
        break;
    default:
        PRINT_JCT(indent, "<unexpected %d>\n", stmt->kind);
    }

    PRINT_END(indent);
}

void dbg_print_ast_expr(const struct ast_generic_expr* expr, uint32_t indent) {
    if(!expr) {
        PRINT_JCT(indent, "(null)\n");
        PRINT_END(indent);
        return;
    }

    switch(expr->kind) {
    case EXPR_INTLIT:
        PRINT_JCT(indent, "intlit: %zu\n", AST_CAST_EXPR(expr, intlit)->value);
        break;
    case EXPR_FLOATLIT:
        PRINT_JCT(indent, "floatlit: %.15f\n", AST_CAST_EXPR(expr, floatlit)->value);
        break;
    case EXPR_CHARLIT:
        PRINT_JCT(indent, "charlit: ");
        if(AST_CAST_EXPR(expr, charlit)->unicode)
            if(iswprint(AST_CAST_EXPR(expr, charlit)->value))
                wprintf(L"'%c'\n", AST_CAST_EXPR(expr, charlit)->value);
            else
                wprintf(L"'*#h%04x'\n", AST_CAST_EXPR(expr, charlit)->value);
        else
            if(isprint(AST_CAST_EXPR(expr, charlit)->value))
                printf("'%c'\n", AST_CAST_EXPR(expr, charlit)->value);
            else
                printf("'*h%02x'\n", AST_CAST_EXPR(expr, charlit)->value);
        break;
    case EXPR_STRINGLIT:
        PRINT_JCT(indent, "stringlit: \"%s\"\n", AST_CAST_EXPR(expr, stringlit)->value);
        break;
    case EXPR_TRUE:
        PRINT_JCT(indent, "true\n");
        break;
    case EXPR_FALSE:
        PRINT_JCT(indent, "false\n");
        break;
    case EXPR_IDENT:
        PRINT_JCT(indent, "identifier: %s\n", AST_CAST_EXPR(expr, ident)->ident);
        break;
    case EXPR_TYPECAST:
        PRINT_JCT(indent, "typecast\n");
        dbg_print_ast_expr(AST_CAST_EXPR(expr, typecast)->expr, indent + 1);
        break;
    case EXPR_VALOF:
        PRINT_JCT(indent, "valof\n");
        dbg_print_ast_stmt(AST_CAST_EXPR(expr, valof)->body, indent + 1);
        break;
    case EXPR_FUNCCALL:
        PRINT_JCT(indent, "function call\n");
        dbg_print_ast_expr(AST_CAST_EXPR(expr, funccall)->callee, indent + 1);
        PRINT_JCT(indent + 1, "params:\n");
        for(size_t i = 0; i < AST_CAST_EXPR(expr, funccall)->params->size; i++)
            dbg_print_ast_expr(AST_CAST_EXPR(expr, funccall)->params->data[i], indent + 2);
        PRINT_END(indent + 1);
        break;
    default:
        PRINT_JCT(indent, "<unexpected %d>\n", expr->kind);
    }

    PRINT_INDENT(indent + 1);
    printf(" type: %3u\n", expr->type);

    PRINT_END(indent);
}

void dbg_print_ast_var_decl(const struct ast_generic_decl* decl, uint32_t indent) {
    switch(decl->kind) {
    case DECL_MANIFEST: 
        PRINT_JCT(indent, "manifest: %s\n", decl->ident);
        PRINT_INDENT(indent + 1);
        printf(" type: %3u\n", AST_CAST_DECL(decl, manifest)->type);
        dbg_print_ast_expr(AST_CAST_DECL(decl, manifest)->expr, indent + 1);
        break;
    case DECL_STATIC:
        PRINT_JCT(indent, "static: %s\n", decl->ident);
        PRINT_INDENT(indent + 1);
        printf(" type: %3u\n", AST_CAST_DECL(decl, static)->type);
        dbg_print_ast_expr(AST_CAST_DECL(decl, static)->expr, indent + 1);
        break;
    case DECL_GLOBAL:
        PRINT_JCT(indent, "global: %s\n", decl->ident);
        PRINT_INDENT(indent + 1);
        printf(" type: %3u\n", AST_CAST_DECL(decl, global)->type);
        PRINT_INDENT(indent + 1);
        printf(" public: %d\n", AST_CAST_DECL(decl, global)->is_public);
        dbg_print_ast_expr(AST_CAST_DECL(decl, static)->expr, indent + 1);
        break;
    default:
        PRINT_JCT(indent, "<unexpected>: ");
        break;
    }

    PRINT_END(indent);
}

void dbg_print_ast_param(const struct ast_param* param, uint32_t indent) {
    PRINT_JCT(indent, "param: %s\n", param->ident);

    PRINT_INDENT(indent + 1);
    printf(" type: %3u\n", param->type);

    if(param->default_value)
        dbg_print_ast_expr(param->default_value, indent + 1);
    else {
        PRINT_INDENT(indent + 1);
        printf(" default value: ---\n");
    }

    PRINT_END(indent);
}

void dbg_print_ast_function_decl(const struct ast_function_decl* func, uint32_t indent) {
    PRINT_JCT(indent, "function: %s\n", func->ident);

    PRINT_INDENT(indent + 1);
    printf(" num paramerters: %u\n", func->params->size);
    PRINT_INDENT(indent + 1);
    printf(" required paramenters: %u\n", func->required_params);

    PRINT_INDENT(indent + 1);
    printf(" return type: %3u\n", func->return_type);

    PRINT_INDENT(indent + 1);
    printf(" tailcall recursive: %d\n", func->tailcall_recursive);

    PRINT_JCT(indent + 1, "parameters\n");
    for(size_t i = 0; i < func->params->size; i++)
        dbg_print_ast_param(func->params->data[i], indent + 2); 
    PRINT_END(indent + 1);

    if(func->body_is_stmt) {
        PRINT_JCT(indent + 1, "body (statement)\n");
        dbg_print_ast_stmt(func->body.stmt, indent + 2);
    }
    else {
        PRINT_JCT(indent + 1, "body (expression)\n");
        dbg_print_ast_expr(func->body.expr, indent + 2);
    }
    PRINT_END(indent + 1);

    PRINT_END(indent);
}

void dbg_print_ast_section(const struct ast_section* section, uint32_t indent) {
    PRINT_JCT(indent, "section: %s\n", section->ident);

    PRINT_INDENT(indent + 1);
    printf(" requires:");
    for(size_t i = 0; i < section->required->size; i++)
        printf(" %s", section->required->strings[i]);
    putchar('\n');

    for(size_t i = 0; i < section->declarations->size; i++) {
        const struct ast_generic_decl* decl = section->declarations->data[i];
 
        if(decl->kind == DECL_FUNCTION)
            dbg_print_ast_function_decl(AST_CAST_DECL(decl, function), indent + 1);
        else
            dbg_print_ast_var_decl(decl, indent + 1);
    }

    PRINT_END(indent);
}

void dbg_print_ast_indexed_type(const struct ast_generic_type* type, ast_type_index_t index, uint32_t indent) {
    PRINT_JCT(indent, "type: %3u\n", index);

    PRINT_INDENT(indent + 1);
    if(type->kind <= BUILTIN_PRIMITIVE_TYPE_END)
        printf(" builtin: %s\n", primitive_types[type->kind]);
    else
        printf(" <complex>\n"); // TODO
    
    PRINT_INDENT(indent + 1);
    printf(" size: %u\n", type->size);

    PRINT_END(indent);
}

void dbg_print_ast_program(const struct ast_program* ast) {
    printf("+ ast_program\n");
    
    for(size_t i = 0; i < ast->types->size; i++)
        dbg_print_ast_indexed_type(ast->types->data[i], i + 1, 1);
    
    for(size_t i = 0; i < ast->sections->size; i++) 
        dbg_print_ast_section(ast->sections->data[i], 1);
}

