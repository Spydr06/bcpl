#ifndef BCPLC_AST_H
#define BCPLC_AST_H

#include "util.h"

#include <stdbool.h>
#include <stdint.h>

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
    const char* ident;
    struct string_list* required;

    struct ptr_list* declarations; // ast_generic_decl
};

//
// Program
//

struct ast_program {
    struct ptr_list* sections;
    struct ptr_list* types;
};

void ast_program_init(struct ast_program* program);

//
// Types
//

typedef uint32_t ast_type_index_t;

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
    ((struct generic_type*) (_expr))

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

#undef AST_TYPE_HDR

//
// Expressions
//

enum ast_expr_kind {
    EXPR_VALOF
};

#define AST_EXPR_HDR(_kind, _type) \
    enum ast_expr_kind _kind;      \
    ast_type_index_t _type         \

#define AST_CAST_EXPR(_generic, _type) \
    ((struct ast_##_type##_expr*) ((struct generic_expr*) (_generic)))

#define AST_AS_GENERIC_EXPR(_expr) \
    ((struct generic_expr*) (_expr))

struct __attribute__((packed)) ast_generic_expr {
    AST_EXPR_HDR(kind, type); 
};

struct __attribute__((packed)) ast_valof_expr {
    AST_EXPR_HDR(kind, type);
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

#define AST_DECL_HDR(_kind, _ident, _is_public) \
    enum ast_decl_kind _kind;                   \
    const char* _ident;                         \
    bool _is_public 

#define AST_CAST_DECL(_generic, _type) \
    ((struct ast_##_type##_decl*) ((struct generic_decl*) (_generic)))

#define AST_AS_GENERIC_DECL(_decl) \
    ((struct generic_decl*) (_decl))

struct __attribute__((packed)) ast_generic_decl {
   AST_DECL_HDR(kind, ident, is_public);
};

struct __attribute__((packed)) ast_global_decl {
    AST_DECL_HDR(kind, ident, is_public);
};

struct __attribute__((packed)) ast_manifest_decl {
    AST_DECL_HDR(kind, ident, __is_public);
};

struct __attribute__((packed)) ast_static_decl {
    AST_DECL_HDR(kind, ident, is_public);
};

struct __attribute__((packed)) ast_param_decl {
    AST_DECL_HDR(kind, ident, __is_public);
};

struct __attribute__((packed)) ast_function_decl {
    AST_DECL_HDR(kind, ident, is_public);

    struct ptr_list* params; // ast_param_decl
    
    ast_type_index_t return_type;

    bool body_is_stmt;
    union {
        struct ast_expr* expr;
        struct ast_stmt* stmt;
    } body;
};

#undef AST_OBJECT_HDR

#endif /* BCPLC_AST_H */

