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

#define LOC_FROM_FILE(file, w) (         \
    (struct location) {                  \
        .file=(file),                    \
        .line=((uint32_t) (file)->line), \
        .width=(w),                      \
        .offset=ftell((file)->fd) - (w)  \
    })

#define ERR_TOK(tok, file, error) ( \
    init_token((tok), LEX_ERROR,    \
        LOC_FROM_FILE(file, 1),     \
        (union token_value) {       \
            .string=(error)         \
        }                           \
    ))

#define STR_TOK(tok, kind, file, str, loc) ( \
    init_token((tok), TOKEN_##kind,          \
        (loc),                               \
        (union token_value) {                \
            .string=str                      \
        }                                    \
    ))

#define BASIC_TOK(tok, file, kind) (               \
    init_token((tok), (kind),                      \
        LOC_FROM_FILE(file, token_widths[(kind)]), \
        (union token_value){.string=NULL}          \
    ))

#define KW_TOK(tok, file, kind) BASIC_TOK(tok, file, TOKEN_##kind)
#define EOF_TOK(tok, file) BASIC_TOK(tok, file, LEX_EOF)

static inline void init_token(struct token* tok, enum token_kind kind, struct location loc, union token_value val) {
    tok->kind = kind;
    tok->loc = loc;
    tok->val = val;
}

static const uint32_t token_widths[] = {
    [1] = 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1,
    1, 1, 1, 1,
    1, 2, 1, 2, 1, 2,
    1, 1, 1, 1, 2, 2,
    4, 5, 3, 3, 5, 8, 6, 6, 4, 6, 5, 2, 6, 5, 3, 5, 8, 5, 5, 4, 7, 2, 2, 2, 2, 2, 7, 7, 5, 8, 6, 3, 3,
    1,
    [0] = 0
};

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

void read_str_constant(struct source_file* file, struct token* tok, char quote) {
    char c;
    size_t start = ftell(file->fd); 

    while((c = fgetc(file->fd)) != quote) {
        if(c == EOF || c == '\n') {
            ERR_TOK(tok, file, "unexpected end of line; expect `'`");
            return;
        }
    }
    
    size_t end = ftell(file->fd);
    fseek(file->fd, start, SEEK_SET);

    char* strval = calloc(end - start, sizeof(char));
    fread(strval, sizeof(char), end - start - 1, file->fd);
    fseek(file->fd, end, SEEK_SET);

    const char* err = resolve_escape_codes(strval);
    if(err) {
        ERR_TOK(tok, file, err);
        return;
    }

    struct location loc = {
        .file = file,
        .line = file->line,
        .offset = start - 1,
        .width = end - start + 1
    };

    if(quote == '\'') {
        if(strlen(strval) > 1)
            ERR_TOK(tok, file, "char literal has more than one characters");
        STR_TOK(tok, CHAR, file, strval, loc);
        return;
    }
    STR_TOK(tok, STRING, file, strval, loc);
}

// TODO: fp support
void read_num_constant(struct source_file* file, struct token* tok, enum format_kind format) {
    char buf[65];
    unsigned i = 0;
    
    while((isalnum(buf[i] = fgetc(file->fd)) || buf[i] == '_') && buf[i] != EOF) {
        if(i > 64) {
            ERR_TOK(tok, file, "numeric constant too long");
            return;
        }
        if(strchr(digits[format], buf[i]) == NULL) {
            ERR_TOK(tok, file, "unexpected character in numeric constant");
            return;
        }
        if(buf[i] != '_')
            i++;
    }
    ungetc(buf[i], file->fd);
    buf[i] = '\0';

    uint64_t val = strtoull(buf, NULL, format);
    if(val == ULLONG_MAX && errno) {
        ERR_TOK(tok, file, "invalid numeric constant");
        return;
    }
    
    init_token(tok, TOKEN_INTEGER, LOC_FROM_FILE(file, strlen(buf)), (union token_value){.integer=val});
}

