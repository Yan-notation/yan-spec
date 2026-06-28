" Vim syntax file
" Language: YAN (Yet Another Notation)
" Maintainer: yan-notation
" Latest Revision: 2026-06-28

if exists("b:current_syntax")
  finish
endif

syntax case match

" Comments
syntax match yanComment "#.*$"
syntax region yanBlockComment start="/\*" end="\*/"

" Strings
syntax region yanString start=/"/ skip=/\\"/ end=/"/
syntax region yanString start=/'/ skip=/\\'/ end=/'/

" Numbers
syntax match yanNumber "\v<-?\d+(\.\d+)?([eE][+-]?\d+)?>"

" Booleans
syntax keyword yanBoolean true false yes no on off

" Null
syntax keyword yanNull null nil Nil
" Standalone underscore as null alias
syntax match yanNull "\v\zs_\ze(\s|$|[};,])"

" Type hints
syntax match yanTypeHint "@\(int\|float\|bool\|date\|datetime\|hex\|base64\|uuid\|url\|regex\|bigint\|email\|ipv4\|ipv6\|color\|duration\)\>"

" Keys (word before a colon)
syntax match yanKey "\v^\s*[A-Za-z_][A-Za-z0-9_-]*\s*\ze:"
syntax match yanKey "\v(\{|;|,)\s*\zs[A-Za-z_][A-Za-z0-9_-]*\s*\ze:"

" Braces & punctuation
syntax match yanBraces "[{}]"
syntax match yanPunctuation "[;,:]"

highlight default link yanComment Comment
highlight default link yanBlockComment Comment
highlight default link yanString String
highlight default link yanNumber Number
highlight default link yanBoolean Boolean
highlight default link yanNull Constant
highlight default link yanTypeHint Type
highlight default link yanKey Identifier
highlight default link yanBraces Delimiter
highlight default link yanPunctuation Delimiter

let b:current_syntax = "yan"
