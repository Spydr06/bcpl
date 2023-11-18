#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdarg.h>
#include <string.h>

#include <getopt.h>

#include "context.h"
#include "token.h"
#include "util.h"

#define DEFAULT_OUTPUT_FILE "a.out"

static const struct option cmdline_options[] = {
    {"help", 0, NULL, 'h'},
    {NULL, 0, NULL, 0}
};

static void help(const char* progname) 
{
    printf("Usage: %s <input file> [OPTIONS]\n\n", progname);
    puts("Options:");
    printf("  -o <output file>  Set an output file; default: `%s`\n", DEFAULT_OUTPUT_FILE);
    printf("  -D <tag name>     Set a BCPL tag\n");
    printf("  -h, --help        Print this help text and exit.\n");
    exit(EXIT_SUCCESS); 
}

static void fatal_error(const char* progname, const char* error, ...)
{
    va_list ap;
    va_start(ap, error);
    
    fprintf(stderr, "\033[1m%s: \033[31mfatal error:\033[0m ", progname);
    vfprintf(stderr, error, ap);
    fprintf(stderr, "\ncompilation terminated.\n");

    va_end(ap);
    exit(EXIT_FAILURE);
}

static const char* get_fileext(const char* filename)
{
    const char* dot = strrchr(filename, '.');
    return dot && dot != filename && dot[1] != '\0' ? dot + 1 : NULL;
}

int main(int argc, char** argv) {
    struct context ctx;
    ctx.output_file = DEFAULT_OUTPUT_FILE;
    ctx.progname = argv[0];
    ctx.tags = string_list_init();

    int ch;
    while((ch = getopt_long(argc, argv, "ho:D:", cmdline_options, NULL)) != EOF)
    {
        switch(ch) {
        case 'h':
            help(argv[0]);
            break;
        case 'o':
            ctx.output_file = optarg;
            break;
        case 'D':
            string_list_add(&ctx.tags, optarg);
            break;
        case '?':
            fprintf(stderr, "Try `%s --help` for more information.\n", argv[0]);
            break;
        default:
            fprintf(stderr, "%s: invalid option -- %c\n", argv[0], ch);
            fprintf(stderr, "Try `%s --help` for more information.\n", argv[0]);
        }
    }

    if(optind >= argc)
        fatal_error(argv[0], "no input files");

    for(; optind < argc; optind++) {
        const char* input_file = argv[optind];
        const char* fileext = get_fileext(input_file); 
        if(!fileext)
            fatal_error(argv[0], "`%s`: unknown file format", input_file);

        if(strcmp(fileext, "b") == 0)
        {
            ctx.cur_filename = input_file;

            FILE* fd = fopen(input_file, "r");
            if(!fd)
                fatal_error(argv[0], "cannot find `%s`: %s", input_file, strerror(errno));

            struct token cur_tok;
            unsigned line = 1;
            do {
                cur_tok = next_token(fd, &line, &cur_tok, &ctx.tags);
                if(cur_tok.kind == LEX_ERROR)
                    lex_error(input_file, fd, line, cur_tok.val.string ? cur_tok.val.string : "");
                dbg_print_token(&cur_tok);
            } while(cur_tok.kind != LEX_EOF);
            
            fclose(fd);
        }
        else
            fatal_error(argv[0], "`%s`: unrecognized file extension `%s`", input_file, fileext);
    }

    printf("%u\n", ctx.tags->size);
 
    return 0;
}