static const struct {
    const char* system_word;
    enum token_kind kind;
} system_words[] = {
    {"true", TOKEN_TRUE},
    {"false", TOKEN_FALSE},
    {"let", TOKEN_LET},
    {"and", TOKEN_AND},
    {"valof", TOKEN_VALOF},
    {"resultis", TOKEN_RESULTIS},
    {"return", TOKEN_RETURN},
    {"finish", TOKEN_FINISH},
    {"skip", TOKEN_SKIP},
    {"repeat", TOKEN_REPEAT},
    {"break", TOKEN_BREAK},
    {"if", TOKEN_IF},
    {"unless", TOKEN_UNLESS},
    {"until", TOKEN_UNTIL},
    {"switchon", TOKEN_SWITCHON},
    {"match", TOKEN_MATCH},
    {"every", TOKEN_EVERY},
    {"case", TOKEN_CASE},
    {"default", TOKEN_DEFAULT},
    {"do", TOKEN_DO},
    {"to", TOKEN_TO},
    {"by", TOKEN_BY},
    {"of", TOKEN_OF},
    {"be", TOKEN_BE},
    {"section", TOKEN_SECTION},
    {"require", TOKEN_REQUIRE},
    {"global", TOKEN_GLOBAL},
    {"manifest", TOKEN_MANIFEST},
    {"static", TOKEN_STATIC},
    {"mod", TOKEN_MOD},
    {"abs", TOKEN_ABS},
    {"for", TOKEN_FOR},
    {NULL, LEX_ERROR}
};

static enum token_kind get_system_word(const char* word) {
    for(unsigned i = 0; system_words[i].system_word; i++) {
        if(strcmp(system_words[i].system_word, word) == 0)
            return system_words[i].kind;
    }

    return TOKEN_IDENT;
}

static inline bool is_word_char(char c) {
    return isalnum(c) || c == '_';
}

static void read_alpha_seq(struct source_file* file, struct token* tok) {
    size_t start = ftell(file->fd) - 1;
    while(is_word_char(fgetc(file->fd)));
    size_t end = ftell(file->fd);
    fseek(file->fd, start, SEEK_SET);

    char* word = calloc(end - start, sizeof(char));
    fread(word, sizeof(char), end - start - 1, file->fd);

    enum token_kind kind = get_system_word(word); 
    if(kind == TOKEN_IDENT) {
        struct location loc = {
            .file = file,
            .line = file->line,
            .offset = start,
            .width = end - start - 1
        };

        STR_TOK(tok, IDENT, file, word, loc);
    }
    else {
        free(word);
        BASIC_TOK(tok, file, kind);
    }
}

bool ends_command(enum token_kind kind) {
    switch(kind) {
    case TOKEN_BREAK:
    case TOKEN_RETURN:
    case TOKEN_FINISH:
    case TOKEN_REPEAT:
    case TOKEN_RPAREN:
    case TOKEN_RBRACE:
    case TOKEN_RBRACKET:
    case TOKEN_IDENT:
    case TOKEN_INTEGER:
    case TOKEN_STRING:
    case TOKEN_TRUE:
    case TOKEN_FALSE:
    case TOKEN_FLOAT:
    case TOKEN_CHAR:
        return true;
    default:
        return false;
    }
}

bool may_start_command(enum token_kind kind) {
    switch(kind) {
    case TOKEN_FOR:
    case TOKEN_IF:
    case TOKEN_UNLESS:
    case TOKEN_UNTIL:
    case TOKEN_WHILE:
    case TOKEN_RESULTIS:
    case TOKEN_CASE:
    case TOKEN_DEFAULT:
    case TOKEN_BREAK:
    case TOKEN_RETURN:
    case TOKEN_FINISH:
    case TOKEN_LBRACE:
    case TOKEN_LPAREN:
    case TOKEN_VALOF:
    case TOKEN_IDENT:
        return true;
    default:
        return false;
    }
}

