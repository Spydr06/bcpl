#include "token.h"

#include <inttypes.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <limits.h>

#define KW_TOK(_kind) (struct token){.kind = (_kind)}
#define ERR_TOK(err) (struct token){.kind = LEX_ERROR, .val.string = (err)}

struct token read_str_constant(FILE* file) {
    char c;
    size_t start = ftell(file); 

    while((c = fgetc(file)) != '\'')
    {
        switch(c) {
        case '*':
            c = fgetc(file);
            if(strchr("'*nsbt", c) == NULL)
                return ERR_TOK("invalid escape character in string constant");
            break;
        case EOF:
        case '\n':
            return ERR_TOK("unexpected end of line; expect `'`");
        }
    }
    size_t end = ftell(file);

    char* strval = malloc(end - start);
    fread(strval, sizeof(char), end - start - 1, file);
    fseek(file, end, SEEK_SET);

    return (struct token){.kind = STRINGCONST, .val.string = strval};
}

struct token read_num_constant(FILE* file, bool maybe_octal) {
    size_t start = ftell(file) - 1; // first char was already read

    char c = fgetc(file);
    bool octal = maybe_octal && isspace(c) && isdigit(c = fgetc(file));
    
    if(!octal)
        fseek(file, start, SEEK_SET);
    else
        start = ftell(file) - 1;

    while(isdigit(c)) c = fgetc(file);
    fseek(file, start, SEEK_SET);

    uint64_t intval;
    int scan_result = octal ? fscanf(file, "%" SCNo64, &intval) : fscanf(file, "%" SCNu64, &intval);
    if(scan_result != 1)
        return ERR_TOK("error parsing numeric constant");


    return (struct token){.kind = NUMBER, .val.integer = intval};
}

static const struct {
    const char* system_word;
    enum token_kind kind;
} system_words[] = {
    {"true", TRUE},
    {"false", FALSE},
    {"valof", VALOF},
    {"lv", LV},
    {"rv", RV},
    {"rem", REM},
    {"div", DIV},
    {"eq", EQ},
    {"ne", NE},
    {"ls", LS},
    {"gr", GR},
    {"le", LE},
    {"ge", GE},
    {"not", NOT},
    {"lshift", LSHIFT},
    {"rshift", RSHIFT},
    {"and", AND},
    {"logand", LOGAND},
    {"logor", LOGOR},
    {"eqv", EQV},
    {"neqv", NEQV},
    {"ass", ASS},
    {"goto", GOTO},
    {"resultis", RESULTIS},
    {"test", TEST},
    {"for", FOR},
    {"if", IF},
    {"unless", UNLESS},
    {"while", WHILE},
    {"until", UNTIL},
    {"repeat", REPEAT},
    {"repeatwhile", REPEATWHILE},
    {"repeatuntil", REPEATUNTIL},
    {"break", BREAK},
    {"return", RETURN},
    {"finish", FINISH},
    {"switchon", SWITCHON},
    {"case", CASE},
    {"default", DEFAULT},
    {"let", LET},
    {"manifest", MANIFEST},
    {"global", GLOBAL},
    {"be", BE},
    {"into", INTO},
    {"to", TO},
    {"do", DO},
    {"vec", VEC},
    {NULL, LEX_ERROR}
};

static enum token_kind get_system_word(const char* word) {
    for(unsigned i = 0; system_words[i].system_word; i++) {
        if(strcmp(system_words[i].system_word, word) == 0)
            return system_words[i].kind;
    }

    return LEX_ERROR;
}

struct token read_alpha_seq(FILE* file, char first_char) {
    size_t start = ftell(file) - 1;
    char c = fgetc(file);
    
    if(islower(first_char)) {
        if(!isalnum(c)) { // single lowercase letter
            char* name = malloc(2);
            name[0] = first_char;
            name[1] = '\0';
            ungetc(c, file);
            return (struct token){.kind = NAME, .val.string = name};
        }

        while(islower(c = fgetc(file)));
        size_t end = ftell(file);

        if(isalnum(c))
            return ERR_TOK("reseved system word terminated by uppercase letter or digit");

        fseek(file, start, SEEK_SET);
        
        char* word = calloc(end - start, sizeof(char));
        fread(word, sizeof(char), end - start - 1, file);

        enum token_kind sysw = get_system_word(word);
        return sysw == LEX_ERROR ? ERR_TOK("unknown system word") : KW_TOK(sysw);
    }

