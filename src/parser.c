#include "parser.h"

#include "ast.h"
#include "token.h"

void parse_file(struct context *ctx, const char* filename, FILE *fd)
{
    struct token cur_tok;
    unsigned line = 1;
    do {
        cur_tok = next_token(fd, &line, &cur_tok, &ctx->tags);
        if(cur_tok.kind == LEX_ERROR)
            lex_error(filename, fd, line, cur_tok.val.string ? cur_tok.val.string : "");
        dbg_print_token(&cur_tok);
    } while(cur_tok.kind != LEX_EOF);
}

