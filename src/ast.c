#include "ast.h"
#include "util.h"

#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

const char* const primitive_types[BUILTIN_PRIMITIVE_TYPE_END + 1] = {
    [TYPE_UINT8] = "UInt8",
    "UInt16",
    "UInt",
    "UInt64",
    
    "Int8",
    "Int16",
    "Int",
    "Int64",

    "Float",
    "Float64",

    "Bool",
    "Char",
    "Unit"
};

static const uint32_t primitive_type_sizes[BUILTIN_PRIMITIVE_TYPE_END + 1] = {
    [TYPE_UINT8] = 1,
    2,
    4,
    8,

    1,
    2,
    4,
    8,

    4,
    8,

    1,
    1,
    1
};

void ast_section_init(struct ast_section* section, const struct location* loc) {
    section->hdr.loc = *loc;
    section->ident = NULL;
    section->required = string_list_init();
    section->declarations = ptr_list_init();
}

void ast_program_init(struct ast_program* program) {
    memset(program, 0, sizeof(struct ast_program));

    program->sections = ptr_list_init();
    program->types = ptr_list_init();

    for(enum ast_type_kind i = BUILTIN_PRIMITIVE_TYPE_START; i <= BUILTIN_PRIMITIVE_TYPE_END; i++) {
        struct ast_builtin_type* builtin = malloc(sizeof(struct ast_builtin_type));
        builtin->kind = i;
        builtin->size = primitive_type_sizes[i];
        ptr_list_add(&program->types, (const void*) builtin);
    }
}

const struct ast_generic_type* ast_lookup_type(const struct ast_program* program, ast_type_index_t type_index) {
    if(!type_index)
        return NULL;
    return program->types->data[type_index - 1];
}

ast_type_index_t ast_builtin_type(const struct ast_program* program, enum ast_type_kind builtin_kind) {
    if(builtin_kind < BUILTIN_PRIMITIVE_TYPE_START || builtin_kind > BUILTIN_PRIMITIVE_TYPE_END)
        return 0;

    for(size_t i = 0; i < program->types->size; i++)
        if(AST_AS_GENERIC_TYPE(program->types->data[i])->kind == builtin_kind)
            return (ast_type_index_t) i + 1;
    
    assert(false);
}

void ast_true_init(struct ast_generic_expr* expr, struct location* loc) {
    memset(expr, 0, sizeof(struct ast_generic_expr));

    expr->kind = EXPR_TRUE;
    expr->loc = *loc;
    expr->kind = PRIMITIVE_TYPE_TO_INDEX(TYPE_BOOL);
}

void ast_false_init(struct ast_generic_expr* expr, struct location* loc) {
    memset(expr, 0, sizeof(struct ast_generic_expr));

    expr->kind = EXPR_FALSE;
    expr->loc = *loc;
    expr->kind = PRIMITIVE_TYPE_TO_INDEX(TYPE_BOOL);
}

void ast_intlit_init(struct ast_intlit_expr *lit, struct location *loc, uint64_t value) {
    memset(lit, 0, sizeof(struct ast_intlit_expr));

    lit->hdr.kind = EXPR_INTLIT;
    lit->hdr.loc = *loc;
    lit->value = value;
    lit->hdr.kind = PRIMITIVE_TYPE_TO_INDEX(value > INT64_MAX ? TYPE_UINT64 : value > INT32_MAX ? TYPE_INT64 : TYPE_INT32); 
}

void ast_floatlit_init(struct ast_floatlit_expr *lit, struct location *loc, double value) {
    memset(lit, 0, sizeof(struct ast_floatlit_expr));

    lit->hdr.kind = EXPR_FLOATLIT;
    lit->hdr.loc = *loc;
    lit->value = value;
    lit->hdr.kind = PRIMITIVE_TYPE_TO_INDEX(TYPE_FLOAT64);
}

void ast_charlit_init(struct ast_charlit_expr *lit, struct location *loc, bool unicode, wchar_t value) {
    memset(lit, 0, sizeof(struct ast_charlit_expr));

    lit->hdr.kind = EXPR_CHARLIT;
    lit->hdr.loc = *loc;
    lit->unicode = unicode;
    lit->value = value;
    
    lit->hdr.kind = PRIMITIVE_TYPE_TO_INDEX(unicode ? TYPE_UINT16 : TYPE_CHAR);
}

void ast_stringlit_init(struct ast_stringlit_expr *lit, struct location *loc, const char *value) {
    memset(lit, 0, sizeof(struct ast_stringlit_expr));

    lit->hdr.kind = EXPR_STRINGLIT;
    lit->hdr.loc = *loc;
    lit->value = value;
    lit->length = strlen(value);

    // TODO: lit->hdr.kind
}

void ast_typecast_init(struct ast_typecast_expr* typecast, struct location loc, ast_type_index_t result_type, struct ast_generic_expr* expr) {
    memset(typecast, 0, sizeof(struct ast_typecast_expr));

    typecast->hdr.kind = EXPR_TYPECAST;
    typecast->hdr.loc = loc;
    typecast->hdr.type = result_type;
    typecast->expr = expr;
}

void ast_valof_init(struct ast_valof_expr* valof, struct location* loc) {
    memset(valof, 0, sizeof(struct ast_valof_expr));

    valof->hdr.kind = EXPR_VALOF;
    valof->hdr.loc = *loc;
}

