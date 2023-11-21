#include "parser.h"

#include "ast.h"
#include "token.h"

void parse_file(struct context *ctx, struct source_file* file)
{
    struct token cur_tok;
    struct token last_tok;
    unsigned line = 1;
    do {
        last_tok = cur_tok;
        next_token(file, &cur_tok, &last_tok, &ctx->tags);
        if(cur_tok.kind == LEX_ERROR)
            lex_error(file->path, file->fd, line, cur_tok.val.string ? cur_tok.val.string : "");
        dbg_print_token(&cur_tok);
        print_err_for(ctx, &cur_tok.loc, "test");
    } while(cur_tok.kind != LEX_EOF);

}

