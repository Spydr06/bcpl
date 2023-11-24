#include "util.h"

#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <stdbool.h>

void strshl(char* str, unsigned amount)
{
    size_t len = strlen(str);
    for(size_t i = 0; i < len - amount + 1; i++)
        str[i] = str[i + amount];
}

struct string_list* string_list_init(void) {
    struct string_list* list = malloc(sizeof(struct string_list) + sizeof(char*) * STRING_LIST_INIT_SIZE);
    list->size = 0;
    list->alloc = STRING_LIST_INIT_SIZE;
    return list;
}

void string_list_add(struct string_list **list, const char *string) {
    if((*list)->size >= (*list)->alloc)
        *list = realloc(*list, sizeof(struct string_list) + sizeof(char*) * ((*list)->alloc <<= 1));
    (*list)->strings[(*list)->size++] = string;
}

const char* string_list_remove(struct string_list* list, const char* string) {
    bool found = false;
    const char* return_val = NULL;
    for(unsigned i = 0; i < list->size; i++) {
        if(!found && strcmp(string, list->strings[i]) == 0) {
            return_val = list->strings[i];
            found = true;
            list->size--;
        }
        if(found)
            list->strings[i] = list->strings[i + 1];
    }

    return return_val;
}

const char* string_list_contains(struct string_list* list, const char* string) {
    for(unsigned i = 0; i < list->size; i++) {
        if(strcmp(string, list->strings[i]) == 0)
            return list->strings[i];
    }
    return NULL;
}

struct ptr_list* ptr_list_init(void) {
    struct ptr_list* list = malloc(sizeof(struct ptr_list) + sizeof(void*) * PTR_LIST_INIT_SIZE);
    list->size = 0;
    list->alloc = PTR_LIST_INIT_SIZE;
    return list;
}

void ptr_list_add(struct ptr_list** list, const void* data) {
    if((*list)->size >= (*list)->alloc)
        *list = realloc(*list, sizeof(struct ptr_list) + sizeof(void*) + ((*list)->alloc <<= 1));
    (*list)->data[(*list)->size++] = data;
}

void ptr_list_pop(struct ptr_list *list) {
    if(list->size)
        list->size--;
}