bool ends_expression(enum token_kind kind) {
    switch(kind) {
    case TOKEN_RPAREN:
    case TOKEN_RBRACE:
    case TOKEN_RBRACKET:
    case TOKEN_IDENT:
    case TOKEN_INTEGER:
    case TOKEN_FLOAT:
    case TOKEN_STRING:
    case TOKEN_TRUE:
    case TOKEN_FALSE:
        return true;
    default:
        return false;
    }
}

bool must_start_command(enum token_kind kind) {
    switch(kind) {
    case TOKEN_FOR:
    case TOKEN_IF:
    case TOKEN_UNLESS:
    case TOKEN_UNTIL:
    case TOKEN_WHILE:
    case TOKEN_RESULTIS:
    case TOKEN_CASE:
    case TOKEN_DEFAULT:
    case TOKEN_BREAK:
    case TOKEN_RETURN:
    case TOKEN_FINISH:
        return true;
    default:
        return false;
    }
}

const char* skip_conditional(struct source_file* file, const char* tag) {
    char c;
    struct token tok;
    while((c = fgetc(file->fd)) != EOF) {
        if(c == '$' && fgetc(file->fd) == '>') {
            if(!is_word_char(fgetc(file->fd)))
                return "expect identifier after `$>`";
            read_alpha_seq(file, &tok);
            if(strcmp(tag, tok.val.string) == 0) {
                free((void*) tok.val.string);
                break;
            }
            free((void*) tok.val.string);
        }
        else if(c == '\n')
            (file->line)++;
    }

    return NULL;
}

