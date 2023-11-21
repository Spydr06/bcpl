#ifndef BCPLC_CONTEXT_H
#define BCPLC_CONTEXT_H

#include "ast.h"
#include "util.h"
#include <stdio.h>

enum build_kind {
    BUILD_EXEC,
    BUILD_SHARED_OBJECT,
    BUILD_OBJECT
};

struct context {
    const char* progname; // compiler program name
    const char* output_file; // output path
    const char* cur_filename;

    struct string_list* tags;

    enum build_kind build_kind;

    struct ast_program ast;
};

#endif /* BCPLC_CONTEXT_H */

