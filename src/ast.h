#ifndef BCPLC_AST_H
#define BCPLC_AST_H

#include "util.h"

#include <stdbool.h>
#include <stdint.h>
#include <wchar.h>

#define AST_STRUCT __attribute__((packed, aligned(8)))

//
// Pre-declarations
//

typedef struct ast_generic_expr ast_generic_expr_t;
typedef struct ast_generic_stmt ast_generic_stmt_t;
typedef struct ast_generic_type ast_generic_type_t;

//
// Sections
//

struct AST_STRUCT ast_section {
    struct location loc;
    const char* ident;

    struct string_list* required;

    struct ptr_list* declarations; // ast_generic_decl
};

void ast_section_init(struct ast_section* section, const struct location* loc);

//
// Types
//

#define TYPE_NOT_FOUND ((ast_type_index_t) 0)

typedef uint32_t ast_type_index_t;

#define PRIMITIVE_TYPE_TO_INDEX(prim_type) ((ast_type_index_t) (prim_type + 1))

enum ast_type_kind {
#define BUILTIN_PRIMITIVE_TYPE_START TYPE_UINT8
    TYPE_UINT8,
    TYPE_UINT16,
    TYPE_UINT32,
    TYPE_UINT64,

    TYPE_INT8,
    TYPE_INT16,
    TYPE_INT32,
    TYPE_INT64,

    TYPE_FLOAT32,
    TYPE_FLOAT64,

    TYPE_BOOL,
    TYPE_CHAR,

#define BUILTIN_PRIMITIVE_TYPE_END TYPE_UNIT
    TYPE_UNIT,

    TYPE_POINTER,
    // TYPE_ARRAY
    // TYPE_TABLE
    // TYPE_FUNCTION
    // TYPE_GENERIC
    // TYPE_...
};

#define AST_TYPE_HDR(_kind, _size) \
    enum ast_type_kind _kind;       \
    uint32_t _size

