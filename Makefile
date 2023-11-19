override MAKEFLAGS := -rR

BUILDDIR := build

CFLAGS = -std=c99 -Wall -Wextra -pedantic -fPIC -g
CFLAGS_LIBB = -nostdlib -c 					\
	-Wno-incompatible-library-redeclaration \
	-Wno-builtin-declaration-mismatch       \
	-ffreestanding

COMPILER_SOURCES = $(shell find src -name '*.c')
COMPILER_OBJECTS = $(patsubst %, $(BUILDDIR)/%.o, $(COMPILER_SOURCES))

COMPILER_EXEC = bcpl++

override DEFAULT_CFLAGS := -g -pipe -c
override CFLAGS += $(DEFAULT_CFLAGS) \
	-Wall -Wextra -pedantic			 \
	-std=c99						 \
	-fPIE

override LD = gcc
CC := gcc

.PHONY: all
all: $(BUILDDIR)/$(COMPILER_EXEC)

.PHONY: install
install: all
	install -m 557 $(COMPILER_EXEC) $(BUILDDIR)/$(COMPILER_EXEC)

$(BUILDDIR)/$(COMPILER_EXEC): $(COMPILER_OBJECTS)
	$(LD) $(LDFLAGS) $^ -o $@

$(BUILDDIR)/%.c.o: %.c | $(BUILDDIR)
	mkdir -p $(@D)
	$(CC) $(CFLAGS) -MMD -MP -MF "$(@:%.c.o=%.c.d)" $< -o $@

-include $(COMPILER_OBJECTS:.o=.d)

$(BUILDDIR):
	mkdir $@

compile_commands.json:
	bear -- $(MAKE)

.PHONY: clean
clean:
	rm -rf $(BUILDDIR)

