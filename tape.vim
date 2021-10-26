" Vim syntax file
" Language: Tape
" Maintainer: Gabriel Pacheco
" Latest Revision: 25 October 2021

if exists("b:current_syntax")
  finish
endif

syn match identifier '\k\+'
syn match repeatStmt '[+]\d\+'
syn match number '\d\+'
syn keyword makeKeyword make nextgroup=identifier skipwhite
syn keyword tapeKeywords incr decr putch getch debug nextgroup=identifier skipwhite
syn keyword tapeKeywords loop nextgroup=loopBlock skipwhite
"syn region loopBlock start='(' end=')' fold transparent contains=keyword,identifier,repeatStmt
"syn region tapeAccess start='\[' end=']' fold transparent contains=identifier,number
syn region commentRegion start='#!' end='\n' fold

let b:current_syntax = "tape"

hi def link identifier    Identifier
hi def link repeatStmt    Constant
hi def link tapeKeywords  Statement
hi def link makeKeyword   Statement
hi def link commentRegion      PreProc
"hi def link loopBlock     PreProc
"hi def link tapeAccess    PreProc

