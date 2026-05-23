# Wurl — VSCode Syntax Highlighting

Syntax highlighting for [Wurl](../../README.md) test suite files.

## Features

- Full syntax highlighting for `.wurl` files
- Embedded highlighting in Markdown fenced code blocks tagged `wurl`
- Language configuration: auto-close quotes and brackets, line comments with `#`

## Wurl Language

Wurl is a declarative HTTP testing language. Tests are grouped and run concurrently; tests within a group run sequentially.

### Basic structure

```wurl
# This is a comment

group auth
  test "returns 200"
    GET /api/me
    assert status 200
    assert body.user present
    assert body.user.name equals "alice"
  end

  test "rejects unauthenticated"
    GET /api/me
    assert not status 200
    assert status 401
  end
end

group posts
  test "creates a post"
    POST /api/posts
    body.title = "Hello World"
    body.published := true
    assert status 201
    assert body.id present
    assert body.title equals "Hello World"
  end
end
```

### Assert targets

| Target                 | What it checks                            |
|------------------------|-------------------------------------------|
| `status`               | HTTP response status code                 |
| `body`                 | Raw response body string                  |
| `body.path[0].field`   | JSON path using dot + bracket notation    |
| `header.content-type`  | Response header value                     |
| `cookie.session`       | Cookie value                              |
| `cookie.session.httponly` | Cookie attribute flag                  |
| `duration`             | Request duration in milliseconds          |

### Matchers

`present` `absent` `empty` `equals` / `eq` `not-equals` / `neq`  
`contains` `not-contains` `starts-with` `ends-with`  
`gt` `gte` `lt` `lte` `length` `matches`

### Negation

```wurl
assert not status 404
assert not body.error present
```

## Installation

### From source (development)

1. Open `editors/vscode/` in VSCode
2. Press `F5` to launch the extension in a new Extension Development Host window

### Via `vsce` (publish / package)

```bash
cd editors/vscode
npm install -g @vscode/vsce
vsce package          # produces wurl-syntax-0.1.0.vsix
vsce publish          # publishes to the Marketplace
```

## Markdown support

Any fenced code block tagged `wurl` will be highlighted:

````markdown
```wurl
group example
  test ping
    GET /ping
    assert status 200
  end
end
```
````

## Scope reference

| Token                       | TextMate scope                            |
|-----------------------------|-------------------------------------------|
| `group` `test` `end` `assert` | `keyword.control.wurl`                 |
| `not`                       | `keyword.operator.logical.not.wurl`       |
| HTTP method                 | `support.function.http-method.wurl`       |
| Assert target base          | `support.type.property-name.*.wurl`       |
| Target path segments        | `variable.other.property.*.wurl`          |
| Matchers                    | `keyword.operator.matcher.wurl`           |
| `"strings"`                 | `string.quoted.double.wurl`               |
| `r"regex"`                  | `string.regexp.wurl`                      |
| Numbers                     | `constant.numeric.decimal.wurl`           |
| `true` `false` `null`       | `constant.language.wurl`                  |
| Comments                    | `comment.line.number-sign.wurl`           |
| Group names                 | `entity.name.section.group.wurl`          |
| Test names                  | `entity.name.function.test.wurl`          |
| Body param key              | `variable.parameter.body.wurl`            |
