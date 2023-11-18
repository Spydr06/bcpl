#include "token.h"
#include "util.h"

#include <errno.h>
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

enum format_kind {
    DECIMAL = 10,
    BINARY = 2,
    OCTAL = 8,
    HEXADECIMAL = 16
};

static const char* const digits[] = {
    [DECIMAL] = "0123456789_",
    [BINARY] = "01_",
    [OCTAL] = "01234567_",
    [HEXADECIMAL] = "0123456789aAbBcCdDeEfF_"
};

static const char basic_escape_codes[] = {
    ['n'] = '\n',
    ['c'] = '\r',
    ['p'] = '\f',
    ['s'] = ' ',
    ['b'] = '\b',
    ['t'] = '\t',
    ['e'] = '\033',
    ['"'] = '"',
    ['\''] = '\'',
    ['*'] = '*'
};

enum string_enconding {
    ASCII,
    UTF8,
    GB2312
};

static const char* resolve_escape_codes(char* strval) {
    enum string_enconding encoding = ASCII;

    for(; strlen(strval); strval++) {
        if(*strval != '*')
            continue; // TODO: handle UTF8 and GB2312

        if(strchr("nNcCpPsSbBtTeE\"'*", strval[1]) != NULL) {
            *strval = basic_escape_codes[(int) strval[1]];
            strshl(strval + 1, 1);
        }
        else if(strval[1] == 'x' || strval[1] == 'X') {
            if(sscanf(strval + 2, "%2hhx", (unsigned char*) strval) != 1)
                return "invalid escape sequence, expect `*xhh`, where `h` is 0-F";
            strshl(strval + 1, 3);
        }
        else if(isdigit(strval[1])) {
            if(sscanf(strval + 1, "%3hhu", (unsigned char*) strval) != 1)
                return "invalid escape sequence, expect `*ddd`, where `d` is 0-9";
            strshl(strval + 1, 3);
        }
        else if(strval[1] == '#') {
            if(strval[2] == 'g') {
                encoding = GB2312;
                strshl(strval, 3);
                strval--;
            }
            else if(strval[2] == 'u') {
                encoding = UTF8;
                strshl(strval, 3);
                strval--;
            }
            else if(isxdigit(strval[2])) {
                if(encoding == UTF8 && sscanf(strval + 2, "%4hx", (uint16_t*) strval) != 1)
                    return "invalid escape sequence, expect `*hhhh`, where `h` is 0-F";
                if(encoding == GB2312 && sscanf(strval + 2, "%4hu", (uint16_t*) strval) != 1)
                    return "invalid escape sequence, expect `*dddd`, where `d` is 0-9";
                if(encoding == ASCII)
                    return "`*#hhhh` escape sequence can only be used in UTF-8 or GB2312 mode";
                strval += sizeof(uint16_t);
                strshl(strval + 1, 3);
            }
            else
                return "invalid escape sequence after `*#`";
        }
        else
            return "invalid escape sequence";
    }

    return NULL;
}

struct token read_str_constant(FILE* file, char quote) {
    char c;
    size_t start = ftell(file); 

    while((c = fgetc(file)) != quote) {
        if(c == EOF || c == '\n')
            return ERR_TOK("unexpected end of line; expect `'`");
    }
    
    size_t end = ftell(file);
    fseek(file, start, SEEK_SET);

    char* strval = calloc(end - start, sizeof(char));
    fread(strval, sizeof(char), end - start - 1, file);
    fseek(file, end, SEEK_SET);

    const char* err = resolve_escape_codes(strval);
    if(err)
        return ERR_TOK(err);
    return (struct token){.kind = STRINGCONST, .val.string = strval};
}

// TODO: fp support
struct token read_num_constant(FILE* file, enum format_kind format) {
    char buf[65];
    unsigned i = 0;
    
    while((isalnum(buf[i] = fgetc(file)) || buf[i] == '_') && buf[i] != EOF) {
        if(i > 64)
            return ERR_TOK("numeric constant too long");
        if(strchr(digits[format], buf[i]) == NULL)
            return ERR_TOK("unexpected character in numeric constant");
        if(buf[i] != '_')
            i++;
    }
    buf[i] = '\0';

