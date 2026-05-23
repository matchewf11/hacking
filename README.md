# wurl

**A declarative HTTP client and test runner for the command line.**

`wurl` does two things well: it sends HTTP requests with a clean, ergonomic CLI, and it runs structured test suites that assert on every part of a response — status codes, JSON fields, headers, cookies, and timing. The same flags and value syntax work on both sides, so there is nothing new to learn when moving from ad-hoc exploration to a repeatable test file.

---

## Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Reference](#cli-reference)
  - [GET](#get)
  - [POST](#post)
  - [JSON Body Syntax](#json-body-syntax)
- [Test Suite Format](#test-suite-format)
  - [Structure](#structure)
  - [Request Line](#request-line)
  - [Assertions](#assertions)
  - [Assert Targets](#assert-targets)
  - [Matchers](#matchers)
  - [Negation](#negation)
- [Preprocessor](#preprocessor)
  - [include](#include)
  - [replace](#replace)
- [Environment Variables](#environment-variables)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **Ergonomic CLI** — `wurl get`, `wurl post` with consistent short flags; responses are pretty-printed and syntax-highlighted JSON.
- **Declarative test suites** — `.wurl` files describe HTTP requests and assertions in plain text; no code required.
- **Parallel group execution** — test groups run concurrently; tests within a group run sequentially, so order-dependent flows work correctly.
- **Rich assertions** — assert on status, JSON body fields (with dot-notation and array indexing), headers, cookies and their attributes, response duration, and raw body content.
- **Full matcher vocabulary** — equality, comparison, substring, prefix, suffix, length, regex, presence, and absence — all negatable with `not`.
- **Shared flag syntax** — `--json`, `--query`, `--headers`, `--cookies` work identically in the CLI and in test request lines.
- **Preprocessor** — `include` splits large suites into reusable files; `replace` injects literal values or environment variables.
- **Auto-typed JSON** — numbers, booleans, `null`, and arrays are detected automatically; dot-notation builds nested objects.

---

## Installation

**Prerequisites:** Rust toolchain ≥ 1.85 (edition 2024).

```sh
git clone https://github.com/yourname/wurl
cd wurl
cargo install --path .
```

Verify:

```sh
wurl --help
```

---

## Quick Start

```sh
# Send a GET request
wurl get "https://api.example.com/users"

# Send a POST request with a JSON body
wurl post "https://api.example.com/users" \
    --json name=Alice \
    --json role=admin \
    --json active=true \
    --json age=30

# Run a test suite
wurl test suite.wurl

# Pipe a suite from stdin
cat suite.wurl | wurl test -

# Print the built-in cookbook
wurl cook
```

---

## CLI Reference

### GET

```
wurl get <url> [options]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--path <path>` | `-p` | Path to append to the URL |
| `--query <key=value>` | `-q` | Query string parameter (repeatable) |
| `--headers <key:value>` | | Request header (repeatable) |
| `--cookies <key=value>` | `-c` | Cookie to send (repeatable) |

```sh
wurl get "https://api.example.com/search" \
    -q q=alice \
    -q page=1 \
    --headers "Authorization:Bearer token123" \
    -c "session=abc"
```

### POST

```
wurl post <url> [options]
```

Accepts all flags from GET, plus:

| Flag | Short | Description |
|------|-------|-------------|
| `--json <key=value>` | `-j` | JSON body field (repeatable) |

```sh
wurl post "https://api.example.com/posts" \
    --json title="Hello World" \
    --json published=false \
    --json tags=[rust,cli,http] \
    --headers "Authorization:Bearer token123"
```

### JSON Body Syntax

`--json` values are **auto-typed**:

| Input | JSON type |
|-------|-----------|
| `count=42` | `number` |
| `active=true` / `active=false` | `boolean` |
| `ref=null` | `null` |
| `ids=[1,2,3]` | `array` |
| `name=Alice` | `string` |

**Dot notation** builds nested objects:

```sh
wurl post "https://api.example.com/users" \
    --json user.name=Alice \
    --json user.address.city=Portland \
    --json settings.theme=dark
```

Produces:

```json
{
  "user": {
    "name": "Alice",
    "address": { "city": "Portland" }
  },
  "settings": { "theme": "dark" }
}
```

---

## Test Suite Format

Test suites are plain-text `.wurl` files. Run them with `wurl test <file>`.

### Structure

```
group "<group name>"
    test "<test name>"
        <request line>
        assert <target> [matcher]
        ...
    end

    test "<test name>"
        ...
    end
end
```

- **Groups** run concurrently with each other.
- **Tests within a group** run sequentially, in declaration order.
- All assertion failures within a test are reported — execution does not stop at the first failure.

### Request Line

The request line inside a test uses the same method and flags as the CLI:

```
<method> "<url>" [--path <p>] [--query <k=v>]... [--json <k=v>]... [--headers <k:v>]... [--cookies <k=v>]...
```

Examples:

```
get "https://api.example.com/users"
post "https://api.example.com/users" --json name=Alice --json role=admin
get "/users" --query page=1 --query per_page=25
```

Relative URLs are prefixed with `WURL_BASE_URL` (see [Environment Variables](#environment-variables)).

### Assertions

```
assert [not] <target> [matcher [value]]
```

### Assert Targets

| Target | Description |
|--------|-------------|
| `status` | HTTP response status code |
| `body` | Raw response body (string) |
| `body.<field>` | JSON field via dot notation (`body.user.name`) |
| `body.<field>[<n>]` | Array element (`body.items[0]`) |
| `body[<n>]` | Top-level array element (`body[0].id`) |
| `header.<name>` | Response header, case-insensitive (`header.content-type`) |
| `cookie.<name>` | Cookie value by name |
| `cookie.<name>.<attr>` | Cookie attribute (`cookie.session.httponly`) |
| `duration` | Response time in milliseconds |

### Matchers

| Matcher | Description |
|---------|-------------|
| *(none)* | Field or status exists and is truthy |
| `present` | Field or header is present |
| `absent` | Field or header is absent |
| `empty` | String or array is empty |
| `equals <value>` / `eq` | Strict equality |
| `not-equals <value>` / `neq` | Not equal |
| `contains <value>` | String contains substring, or array contains element |
| `not-contains <value>` | Does not contain |
| `starts-with <value>` | String starts with prefix |
| `ends-with <value>` | String ends with suffix |
| `gt <n>` | Greater than |
| `gte <n>` | Greater than or equal |
| `lt <n>` | Less than |
| `lte <n>` | Less than or equal |
| `length <n>` | Exact length |
| `length gt/gte/lt/lte <n>` | Length comparison |
| `matches r"<pattern>"` | Regex match |

**Values** in matchers can be:
- Quoted strings: `"hello world"`
- Bare single-word strings: `admin`
- Numbers: `200`, `3.14`
- Booleans: `true`, `false`
- Null: `null`
- Regex literals: `r"^pattern$"`

An unrecognised bare word is treated as an implicit `equals`:

```
assert body.role "admin"      # explicit quoted string
assert body.role admin        # same — bare ident is a string
assert body.count 0           # same — number
```

### Negation

Prefix any assertion with `not` to invert it:

```
assert not status 404
assert not body.error present
assert not header.x-deprecated present
assert not body.user.name equals "root"
```

### Full Example

```
group "auth"
    test "valid login returns a JWT"
        post "https://api.example.com/auth/login" \
            --json email=alice@example.com \
            --json password=hunter2
        assert status 200
        assert body.token present
        assert body.token matches r"^[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+$"
        assert cookie.session present
        assert cookie.session.httponly
        assert cookie.session.secure
        assert duration lt 800
    end

    test "wrong password is rejected"
        post "https://api.example.com/auth/login" \
            --json email=alice@example.com \
            --json password=wrong
        assert status 401
        assert body.error equals "invalid_credentials"
        assert not body.token present
        assert not cookie.session present
    end
end

group "posts"
    test "create a post"
        post "https://api.example.com/posts" \
            --json title="Hello Wurl" \
            --json published=false
        assert status 201
        assert body.id present
        assert header.location present
    end

    test "fetch post list"
        get "https://api.example.com/posts" --query page=1 --query per_page=10
        assert status 200
        assert body.data present
        assert body.data length gt 0
        assert body.data[0].id present
        assert header.content-type contains "json"
    end
end
```

---

## Preprocessor

Before parsing, `.wurl` files pass through a lightweight preprocessor that handles file inclusion and variable substitution.

### include

```
include "path/to/file.wurl"
```

Recursively inlines another `.wurl` file at the point of the directive. Paths are resolved relative to the including file.

```
include "env.wurl"
include "helpers/auth.wurl"

group "users"
    ...
end
```

### replace

```
replace VARIABLE_NAME [literal value]
```

Substitutes every occurrence of `VARIABLE_NAME` in the file with the given value. If no literal is provided, the value is read from the environment variable of the same name.

```
replace API_BASE https://api.example.com
replace SECRET_KEY                        # read from $SECRET_KEY env var

group "auth"
    test "login"
        post "API_BASE/auth/login" --json key=SECRET_KEY
        assert status 200
    end
end
```

`replace` directives are stripped from the output before parsing; they do not appear in the final test suite.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `WURL_BASE_URL` | Prefix prepended to relative URLs in test suites. If a test request URL does not start with `http://` or `https://`, this value is prepended. |

```sh
WURL_BASE_URL=https://staging.example.com wurl test suite.wurl
```

---

## Contributing

Contributions are welcome. Please open an issue before submitting a pull request for significant changes.

```sh
# Run tests
cargo test

# Build in release mode
cargo build --release

# Run locally without installing
cargo run -- get "https://httpbin.org/get"
cargo run -- test examples/smoke.wurl
```

Code style follows standard `rustfmt` defaults. Run `cargo fmt` and `cargo clippy` before opening a PR.

---

## License

MIT © Quienten Miller