void ast_ident_expr_init(struct ast_ident_expr* expr, const struct location* loc, const char* ident) {
    memset(expr, 0, sizeof(struct ast_ident_expr));

    expr->hdr.kind = EXPR_IDENT;
    expr->hdr.loc = *loc;
    expr->ident = ident;
}

void ast_funccall_init(struct ast_funccall_expr* call, const struct location* loc, struct ast_generic_expr* callee) {
    memset(call, 0, sizeof(struct ast_funccall_expr));

    call->hdr.kind = EXPR_FUNCCALL;
    call->hdr.loc = *loc;
    call->callee = callee;
    call->params = ptr_list_init();
}

void ast_expr_stmt_init(struct ast_expr_stmt* stmt, const struct location* loc, struct ast_generic_expr* expr) {
    memset(stmt, 0, sizeof(struct ast_expr_stmt));

    stmt->kind = STMT_EXPR;
    stmt->loc = *loc;
    stmt->expr = expr;
}

void ast_block_stmt_init(struct ast_block_stmt* block, const struct location* loc) {
    memset(block, 0, sizeof(struct ast_block_stmt));

    block->kind = STMT_BLOCK;
    block->loc = *loc;
    block->stmts = ptr_list_init();
}

void ast_block_stmt_add(struct ast_block_stmt* block, struct ast_generic_stmt* stmt) {
    ptr_list_add(&block->stmts, stmt);
}

void ast_resultis_stmt_init(struct ast_resultis_stmt* stmt, const struct location* loc) {
    memset(stmt, 0, sizeof(struct ast_resultis_stmt));

    stmt->kind = STMT_RESULTIS;
    stmt->loc = *loc;
}

ast_type_index_t ast_generic_decl_type(struct ast_generic_decl* decl) {
    switch(decl->hdr.kind) {
    case DECL_GLOBAL:
        return AST_CAST_DECL(decl, global)->type;
    case DECL_STATIC:
        return AST_CAST_DECL(decl, static)->type;
    case DECL_MANIFEST:
        return AST_CAST_DECL(decl, manifest)->type;
    default:
        assert(false);
    }
}

void ast_generic_decl_set_type(struct ast_generic_decl* decl, ast_type_index_t type_index) {
    switch(decl->hdr.kind) {
    case DECL_GLOBAL:
        AST_CAST_DECL(decl, global)->type = type_index;
        break;
    case DECL_STATIC:
        AST_CAST_DECL(decl, static)->type = type_index;
        break;
    case DECL_MANIFEST:
        AST_CAST_DECL(decl, manifest)->type = type_index;
        break;
    default:
        assert(false);
    }
}

void ast_generic_decl_set_expr(struct ast_generic_decl* decl, struct ast_generic_expr* expr) {
    switch(decl->hdr.kind) {
    case DECL_GLOBAL:
        AST_CAST_DECL(decl, global)->expr = expr;
        break;
    case DECL_STATIC:
        AST_CAST_DECL(decl, static)->expr = expr;
        break;
    case DECL_MANIFEST:
        AST_CAST_DECL(decl, manifest)->expr = expr;
        break;
    case DECL_FUNCTION:
        AST_CAST_DECL(decl, function)->body_is_stmt = false;
        AST_CAST_DECL(decl, function)->body.expr = expr;
        if(!AST_CAST_DECL(decl, function)->return_type)
            AST_CAST_DECL(decl, function)->return_type = expr->type;
        break;
    default:
        assert(false);
    }
}

void ast_global_decl_init(struct ast_global_decl* decl, const struct location* loc, const char* ident) {
    memset(decl, 0, sizeof(struct ast_global_decl));

    decl->hdr.kind = DECL_GLOBAL;
    decl->hdr.loc = *loc;
    decl->is_public = true;
    decl->ident = ident;
}

void ast_manifest_decl_init(struct ast_manifest_decl *decl, const struct location *loc, const char* ident) {
    memset(decl, 0, sizeof(struct ast_manifest_decl));

    decl->hdr.kind = DECL_MANIFEST;
    decl->hdr.loc = *loc;
    decl->ident = ident;
}

void ast_static_decl_init(struct ast_static_decl* decl, const struct location* loc, const char* ident) {
    memset(decl, 0, sizeof(struct ast_static_decl));

    decl->hdr.kind = DECL_STATIC;
    decl->hdr.loc = *loc;
    decl->ident = ident;
}

void ast_param_init(struct ast_param* param, const struct location* loc, const char* ident) {
    memset(param, 0, sizeof(struct ast_param));

    param->hdr.loc = *loc;
    param->ident = ident;
}

bool ast_param_has_default_value(struct ast_param* param) {
    return param->default_value != NULL;
}

void ast_function_decl_init(struct ast_function_decl* decl, const struct location* loc, const char* ident, bool tailcall_recursive) {
    memset(decl, 0, sizeof(struct ast_function_decl));

    decl->hdr.kind = DECL_FUNCTION;
    decl->hdr.loc = *loc;
    decl->ident = ident;
    decl->tailcall_recursive = tailcall_recursive;

    decl->params = ptr_list_init();
}

void ast_function_decl_add_param(struct ast_function_decl* decl, struct ast_param* param) {
    ptr_list_add(&decl->params, param);
    if(!param->default_value)
        decl->required_params++;
}

void ast_function_decl_set_stmt(struct ast_function_decl* decl, struct ast_generic_stmt* stmt) {
    decl->body_is_stmt = true;
    decl->body.stmt = stmt;
    decl->return_type = PRIMITIVE_TYPE_TO_INDEX(TYPE_UNIT);
}

