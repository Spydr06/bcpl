#ifndef BCPLC_CONTEXT_H
#define BCPLC_CONTEXT_H

#include "ast.h"
#include "util.h"
#include <stdio.h>

#define DEFAULT_OUTPUT_FILE "a.out"

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

void context_init(struct context* ctx);

enum error_severity {
    ERROR_INFO,
    ERROR_WARNING,
    ERROR_DEFAULT,
    ERROR_FATAL,
};

bool error_severity_exit(enum error_severity severity);

void 
#ifdef __GLIBC__ 
    __attribute__((format(printf, 4, 5)))
#endif 
    print_compiler_error(const struct context* ctx, enum error_severity severity, const struct location* loc, const char* error, ...);

#endif /* BCPLC_CONTEXT_H */

