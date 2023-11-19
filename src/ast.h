#ifndef BCPLC_AST_H
#define BCPLC_AST_H

#include "util.h"

#include <stdint.h>

enum jcom {
    JCOM_NEXT,
    JCOM_EXIT,
    JCOM_BREAK,
    JCOM_LOOP,
    JCOM_ENDCASE,
    JCOM_RETURN
};

enum bexp_kind {
    BEXP_NAME, // identifiers
    BEXP_CONST, // numerical constant
    BEXP_STRING, // string constant
    BEXP_SLCT, // vector constant
    
    // jcom
    BEXP_NEXT,
    BEXP_EXIT,
    BEXP_BREAK,
    BEXP_LOOP, 
    BEXP_ENDCASE,
    BEXP_RETURN,
    BEXP_PAREN, // ( <bexp> )
    BEXP_FLOAT, // FLOAT <bexp>
    BEXP_FIX, // FIX <bexp>
    BEXP_BANG, // !<bexp>
    BEXP_AT, // @<bexp>
    
    // posop
    BEXP_PLUS,
    BEXP_MINUS,
    BEXP_ABS,
    BEXP_FLTPLUS,
    BEXP_FLTMINUS,
    BEXP_FLTABS,

    BEXP_NOT, // NOT <bexp>
    BEXP_TABLE, // ???
    BEXP_MATCH,
    BEXP_EVERY,
    BEXP_VALOF
};

typedef struct mlist mlist_t;

typedef struct bexp bexp_t;
struct bexp {
    enum bexp_kind kind;

    union {
        // name
        const char* name;

        //const
        uint64_t const_val;
        
        // string
        const char* string;

        // (), FLOAT, FIX, !, @, posop, NOT
        bexp_t* expr;

        // SLCT, TABLE
        struct ptr_list* exprs;

        // MATCH / EVERY
        struct {
            struct ptr_list* conditions;
            mlist_t* bodies;
        } em;

        // VALOF

    } value;
};

struct mlist {
    
};

typedef struct manifest_decl manifest_decl_t;
struct manifest_decl {
    const char* name;
    struct expr* expr;
    bool flt;
    
    manifest_decl_t* next;
};

struct function_def {
    const char* name;
    struct string_list* args;
};

#endif /* BCPLC_AST_H */

