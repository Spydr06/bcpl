#ifndef BCPLC_AST_H
#define BCPLC_AST_H

#include "util.h"

#include <stdbool.h>
#include <stdint.h>
#include <wchar.h>

//
// Pre-declarations
//

typedef struct ast_generic_expr ast_generic_expr_t;
typedef struct ast_generic_stmt ast_generic_stmt_t;
typedef struct ast_generic_type ast_generic_type_t;

//
// Sections
//

struct ast_section {
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

#define AST_TYPE_HDR(_kind) \
    enum ast_type_kind _kind

#define AST_TYPE_EXPR(_generic, _type) \
    ((struct ast_##_type##_type*) ((struct generic_type*) (_generic)))

#define AST_AS_GENERIC_TYPE(_expr) \
    ((struct ast_generic_type*) (_expr))

struct __attribute__((packed)) ast_generic_type {
    AST_TYPE_HDR(kind);
};

struct __attribute__((packed)) ast_builtin_type {
    AST_TYPE_HDR(kind);
};

struct __attribute__((packed)) ast_pointer_type {
    AST_TYPE_HDR(kind);

    ast_generic_type_t* inner;
};

extern const char* const primitive_types[BUILTIN_PRIMITIVE_TYPE_END + 1];

#undef AST_TYPE_HDR

//
// Program
//

struct ast_program {
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
    
    EXPR_TYPECAST,
    EXPR_VALOF,
};

#define AST_EXPR_HDR(_kind, _loc, _type) \
    enum ast_expr_kind _kind;            \
    struct location _loc;                \
    ast_type_index_t _type               \

#define AST_CAST_EXPR(_generic, _type) \
    ((struct ast_##_type##_expr*) ((struct generic_expr*) (_generic)))

#define AST_AS_GENERIC_EXPR(_expr) \
    ((struct ast_generic_expr*) (_expr))

struct __attribute__((packed)) ast_generic_expr {
    AST_EXPR_HDR(kind, loc, type); 
};

void ast_true_init(struct ast_generic_expr* expr, struct location* loc);
void ast_false_init(struct ast_generic_expr* expr, struct location* loc);

struct __attribute__((packed)) ast_intlit_expr {
    AST_EXPR_HDR(kind, loc, type);
    uint64_t value;
};

void ast_intlit_init(struct ast_intlit_expr* lit, struct location* loc, uint64_t value);

struct __attribute__((packed)) ast_floatlit_expr {
    AST_EXPR_HDR(kind, loc, type); 
    double value;
};

void ast_floatlit_init(struct ast_floatlit_expr* lit, struct location* loc, double value);

struct __attribute__((packed)) ast_charlit_expr {
    AST_EXPR_HDR(kind, loc, type);
    
    bool unicode;
    wchar_t value;
};

void ast_charlit_init(struct ast_charlit_expr* lit, struct location* loc, bool unicode, wchar_t value);

struct __attribute__((packed)) ast_stringlit_expr {
    AST_EXPR_HDR(kind, loc, type);

    size_t length;
    const char* value;
};

void ast_stringlit_init(struct ast_stringlit_expr* lit, struct location* loc, const char* value);

struct __attribute__((packed)) ast_typecast_expr {
    AST_EXPR_HDR(kind, loc, result_type);

    struct ast_generic_expr* expr;
};

void ast_typecast_init(struct ast_typecast_expr* typecast, struct location* loc, ast_type_index_t result_type, struct ast_generic_expr* expr);

struct __attribute__((packed)) ast_valof_expr {
    AST_EXPR_HDR(kind, loc, type);
    ast_generic_stmt_t* body;
};

#undef AST_EXPR_HDR

//
// Statements
//

enum ast_stmt_kind {
    STMT_
};

#define AST_STMT_HDR(_kind) \
    enum ast_stmt_kind _kind

#define AST_CAST_STMT(_generic, _type) \
    ((struct ast_##_type##_stmt*) ((struct generic_stmt*) (_generic)))

#define AST_AS_GENERIC_STMT(_stmt) \
    ((struct generic_stmt*) (_stmt))


struct __attribute__((packed)) ast_generic_stmt {
    AST_STMT_HDR(kind);
};

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
    ((struct generic_decl*) (_decl))

struct __attribute__((packed)) ast_generic_decl {
   AST_DECL_HDR(kind, loc, ident, is_public);
};

ast_type_index_t ast_generic_decl_type(struct ast_generic_decl* decl);
void ast_generic_decl_set_type(struct ast_generic_decl* decl, ast_type_index_t type_index);

void ast_generic_decl_set_expr(struct ast_generic_decl* decl, struct ast_generic_expr* expr);

struct __attribute__((packed)) ast_global_decl {
    AST_DECL_HDR(kind, loc, ident, is_public);

    ast_type_index_t type;
    struct ast_generic_expr* expr;
};

void ast_global_decl_init(struct ast_global_decl* decl, const struct location* loc, const char* ident);

struct __attribute__((packed)) ast_manifest_decl {
    AST_DECL_HDR(kind, loc, ident, __is_public);

    ast_type_index_t type;
    struct ast_generic_expr* expr;
};

void ast_manifest_decl_init(struct ast_manifest_decl* decl, const struct location* loc, const char* ident);

struct __attribute__((packed)) ast_static_decl {
    AST_DECL_HDR(kind, loc, ident, __is_public);

    ast_type_index_t type;
    struct ast_generic_expr* expr;
};

void ast_static_decl_init(struct ast_static_decl* decl, const struct location* loc, const char* ident);

struct __attribute__((packed)) ast_param_decl {
    AST_DECL_HDR(kind, loc, ident, __is_public);
};

struct __attribute__((packed)) ast_function_decl {
    AST_DECL_HDR(kind, loc, ident, is_public);

    struct ptr_list* params; // ast_param_decl
    
    ast_type_index_t return_type;
    bool tailcall_recursive; // recursiveness indicated by the `and` declaration

    bool body_is_stmt;
    union {
        struct ast_generic_expr* expr;
        struct ast_generic_stmt* stmt;
    } body;
};

#undef AST_OBJECT_HDR

#endif /* BCPLC_AST_H */

