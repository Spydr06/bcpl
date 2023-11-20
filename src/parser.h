#ifndef BCPLC_PARSER_H
#define BCPLC_PARSER_H

#include <stdio.h>

#include "context.h"

void parse_file(struct context* ctx, const char* filename, FILE* fd);

#endif /* BCPLC_PARSER_H */

