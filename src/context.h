#ifndef BCPLC_CONTEXT_H
#define BCPLC_CONTEXT_H

struct context {
    const char* progname; // compiler program name
    const char* output_file; // output path
    const char* cur_filename;
};

#endif /* BCPLC_CONTEXT_H */

