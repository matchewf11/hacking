" Vim syntax file
" Language: Wurl
" Maintainer: Wurl
" Latest Revision: 2026-05-22

if exists("b:current_syntax")
  finish
endif

" Comments
syntax match wurlComment "#.*$"
highlight default link wurlComment Comment

" Keywords
syntax keyword wurlKeyword group test end include
highlight default link wurlKeyword Keyword

" HTTP Methods (case-insensitive)
syntax case ignore
syntax keyword wurlMethod GET POST PUT PATCH DELETE HEAD OPTIONS
syntax case match
highlight default link wurlMethod Function

" Assert Statement
syntax keyword wurlAssert assert
highlight default link wurlAssert Keyword

" Logical Operators
syntax keyword wurlOperator not
highlight default link wurlOperator Operator

" Matchers
syntax keyword wurlMatcher present absent empty equals eq not-equals neq contains not-contains starts-with ends-with gt gte lt lte length matches
highlight default link wurlMatcher Operator

" Target Types (status, duration, body, header, cookie)
syntax keyword wurlTarget status duration
syntax match wurlTarget "\<body\>\(\.[a-zA-Z0-9_\-]\+\|\(\[[0-9]\+\]\)\)*"
syntax match wurlTarget "\<header\>\(\.[a-zA-Z0-9_\-]\+\)*"
syntax match wurlTarget "\<cookie\>\(\.[a-zA-Z0-9_\-]\+\)*"
highlight default link wurlTarget Identifier

" Assignment
syntax match wurlAssignment "="
highlight default link wurlAssignment Operator

" Regex Literal (r"...")
syntax region wurlRegex start=/r"/ skip=/\\"/ end=/"/
highlight default link wurlRegex String

" Strings ("...")
syntax region wurlString start=/"/ skip=/\\"/ end=/"/
highlight default link wurlString String

" Numbers
syntax match wurlNumber "\<\-\?\d\+\(\.\d\+\)\?\>"
highlight default link wurlNumber Number

" Booleans
syntax keyword wurlBoolean true false
highlight default link wurlBoolean Boolean

" Null
syntax keyword wurlNull null
highlight default link wurlNull Constant

let b:current_syntax = "wurl"