    ungetc(c, file);
    while(isalnum(c = fgetc(file)));
    size_t end = ftell(file);
    fseek(file, start, SEEK_SET);

    char* name = calloc(end - start, sizeof(char));
    fread(name, sizeof(char), end - start - 1, file);

    return (struct token){.kind = NAME, .val.string = name};
}

bool ends_command(enum token_kind kind) {
    switch(kind) {
    case BREAK:
    case RETURN:
    case FINISH:
    case REPEAT:
    case SKET:
    case RKET:
    case SECTKET:
    case NAME:
    case STRINGCONST:
    case NUMBER:
    case TRUE:
    case FALSE:
        return true;
    default:
        return false;
    }
}

bool may_start_command(enum token_kind kind) {
    switch(kind) {
    case TEST:
    case FOR:
    case IF:
    case UNLESS:
    case UNTIL:
    case WHILE:
    case GOTO:
    case RESULTIS:
    case CASE:
    case DEFAULT:
    case BREAK:
    case RETURN:
    case FINISH:
    case SECTBRA:
    case RBRA:
    case VALOF:
    case LV:
    case RV:
    case NAME:
        return true;
    default:
        return false;
    }
}

bool ends_expression(enum token_kind kind) {
    switch(kind) {
    case SKET:
    case RKET:
    case SECTKET:
    case NAME:
    case NUMBER:
    case STRINGCONST:
    case TRUE:
    case FALSE:
        return true;
    default:
        return false;
    }
}

bool must_start_command(enum token_kind kind) {
    switch(kind) {
    case TEST:
    case FOR:
    case IF:
    case UNLESS:
    case UNTIL:
    case WHILE:
    case GOTO:
    case RESULTIS:
    case CASE:
    case DEFAULT:
    case BREAK:
    case RETURN:
    case FINISH:
        return true;
    default:
        return false;
    }
}

struct token next_token(FILE* file, unsigned* line, struct token* prev) {
    char c;
    
repeat:
    ;

    bool newline = false;
    while(isspace(c = fgetc(file))) {
        if(c == '\n') {
            (*line)++;
            newline = true;
        }
    };

    size_t start = ftell(file) - 1;
    
    struct token tok;

    switch(c) {
    case EOF:
        tok = KW_TOK(LEX_EOF);
        break;
    case '(':
        tok = KW_TOK(RBRA);
        break;
    case ')':
        tok = KW_TOK(RKET);
        break;
    case '[':
        tok = KW_TOK(SBRA);
        break;
    case ']':
        tok = KW_TOK(SKET);
        break;
    case '{':
        tok = KW_TOK(SECTBRA);
        break;
    case '}':
        tok = KW_TOK(SECTKET);
        break;
    case '$':
        switch(c = fgetc(file)) {
        case '(':
            tok = KW_TOK(SECTBRA);
            break;
        case ')':
            tok = KW_TOK(SECTKET);
            break;
        default:
            tok = ERR_TOK("unexpected character after `$`; expect `(` or `)`");
            break;
        }
        break;
    case '\'':
        tok = read_str_constant(file);
        break;
    case '+':
        tok = KW_TOK(PLUS);
        break;
    case '-':
        if((c = fgetc(file)) == '>')
            tok = KW_TOK(COND);
        else {
            ungetc(c, file);
            tok = KW_TOK(MINUS);
        }
        break;
    case '*':
        tok = KW_TOK(STAR);
        break;
    case '/':
        if((c = fgetc(file)) == '/') {
            while((c = fgetc(file)) != '\n' && c != EOF);
            goto repeat;
        }
        else {
            ungetc(c, file);
            tok = KW_TOK(DIV);
        }
        break;
    case '=':
        tok = KW_TOK(EQ);
        break;
    case '!':
        if((c = fgetc(file)) == '=')
            tok = KW_TOK(NE);
        else {
            ungetc(c, file);
            tok = ERR_TOK("unknown operator `!`");
        }
        break;
    case ':':
        if((c = fgetc(file)) == '=')
            tok = KW_TOK(ASS);
        else {
            tok = KW_TOK(COLON);
        }
        break;
    case ',':
        tok = KW_TOK(COMMA);
        break;
    case ';':
        tok = KW_TOK(SEMICOLON);
        break;
    case '<':
        if((c = fgetc(file)) == '=')
            tok = KW_TOK(LE);
        else {
            ungetc(c, file);
            tok = KW_TOK(LS);
        }
        break;
    case '>':
        if((c = fgetc(file)) == '=')
            tok = KW_TOK(GE);
        else {
            ungetc(c, file);
            tok = KW_TOK(GR);
        }
        break;
    case '~':
        tok = KW_TOK(NOT);
        break;
    default:
        if(isdigit(c))
            tok = read_num_constant(file, c == '8');
        else if(isalpha(c))
            tok = read_alpha_seq(file, c);
        else
            tok = ERR_TOK("unexpected character");
        break;
    }

