SHELL = /bin/sh

CFLAGS = -std=c99 -Wall -Wextra -pedantic -fPIC -g
CFLAGS_LIBB = -nostdlib -c 					\
	-Wno-incompatible-library-redeclaration \
	-Wno-builtin-declaration-mismatch       \
	-ffreestanding

COMPILER_FILES = $(shell find src -name '*.c')
COMPILER_EXEC = bcplc

BINDIR = ${SYSROOT}/bin

.PHONY: all
all: ${COMPILER_EXEC}

.PHONY: install
install: all
	install -m 557 ${COMPILER_EXEC} ${BINDIR}/${COMPILER_EXEC}

${COMPILER_EXEC}:
	${CC} ${CFLAGS} ${COMPILER_FILES} -o $@

compile_commands.json:
	bear -- ${MAKE}

.PHONY: clean
clean:
	rm -rf *.o *.a *.out ${COMPILER_EXEC}

