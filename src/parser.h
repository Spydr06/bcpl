#ifndef BCPLC_PARSER_H
#define BCPLC_PARSER_H

#include <stdio.h>

#include "context.h"
#include "token.h"

void parse_file(struct context *ctx, struct source_file* file);

#endif /* BCPLC_PARSER_H */

