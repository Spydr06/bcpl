#ifndef BCPLC_TOKEN_H
#define BCPLC_TOKEN_H

#include <stdint.h>
#include <stdio.h>

enum token_kind {
    NUMBER = 1,
    NAME,
    STRINGCONST,
    TRUE,
    FALSE,
    VALOF,
    LV,
    RV,
    DIV,
    REM,
    PLUS,
    MINUS,
    EQ,
    NE,
    LS,
    GR,
    LE,
    GE,
    NOT,
    LSHIFT,
    RSHIFT,
    LOGAND,
    LOGOR,
    EQV,
    NEQV,
    COND,
    COMMA,
    AND,
    ASS,
    GOTO,
    RESULTIS,
    COLON,
    TEST,
    FOR,
    IF,
    UNLESS,
    WHILE,
    UNTIL,
    REPEAT,
    REPEATWHILE,
    REPEATUNTIL,
    BREAK,
    RETURN,
    FINISH,
    SWITCHON,
    CASE,
    DEFAULT,
    LET,
    MANIFEST,
    GLOBAL,
    BE,
    SECTBRA,
    SECTKET,
    RBRA,
    RKET,
    SBRA,
    SKET,
    SEMICOLON,
    INTO,
    TO,
    DO,
    OR,
    VEC,
    STAR,
    LEX_EOF = 0,
    LEX_ERROR = -1
};

struct token {
    enum token_kind kind;
    union {
        const char* string;
        uint64_t integer;
    } val;
};

struct token next_token(FILE* file, unsigned* line, struct token* prev);

void lex_error(const char* filename, FILE* fd, unsigned line, const char* error);

void dbg_print_token(struct token* t);

#endif