    uint64_t val = strtoull(buf, NULL, format);
    if(val == ULLONG_MAX && errno)
        return ERR_TOK("invalid numeric constant");

    return (struct token){.kind = NUMBER, .val.integer = val};
}

static const struct {
    const char* system_word;
    enum token_kind kind;
} system_words[] = {
    {"true", TRUE}, {"TRUE", TRUE},
    {"false", FALSE}, {"FALSE", FALSE},
    {"valof", VALOF}, {"VALOF", VALOF},
    {"lv", LV}, {"LV", LV},
    {"rv", RV}, {"RV", RV},
    {"rem", REM}, {"REM", REM},
    {"div", DIV}, {"DIV", DIV},
    {"eq", EQ}, {"EQ", EQ},
    {"ne", NE}, {"NE", NE},
    {"ls", LS}, {"LS", LS},
    {"gr", GR}, {"GR", GR},
    {"le", LE}, {"LE", LE},
    {"ge", GE}, {"GE", GE},
    {"not", NOT}, {"NOT", NOT},
    {"lshift", LSHIFT}, {"LSHIFT", LSHIFT},
    {"rshift", RSHIFT}, {"RSHIFT", RSHIFT},
    {"and", AND}, {"AND", AND},
    {"logand", LOGAND}, {"LOGAND", LOGAND},
    {"logor", LOGOR}, {"LOGOR", LOGOR},
    {"eqv", EQV}, {"EQV", EQV},
    {"neqv", NEQV}, {"NEQV", NEQV},
    {"ass", ASS}, {"ASS", ASS},
    {"goto", GOTO}, {"GOTO", GOTO},
    {"resultis", RESULTIS}, {"RESULTIS", RESULTIS},
    {"test", TEST}, {"TEST", TEST},
    {"for", FOR}, {"FOR", FOR},
    {"if", IF}, {"IF", IF},
    {"unless", UNLESS}, {"UNLESS", UNLESS},
    {"while", WHILE}, {"WHILE", WHILE},
    {"until", UNTIL}, {"UNTIL", UNTIL},
    {"repeat", REPEAT}, {"REPEAT", REPEAT},
    {"repeatwhile", REPEATWHILE}, {"REPEATWHILE", REPEATWHILE},
    {"repeatuntil", REPEATUNTIL}, {"REPEATUNTIL", REPEATUNTIL},
    {"break", BREAK}, {"BREAK", BREAK},
    {"return", RETURN}, {"RETURN", RETURN},
    {"finish", FINISH}, {"FINISH", FINISH},
    {"switchon", SWITCHON}, {"SWITCHON", SWITCHON},
    {"match", MATCH}, {"MATCH", MATCH},
    {"every", EVERY}, {"EVERY", EVERY},
    {"case", CASE}, {"CASE", CASE},
    {"default", DEFAULT}, {"DEFAULT", DEFAULT},
    {"let", LET}, {"LET", LET},
    {"manifest", MANIFEST}, {"MANIFEST", MANIFEST},
    {"global", GLOBAL}, {"GLOBAL", GLOBAL},
    {"be", BE}, {"BE", BE},
    {"into", INTO}, {"INTO", INTO},
    {"to", TO}, {"TO", TO},
    {"do", DO}, {"DO", DO},
    {"vec", VEC}, {"VEC", VEC},
    {"bitsperbcplword", BITSPERBCPLWORD}, {"BITSPERBCPLWORD", BITSPERBCPLWORD},
    {"slct", SLCT}, {"SLCT", SLCT},
    {NULL, LEX_ERROR}
};

static enum token_kind get_system_word(const char* word) {
    for(unsigned i = 0; system_words[i].system_word; i++) {
        if(strcmp(system_words[i].system_word, word) == 0)
            return system_words[i].kind;
    }

    return NAME;
}

static inline bool is_word_char(char c) {
    return isalnum(c) || c == '_' || c == '.';
}

struct token read_alpha_seq(FILE* file) {
    size_t start = ftell(file) - 1;
    while(is_word_char(fgetc(file)));
    size_t end = ftell(file);
    fseek(file, start, SEEK_SET);

