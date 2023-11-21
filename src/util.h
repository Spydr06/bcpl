#ifndef BCPLC_UTIL_H
#define BCPLC_UTIL_H

//
// Numeric functions
//

#define MIN(a, b) ((a) < (b) ? (a) : (b))
#define MAX(a, b) ((a) > (b) ? (a) : (b))

//
// String functions
//

void strshl(char* str, unsigned amount);

//
// String list (dynamic array)
//

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

//
// Generic pointer list (dynamic array)
//

#define PTR_LIST_INIT_SIZE 32

struct ptr_list {
    unsigned size;
    unsigned alloc;
    const void* data[];
};

struct ptr_list* ptr_list_init(void);
void ptr_list_add(struct ptr_list** list, const void* data);

#endif /* BCPLC_UTIL_H */

