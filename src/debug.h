#ifndef BCPLPP_DEBUG_H
#define BCPLPP_DEBUG_H

#include "token.h"
#include "ast.h"

void dbg_print_token(struct token* t);

void dbg_print_ast_program(const struct ast_program* ast);

#endif /* BCPLPP_DEBUG_H */