    char* word = calloc(end - start, sizeof(char));
    fread(word, sizeof(char), end - start - 1, file);

    enum token_kind kind = get_system_word(word); 
    if(kind == NAME)
        return (struct token){.kind = NAME, .val.string = word};
    else {
        free(word);
        return KW_TOK(kind);
    }
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

const char* skip_conditional(FILE* file, unsigned* line, const char* tag) {
    char c;
    struct token tok;
    while((c = fgetc(file)) != EOF) {
        if(c == '$' && fgetc(file) == '>') {
            if(!is_word_char(fgetc(file)))
                return "expect identifier after `$>`";
            tok = read_alpha_seq(file);
            if(strcmp(tag, tok.val.string) == 0) {
                free((void*) tok.val.string);
                break;
            }
            free((void*) tok.val.string);
        }
        else if(c == '\n')
            (*line)++;
    }

    return NULL;
}

struct token next_token(FILE* file, unsigned* line, struct token* prev, struct string_list** tags) {
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
        // compile-time conditionals:
#define GET_TAG(sym) if(!is_word_char(fgetc(file)))                           \
                        return ERR_TOK("expect idetntifier after `" sym "`"); \
                    tok = read_alpha_seq(file)
        case '$':
            GET_TAG("$$");
            tok = read_alpha_seq(file);
            if(string_list_contains(*tags, tok.val.string))
            {
                const char* removed = string_list_remove(*tags, tok.val.string);
                free((void*) tok.val.string);
                free((void*) removed);
            }
            else
                string_list_add(tags, tok.val.string);
            goto repeat;
        case '<': {
            GET_TAG("<");
            if(string_list_contains(*tags, tok.val.string)) {
                free((void*) tok.val.string);
                goto repeat;
            }
            
            const char* err;
            if((err = skip_conditional(file, line, tok.val.string)))
                return ERR_TOK(err);

            free((void*) tok.val.string);
            goto repeat;
        }
        case '~':
            GET_TAG("$~");
            if(!string_list_contains(*tags, tok.val.string)) {
                free((void*) tok.val.string);
                goto repeat;
            }
            
            const char* err;
            if((err = skip_conditional(file, line, tok.val.string)))
                return ERR_TOK(err);

           printf("$~ not yet implemented.");
            free((void*) tok.val.string);
            goto repeat;
        case '>':
            if(!is_word_char(fgetc(file))) {
                tok = ERR_TOK("expect identifier after `$>`");
                break;
            }
            while(is_word_char(c = fgetc(file)));
            ungetc(c, file);
            goto repeat;
#undef GET_TAG
        default:
            tok = ERR_TOK("unexpected character after `$`; expect `(` or `)`");
            break;
        }
        break;
    case '"':
    case '\'':
        tok = read_str_constant(file, c);
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
        else if(c == '*') {
            while((c = fgetc(file)) != EOF)
                if(c == '*' && fgetc(file) == '/')
                    goto repeat;
                else if(c == '\n')
                    (*line)++;
            tok = ERR_TOK("unclosed multiline comment");
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
    case '?':
        tok = KW_TOK(QUESTIONMARK);
        break;
    case '#':
        switch(c = fgetc(file)) {
        case 'B':
        case 'b':
            tok = read_num_constant(file, BINARY);
            break;
        case 'O':
        case 'o':
            tok = read_num_constant(file, OCTAL);
            break;
        case 'X':
        case 'x':
            tok = read_num_constant(file, HEXADECIMAL);
            break;
        default:
            ungetc(c, file);
            tok = read_num_constant(file, OCTAL);
        }
        break;
    default:
        if(isdigit(c)) {
            ungetc(c, file);
            tok = read_num_constant(file, DECIMAL); 
        } 
        else if(isalpha(c))
            tok = read_alpha_seq(file);
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
    "MATCH",
    "EVERY",
    "CASE",
    "DEFAULT",
    "LET",
    "MANIFEST",
    "GLOBAL",
    "STATIC",
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
    "SLCT",
    "BITSPERBCPLWORD",
    "QUESTIONMARK",
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