void next_token(struct source_file* file, struct token* tok, struct token* prev, struct string_list** tags) {
    char c;
    
repeat:
    ;

    bool newline = false;
    while(isspace(c = fgetc(file->fd))) {
        if(c == '\n') {
            (file->line)++;
            newline = true;
        }
    };

    size_t start = ftell(file->fd) - 1;
    
    switch(c) {
    case EOF:
        EOF_TOK(tok, file);
        break;
    case '(':
        KW_TOK(tok, file, LPAREN);
        break;
    case ')':
        KW_TOK(tok, file, RPAREN);
        break;
    case '[':
        KW_TOK(tok, file, LBRACKET);
        break;
    case ']':
        KW_TOK(tok, file, RBRACKET);
        break;
    case '{':
        KW_TOK(tok, file, LBRACE);
        break;
    case '}':
        KW_TOK(tok, file, RBRACE);
        break;
    case '$':
        // compile-time conditionals:
        switch(c = fgetc(file->fd)) {
#define GET_TAG(sym) if(!is_word_char(fgetc(file->fd))) {                           \
                        ERR_TOK(tok, file, "expect idetntifier after `" sym "`");   \
                        return;                                                     \
                    }                                                               \
                    read_alpha_seq(file, tok)
        case '$':
            GET_TAG("$$");
            if(string_list_contains(*tags, tok->val.string))
            {
                const char* removed = string_list_remove(*tags, tok->val.string);
                free((void*) tok->val.string);
                free((void*) removed);
            }
            else
                string_list_add(tags, tok->val.string);
            goto repeat;
        case '<': {
            GET_TAG("<");
            if(string_list_contains(*tags, tok->val.string)) {
                free((void*) tok->val.string);
                goto repeat;
            }
            
            const char* err;
            if((err = skip_conditional(file, tok->val.string))) {
                ERR_TOK(tok, file, err);
                return;
            }

            free((void*) tok->val.string);
            goto repeat;
        }
        case '~':
            GET_TAG("$~");
            if(!string_list_contains(*tags, tok->val.string)) {
                free((void*) tok->val.string);
                goto repeat;
            }
            
            const char* err;
            if((err = skip_conditional(file, tok->val.string))) {
                ERR_TOK(tok, file, err);
                return;
            }

            free((void*) tok->val.string);
            goto repeat;
        case '>':
            if(!is_word_char(fgetc(file->fd))) {
                ERR_TOK(tok, file, "expect identifier after `$>`");
                break;
            }
            while(is_word_char(c = fgetc(file->fd)));
            ungetc(c, file->fd);
            goto repeat;
#undef GET_TAG
        default:
            ERR_TOK(tok, file, "unexpected character after `$`; expect `$` ,`<`, `>` or `~` ");
            break;
        }
        break;
    case '"':
    case '\'':
        read_str_constant(file, tok, c);
        break;
    case '+':
        KW_TOK(tok, file, PLUS);
        break;
    case '-':
        if((c = fgetc(file->fd)) == '>')
            KW_TOK(tok, file, COND);
        else {
            ungetc(c, file->fd);
            KW_TOK(tok, file, MINUS);
        }
        break;
    case '*':
        KW_TOK(tok, file, STAR);
        break;
    case '/':
        if((c = fgetc(file->fd)) == '/') {
            while((c = fgetc(file->fd)) != '\n' && c != EOF);
            goto repeat;
        }
        else if(c == '*') {
            while((c = fgetc(file->fd)) != EOF)
                if(c == '*' && fgetc(file->fd) == '/')
                    goto repeat;
                else if(c == '\n')
                    (file->line)++;
            ERR_TOK(tok, file, "unclosed multiline comment");
        }
        else {
            ungetc(c, file->fd);
            KW_TOK(tok, file, SLASH);
        }
        break;
    case '=':
        KW_TOK(tok, file, EQ);
        break;
    case '!':
        KW_TOK(tok, file, EMARK);
        break;
    case ':':
        if((c = fgetc(file->fd)) == '=')
            KW_TOK(tok, file, ASSIGN);
        else if(c == ':')
            KW_TOK(tok, file, OF);
        else {
            ungetc(c, file->fd);
            KW_TOK(tok, file, COLON);
        }
        break;
    case ',':
        KW_TOK(tok, file, COMMA);
        break;
    case ';':
        KW_TOK(tok, file, SEMICOLON);
        break;
    case '<':
        if((c = fgetc(file->fd)) == '=')
            KW_TOK(tok, file, LE);
        else {
            ungetc(c, file->fd);
            KW_TOK(tok, file, LT);
        }
        break;
    case '>':
        if((c = fgetc(file->fd)) == '=')
            KW_TOK(tok, file, GE);
        else {
            ungetc(c, file->fd);
            KW_TOK(tok, file, GT);
        }
        break;
    case '~':
        if((c = fgetc(file->fd)) == '=')
            KW_TOK(tok, file, NE);
        else {
            ungetc(c, file->fd);
            KW_TOK(tok, file, NOT);
        }
        break;
    case '?':
        KW_TOK(tok, file, QMARK);
        break;
    case '@':
        KW_TOK(tok, file, AT);
        break;
    case '#':
        switch(c = fgetc(file->fd)) {
        case 'B':
        case 'b':
            read_num_constant(file, tok, BINARY);
            break;
        case 'O':
        case 'o':
            read_num_constant(file, tok, OCTAL);
            break;
        case 'X':
        case 'x':
            read_num_constant(file, tok, HEXADECIMAL);
            break;
        default:
            ungetc(c, file->fd);
            read_num_constant(file, tok, OCTAL);
        }
        break;
    default:
        if(isdigit(c)) {
            ungetc(c, file->fd);
            read_num_constant(file, tok, DECIMAL); 
        } 
        else if(isalpha(c))
            read_alpha_seq(file, tok);
        else
            ERR_TOK(tok, file, "unexpected character");
        break;
    }

    if(newline && ends_command(prev->kind) && may_start_command(tok->kind)) {
        fseek(file->fd, start, SEEK_SET);
        KW_TOK(tok, file, SEMICOLON);
    }

    else if(!newline && ends_expression(prev->kind) && must_start_command(tok->kind)) {
        fseek(file->fd, start, SEEK_SET);
        KW_TOK(tok, file, DO);
    }
}

#undef KW_TOK
#undef ERR_TOK