#define AST_TYPE_EXPR(_generic, _type) \
    ((struct ast_##_type##_type*) ((struct generic_type*) (_generic)))

#define AST_AS_GENERIC_TYPE(_expr) \
    ((struct ast_generic_type*) (_expr))

struct AST_STRUCT ast_generic_type {
    AST_TYPE_HDR(kind, size);
};

struct AST_STRUCT ast_builtin_type {
    AST_TYPE_HDR(kind, size);
};

struct AST_STRUCT ast_pointer_type {
    AST_TYPE_HDR(kind, size);

    ast_generic_type_t* inner;
};

extern const char* const primitive_types[BUILTIN_PRIMITIVE_TYPE_END + 1];

#undef AST_TYPE_HDR

//
// Program
//

struct AST_STRUCT ast_program {
    struct ptr_list* sections;
    struct ptr_list* types;
};

void ast_program_init(struct ast_program* program);

const struct ast_generic_type* ast_lookup_type(const struct ast_program* program, ast_type_index_t type_index);
ast_type_index_t ast_builtin_type(const struct ast_program* program, enum ast_type_kind builtin_kind);

//
// Expressions
//

enum ast_expr_kind {
    EXPR_INTLIT,
    EXPR_FLOATLIT,
    EXPR_CHARLIT,
    EXPR_STRINGLIT,
    EXPR_TRUE,
    EXPR_FALSE,
    
    EXPR_IDENT,
    EXPR_TYPECAST,
    EXPR_VALOF,
    EXPR_FUNCCALL,
};

#define AST_EXPR_HDR(_kind, _loc, _type) \
    enum ast_expr_kind _kind;            \
    struct location _loc;                \
    ast_type_index_t _type               \

#define AST_CAST_EXPR(_generic, _type) \
    ((struct ast_##_type##_expr*) ((struct ast_generic_expr*) (_generic)))

#define AST_AS_GENERIC_EXPR(_expr) \
    ((struct ast_generic_expr*) (_expr))

struct AST_STRUCT ast_generic_expr {
    AST_EXPR_HDR(kind, loc, type); 
};

void ast_true_init(struct ast_generic_expr* expr, struct location* loc);
void ast_false_init(struct ast_generic_expr* expr, struct location* loc);

struct AST_STRUCT ast_intlit_expr {
    struct ast_generic_expr hdr;
    uint64_t value;
};

void ast_intlit_init(struct ast_intlit_expr* lit, struct location* loc, uint64_t value);

struct AST_STRUCT ast_floatlit_expr {
    struct ast_generic_expr hdr;
    double value;
};

void ast_floatlit_init(struct ast_floatlit_expr* lit, struct location* loc, double value);

struct AST_STRUCT ast_charlit_expr {
    struct ast_generic_expr hdr;
    
    bool unicode;
    wchar_t value;
};

void ast_charlit_init(struct ast_charlit_expr* lit, struct location* loc, bool unicode, wchar_t value);

struct AST_STRUCT ast_stringlit_expr {
    struct ast_generic_expr hdr;

    size_t length;
    const char* value;
};

void ast_stringlit_init(struct ast_stringlit_expr* lit, struct location* loc, const char* value);

struct AST_STRUCT ast_typecast_expr {
    struct ast_generic_expr hdr;

    struct ast_generic_expr* expr;
};

void ast_typecast_init(struct ast_typecast_expr* typecast, struct location loc, ast_type_index_t result_type, struct ast_generic_expr* expr);

struct AST_STRUCT ast_valof_expr {
    struct ast_generic_expr hdr;
    ast_generic_stmt_t* body;
};

void ast_valof_init(struct ast_valof_expr* valof, struct location* loc);

struct AST_STRUCT ast_ident_expr {
    struct ast_generic_expr hdr;

    const char* ident;
};

void ast_ident_expr_init(struct ast_ident_expr* expr, const struct location* loc, const char* ident);

struct AST_STRUCT ast_funccall_expr {
    struct ast_generic_expr hdr;

    struct ast_generic_expr* callee;
    struct ptr_list* params;
};

void ast_funccall_init(struct ast_funccall_expr* call, const struct location* loc, struct ast_generic_expr* callee);

#undef AST_EXPR_HDR

//
// Statements
//

enum ast_stmt_kind {
    STMT_EXPR,
    STMT_BLOCK,
    STMT_RESULTIS,
};

#define AST_STMT_HDR(_kind, _loc) \
    enum ast_stmt_kind _kind;     \
    struct location _loc

#define AST_CAST_STMT(_generic, _type) \
    ((struct ast_##_type##_stmt*) ((struct generic_stmt*) (_generic)))

#define AST_AS_GENERIC_STMT(_stmt) \
    ((struct generic_stmt*) (_stmt))

struct AST_STRUCT ast_generic_stmt {
    AST_STMT_HDR(kind, loc);
};

struct AST_STRUCT ast_expr_stmt {
    AST_STMT_HDR(kind, loc);

    struct ast_generic_expr* expr;
};

void ast_expr_stmt_init(struct ast_expr_stmt* stmt, const struct location* loc, struct ast_generic_expr* expr);

struct AST_STRUCT ast_block_stmt {
    AST_STMT_HDR(kind, loc);

    struct ptr_list* stmts;
};

void ast_block_stmt_init(struct ast_block_stmt* block, const struct location* loc);
void ast_block_stmt_add(struct ast_block_stmt* block, struct ast_generic_stmt* stmt);

struct AST_STRUCT ast_resultis_stmt {
    AST_STMT_HDR(kind, loc);

    struct ast_generic_expr* expr;
};

void ast_resultis_stmt_init(struct ast_resultis_stmt* stmt, const struct location* loc);

#undef AST_STMT_HDR

//
// Declarations
//

enum ast_decl_kind {
    DECL_GLOBAL,
    DECL_MANIFEST,
    DECL_STATIC,
    DECL_FUNCTION
};

#define AST_DECL_HDR(_kind, _loc, _ident, _is_public)   \
    enum ast_decl_kind _kind;                           \
    struct location _loc;                               \
    const char* _ident;                                 \
    bool _is_public 

#define AST_CAST_DECL(_generic, _type) \
    ((struct ast_##_type##_decl*) ((struct generic_decl*) (_generic)))

#define AST_AS_GENERIC_DECL(_decl) \
    ((struct ast_generic_decl*) (_decl))

struct AST_STRUCT ast_generic_decl {
   AST_DECL_HDR(kind, loc, ident, is_public);
};

ast_type_index_t ast_generic_decl_type(struct ast_generic_decl* decl);
void ast_generic_decl_set_type(struct ast_generic_decl* decl, ast_type_index_t type_index);

void ast_generic_decl_set_expr(struct ast_generic_decl* decl, struct ast_generic_expr* expr);

struct AST_STRUCT ast_global_decl {
    AST_DECL_HDR(kind, loc, ident, is_public);

    ast_type_index_t type;
    struct ast_generic_expr* expr;
};

void ast_global_decl_init(struct ast_global_decl* decl, const struct location* loc, const char* ident);

struct AST_STRUCT ast_manifest_decl {
    AST_DECL_HDR(kind, loc, ident, __is_public);

    ast_type_index_t type;
    struct ast_generic_expr* expr;
};

void ast_manifest_decl_init(struct ast_manifest_decl* decl, const struct location* loc, const char* ident);

struct AST_STRUCT ast_static_decl {
    AST_DECL_HDR(kind, loc, ident, __is_public);

    ast_type_index_t type;
    struct ast_generic_expr* expr;
};

void ast_static_decl_init(struct ast_static_decl* decl, const struct location* loc, const char* ident);

struct AST_STRUCT ast_param {
    struct location loc;
    const char* ident;
    ast_type_index_t type;
    struct ast_generic_expr* default_value;
};

void ast_param_init(struct ast_param* param, const struct location* loc, const char* ident);
bool ast_param_has_default_value(struct ast_param* param);

struct AST_STRUCT ast_function_decl {
    AST_DECL_HDR(kind, loc, ident, is_public);

    struct ptr_list* params; // ast_param
    uint32_t required_params;

    ast_type_index_t return_type;
    bool tailcall_recursive; // recursiveness indicated by the `and` declaration

    bool body_is_stmt;
    union {
        struct ast_generic_expr* expr;
        struct ast_generic_stmt* stmt;
    } body;
};

void ast_function_decl_init(struct ast_function_decl* decl, const struct location* loc, const char* ident, bool tailcall_recursive);
void ast_function_decl_add_param(struct ast_function_decl* decl, struct ast_param* param);
void ast_function_decl_set_stmt(struct ast_function_decl* decl, struct ast_generic_stmt* stmt);

#undef AST_OBJECT_HDR

#endif /* BCPLC_AST_H */

