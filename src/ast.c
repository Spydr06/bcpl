#include "ast.h"
#include "util.h"

#include <stdlib.h>
#include <string.h>

void ast_program_init(struct ast_program* program)
{
    memset(program, 0, sizeof(struct ast_program));

    program->sections = ptr_list_init();
    program->types = ptr_list_init();

    for(enum ast_type_kind i = BUILTIN_PRIMITIVE_TYPE_START; i <= BUILTIN_PRIMITIVE_TYPE_END; i++) {
        struct ast_builtin_type* builtin = malloc(sizeof(struct ast_builtin_type));
        builtin->kind = i;
        ptr_list_add(&program->types, (const void*) builtin);
    }
}