    if(newline && ends_command(prev->kind) && may_start_command(tok.kind)) {
        fseek(file, start, SEEK_SET);
        return KW_TOK(SEMICOLON);
    }

    if(!newline && ends_expression(prev->kind) && must_start_command(tok.kind)) {
        fseek(file, start, SEEK_SET);
        return KW_TOK(DO);
    }

    return tok;
}

void lex_error(const char* filename, FILE* fd, unsigned line, const char* error)
{
    size_t pos = ftell(fd);
    size_t line_start = pos;

    while(line_start > 0) {
        fseek(fd, --line_start - 1, SEEK_SET);
        char c;
        if((c = fgetc(fd)) == '\n')
            break;
    }

    size_t line_end = pos;
    fseek(fd, pos, SEEK_SET);
    char c;
    while((c = fgetc(fd)) != '\n' && c != EOF)
        line_end++;

    fseek(fd, line_start, SEEK_SET);
    char* line_str = calloc(line_end - line_start, sizeof(char));
    fread(line_str, line_end - line_start, sizeof(char), fd);

    size_t column = pos - line_start;

    fprintf(stderr, "\033[1m%s:%u:%zu: \033[31merror:\033[0m %s\n", filename, line, column, error);
    fprintf(stderr, " %4d | %s\n", line, line_str);
    fprintf(stderr, "      | %*s\033[35m^~here\033[0m\n", (int) column - 1, "");
    
    exit(EXIT_FAILURE);
}

static const char* const token_kind_strs[] = {
    "LEX_EOF",
    "NUMBER",
    "NAME",
    "STRINGCONST",
    "TRUE",
    "FALSE",
    "VALOF",
    "LV",
    "RV",
    "DIV",
    "REM",
    "PLUS",
    "MINUS",
    "EQ",
    "NE",
    "LS",
    "GR",
    "LE",
    "GE",
    "NOT",
    "LSHIFT",
    "RSHIFT",
    "LOGAND",
    "LOGOR",
    "EQV",
    "NEQV",
    "COND",
    "COMMA",
    "AND",
    "ASS",
    "GOTO",
    "RESULTIS",
    "COLON",
    "TEST",
    "FOR",
    "IF",
    "UNLESS",
    "WHILE",
    "UNTIL",
    "REPEAT",
    "REPEATWHILE",
    "REPEATUNTIL",
    "BREAK",
    "RETURN",
    "FINISH",
    "SWITCHON",
    "CASE",
    "DEFAULT",
    "LET",
    "MANIFEST",
    "GLOBAL",
    "BE",
    "SECTBRA",
    "SECTKET",
    "RBRA",
    "RKET",
    "SBRA",
    "SKET",
    "SEMICOLON",
    "INTO",
    "TO",
    "DO",
    "OR",
    "VEC",
    "STAR",
    [CHAR_MAX] = "LEX_ERROR",
};

void dbg_print_token(struct token* t) {
    switch(t->kind) {
        case NUMBER:
            printf("NUMBER %lu\n", t->val.integer);
            break;
        case NAME:
            printf("NAME %s\n", t->val.string);
            break;
        case STRINGCONST:
            printf("STRINGCONST %s\n", t->val.string);
            break;
        case LEX_ERROR:
            printf("LEX_ERROR %s\n", t->val.string);
            break;
        default:
            puts(token_kind_strs[t->kind]);
            break;
    } 
}

#undef KW_TOK
#undef ERR_TOK

