" autocmd BufRead, BufNewFile *.bpp set filetype=bcplpp
" autocmd Syntax bcplpp runtime! bcplpp.vim


if exists("b:current_syntax")
    finish
endif
let b:current_syntax = "bcplpp"

"
" Syntax
"

"" True, False

syn keyword bcplppLiteralKeyword true false
hi def link bcplppLiteralKeyword Boolean

" Keywords

syn keyword bcplppConditionalKeyword resultis return if unless switchon match every
hi def link bcplppConditionalKeyword Conditional

syn keyword bcplppLoopKeyword repeat while until for
hi def link bcplppLoopKeyword Repeat

syn keyword bcplppLabelKeyword case default break finish skip
hi def link bcplppLabelKeyword Label

syn keyword bcplppOperatorKeyword valof do mod abs be of by to
hi def link bcplppOperatorKeyword Operator

syn keyword bcplppPreProcessorKeyword section require
hi def link bcplppPreProcessorKeyword Statement

syn keyword bcplppPrimitiveTypes Int8 Int16 Int Int64 UInt8 Uint16 Uint Uint64 Char Bool
hi def link bcplppPrimitiveTypes Type

syn keyword bcplppStorageClass global manifest static let and
hi def link bcplppStorageClass StorageClass

" Preprocessor

syn match bcplppPreCondit "\$[~<>][a-zA-Z0-9_]*"
hi def link bcplppPreCondit PreCondit

syn match bcplppDefine "\$\$[a-zA-Z0-9_]*"
hi def link bcplppDefine Define

" Symbols

syn match bcplppParens "[\[\](){}]"
hi def link bcplppParens Delimiter

syn match bcplppDelims "[.,;]"
hi def link bcplppDelims Delimiter

syn match bcplppOperators "[:=\->?!@+*/~<&|^]"
hi def link bcplppOperators Operator

" Numbers

syn match bcplppNumber "[0-9][0-9_]*"
syn match bcplppNumber "#[Bb][01]*"
syn match bcplppNumber "#[Oo]?[0-7]*"
syn match bcplppNumber "#[Hh][0-9a-fA-F]*"
hi def link bcplppNumber Number

syn match bcplppFloat "\<\d*\.\d\+\([eE][+-]\=\d+\)\=\>"
hi def link bcplppFloat Float

" Strings

syn match bcplppStringEscape "\*[ncpsbte\"'*]" contained
syn match bcplppStringEscape "\*\d\d\d" contained
syn match bcplppStringEscape "\*[hH][0-9a-fA-F][0-9a-fA-F]" contained 
syn match bcplppStringEscape "\*#[gu]" contained
syn match bcplppStringEscape "\*#[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]" contained
hi def link bcplppStringEscape SpecialChar

syn region bcplppString start=+"+ end=+"+ skip=+\\"+ contains=bcplppStringEscape
hi def link bcplppString String

syn region bcplppChar start="'" end="'" skip="\\'" contains=bcplppStringEscape
hi def link bcplppChar Character

" Identifiers

syn match bcplppFunctionParameter "[a-zA-Z_][a-zA-Z0-9_]*"
hi def link bcplppFunctionParameter Normal 

syn match bcplppFnIdent "[a-zA-Z_][a-zA-Z0-9_]*\s*\ze("
hi def link bcplppFnIdent Function

" Comments

syntax match bcplppComment "//.*$" contains=bcplppComment
syntax region bcplppComment start="/\*" end="\*/" contains=bcplppComment
hi def link bcplppComment Comment

