#include "context.h"
#include "util.h"

#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>

void context_init(struct context *ctx) {
    memset(ctx, 0, sizeof(struct context));

    ctx->output_file = DEFAULT_OUTPUT_FILE;
    ctx->tags = string_list_init();
    ctx->build_kind = BUILD_EXEC;
}

bool error_severity_exit(enum error_severity severity) {
    return severity == ERROR_FATAL;
}

static const char* error_severity_strs[] = {
    [ERROR_DEFAULT] = "\033[31merror:\033[0m",
    [ERROR_FATAL]   = "\033[31merror:\033[0m",
    [ERROR_INFO]    = "\033[36minfo:\033[0m",
    [ERROR_WARNING] = "\033[33mwarn:\033[0m",
};

void print_compiler_error(const struct context* ctx, enum error_severity severity, const struct location* loc, const char* error, ...) {
    size_t fd_pos = ftell(loc->file->fd);
    size_t line_start = loc->offset + 1;

    while(line_start > 0) {
        fseek(loc->file->fd, --line_start - 1, SEEK_SET);
        char c;
        if((c = fgetc(loc->file->fd)) == '\n')
            break;
    }

    size_t line_end = loc->offset + loc->width;
    fseek(loc->file->fd, line_end, SEEK_SET);
    char c;
    while((c = fgetc(loc->file->fd)) != '\n' && c != EOF)
        line_end++;

    fseek(loc->file->fd, line_start, SEEK_SET);
    char* line_str = calloc(line_end - line_start, sizeof(char));
    fread(line_str, line_end - line_start, sizeof(char), loc->file->fd);

    size_t column = loc->offset - line_start;

    fprintf(stderr, "\033[1m%s:%u:%zu: %s ", loc->file->path, loc->line, column, error_severity_strs[severity]);

    va_list ap;
    va_start(ap, error); 
    vfprintf(stderr, error, ap);
    va_end(ap);

    fprintf(stderr, "\n\033[1m\033[90m %4d \033[22m|\033[0m ", loc->line);
    fwrite(line_str, sizeof(char), column, stderr);
    fprintf(stderr, "\033[33m\033[1m");
    fwrite(line_str + column, sizeof(char), loc->width, stderr);
    fprintf(stderr, "\033[0m%s\n", line_str + column + loc->width);

    fprintf(stderr, "\033[90m      |\033[0m %*s\033[33m", (int) column, "");
    for(size_t i = 0; i < MAX(loc->width, 1); i++)
        fputc('^', stderr);
    fprintf(stderr, "\033[90m <- here\033[0m\n"); 

    fseek(loc->file->fd, fd_pos, SEEK_SET);

    if(error_severity_exit(severity)) {
        fputs("compilation terminated.\n", stderr);
        exit(EXIT_FAILURE);
    }
    
    fputc('\n', stderr);
}

