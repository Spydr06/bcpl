#ifndef BCPLC_UTIL_H
#define BCPLC_UTIL_H

void strshl(char* str, unsigned amount);

#define STRING_LIST_INIT_SIZE 32

struct string_list {
    unsigned size;
    unsigned alloc;
    const char* strings[];
};

struct string_list* string_list_init(void);
void string_list_add(struct string_list** list, const char* string);
const char* string_list_remove(struct string_list* list, const char* string);

const char* string_list_contains(struct string_list* list, const char* string);

struct ptr_list {
    unsigned size;
    unsigned alloc;
    const void* data[];
};

#endif /* BCPLC_UTIL_H */

