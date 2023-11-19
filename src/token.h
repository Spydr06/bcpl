#ifndef BCPLC_TOKEN_H
#define BCPLC_TOKEN_H

#include "util.h"
#include <stdint.h>
#include <stdio.h>

enum token_kind {
    // constants
    TOKEN_IDENT, // identifiers
    TOKEN_INTEGER, // integer constants
    TOKEN_FLOAT, // floating-point constants
    TOKEN_STRING, // string constants
    TOKEN_CHAR, // character constant

    // symbols
    TOKEN_LPAREN, // `(`
    TOKEN_RPAREN, // `)`
    TOKEN_LBRACE, // `{`
    TOKEN_RBRACE, // `}`
    TOKEN_LBRACKET, // `[`
    TOKEN_RBRACKET, // `]`
    TOKEN_SEMICOLON, // `;` 
    TOKEN_COMMA, // `,`
    TOKEN_DOT, // `.`
    TOKEN_COLON, // `:`
    TOKEN_ASSIGN, // `:=`
    TOKEN_COND, // `->`
    TOKEN_QMARK, // `?`

    // operators
    TOKEN_PLUS, // `+`
    TOKEN_MINUS, // `-`
    TOKEN_STAR, // `*`
    TOKEN_SLASH, // `/` 

    TOKEN_EQ, // `=`
    TOKEN_NE, // `~=`
    TOKEN_GT, // `>`
    TOKEN_GE, // `>=`
    TOKEN_LT, // `<`
    TOKEN_LE, // `<=`

    TOKEN_NOT, // `~`
    TOKEN_LOGAND, // `&`
    TOKEN_LOGOR, // `|`
    TOKEN_XOR, // `^`
    TOKEN_LSHIFT, // `<<`
    TOKEN_RSHIFT, // `>>`

    // keywords
    TOKEN_TRUE, // `true`
    TOKEN_FALSE, // `false`
    TOKEN_LET, // `let`
    TOKEN_AND, // `and`
    TOKEN_VALOF, // `valof`
    TOKEN_RESULTIS, // `resultis`
    TOKEN_RETURN, // `return`
    TOKEN_FINISH, // `finish`
    TOKEN_SKIP, // `skip`
    TOKEN_REPEAT, // `repeat`
    TOKEN_BREAK, // `break`
    TOKEN_IF, // `if`
    TOKEN_UNLESS, // `unless`
    TOKEN_WHILE, // `while`
    TOKEN_FOR,  // `for`
    TOKEN_UNTIL, // `until`
    TOKEN_SWITCHON, // `switchon`
    TOKEN_MATCH, // `match`
    TOKEN_EVERY, // `every`
    TOKEN_CASE, // `case`
    TOKEN_DEFAULT, // `default`
    TOKEN_DO, // `do`
    TOKEN_TO, // `to`
    TOKEN_BY, // `by`
    TOKEN_OF, // `of` or `::`  
    TOKEN_BE, // `be`
    TOKEN_SECTION, // `section`
    TOKEN_REQUIRE, // `require`
    TOKEN_GLOBAL, // `global`
    TOKEN_MANIFEST, // `manifest`
    TOKEN_STATIC, // `static`
    TOKEN_MOD, // `mod`
    TOKEN_ABS, // `abs`

    // lexer signals
    LEX_ERROR,
    LEX_EOF = 0,
};

struct token {
    enum token_kind kind;
    union {
        const char* string;
        uint64_t integer;
    } val;
};

struct token next_token(FILE* file, unsigned* line, struct token* prev, struct string_list** tags);

void lex_error(const char* filename, FILE* fd, unsigned line, const char* error);

void dbg_print_token(struct token* t);

#endif

