#ifndef BCPLC_CONTEXT_H
#define BCPLC_CONTEXT_H

#include "util.h"

struct context {
    const char* progname; // compiler program name
    const char* output_file; // output path
    const char* cur_filename;

    struct string_list* tags;
};

#endif /* BCPLC_CONTEXT_H */

