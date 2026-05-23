
# RFC-003: Assertions in Wurl Test Suite

# Summary 

This RFC defines the assertion system for the Wurl test suite. Assertions are the mechanism by which tests validate HTTP responses. This document covers the motivation for assertions, the full assertion syntax, supported targets, comparison operators, and example usage.

# Motivation

When testing HTTP APIs, the most common need after making a request is to verify the response. This includes:

 - Status codes - Did the server respond as expected?
 - Response bodies - Is the data correct and present?
 - Headers - Are the right headers set (e.g., `Content-Type`, `Set-Cookie`)?
 - Cookies - Were cookies set or cleared correctly?
 - Response timing - Did the request complete within acceptable bounds?

Without a structured assertion system, users are forced to pipe output through jq, grep, awk, and shell conditionals - making tests fragile, hard to read, and difficult to parallelize. Wurl's assertion system is declarative, readable, and tightly integrated with the request lifecycle.

# Terminology

- Target - The thing being asserted on (e.g., `status`, `body.foo`, `header.content-type`)
- Matcher - The condition being checked (e.g., equals, exists, contains)
- Value - The expected value used in the comparison
- Assertion - A single assert `<target> [matcher] [value]` statement

# Assertion Syntax

The general form of an assertion is:

```wurl
assert <target> [<matcher> [<value>]]
```

1. If only `<target>` is provided, the assertion checks existence / truthiness.
2. If `<matcher>` is provided without a value, it applies unary matchers (e.g., `empty`, `present`).
3. If both `<matcher>` and `<value>` are provided, it performs a comparison.
4. Assertions must appear inside a `test` block.

# Assertion Targets

## Status Code

```wurl
assert status <code>
```

Asserts the HTTP response status code.

```wurl
assert status 200
assert status 404
assert status 201
```

## Body (Raw)

```wurl
assert body <matcher> [value]
```

Asserts on the raw response body as a string.

```wurl
assert body contains "success"
assert body equals "ok"
assert body empty
```

## Body (JSON Path)
text

```wurl
assert body.<path> [matcher] [value]
```

JSON paths use dot notation. Array indices are accessed with bracket notation.

### Existence check - asserts the key is present and non-null

```wurl
assert body.user
assert body.user.name
assert body.items[0]
```

### Equality

```wurl
assert body.user.name "alice"
assert body.status "active"
```

### Numeric

```wurl
assert body.count equals 3
assert body.score gt 90
```

### Boolean

```wurl
assert body.enabled true
assert body.deleted false
```

### Array length

```wurl
assert body.items length 5
assert body.tags length gt 0
```

### Headers

```wurl
assert header.<name> [matcher] [value]
```

Header names are case-insensitive.

```wurl
assert header.content-type
assert header.content-type equals "application/json"
assert header.content-type contains "json"
assert header.x-request-id present
assert header.x-deprecated absent
```

# Cookies

```wurl
assert cookie.<name> [matcher] [value]
assert cookie.<name>.<attribute>
```

Checks cookies set by the response via `Set-Cookie`.

## Existence

```wurl
assert cookie.session
assert cookie.session present
```

## Value

```wurl
assert cookie.session equals "abc123"
```

## Cookie attributes

```
assert cookie.session.httponly
assert cookie.session.secure
assert cookie.session.path equals "/"
assert cookie.session.samesite equals "Strict"
```

## Absence

```wurl
assert cookie.tracking absent
```

# Response Time

```wurl
assert duration <matcher> <value>
```

Value is in milliseconds.

```wurl
assert duration lt 500
assert duration lte 1000
```

# Matchers

Matcher	Alias	Description	Value Required
equals	eq	Exact equality	Yes
not-equals	neq	Not equal	Yes
contains		String/array contains value	Yes
not-contains		Does not contain	Yes
starts-with		String starts with	Yes
ends-with		String ends with	Yes
gt		Greater than	Yes
gte		Greater than or equal	Yes
lt		Less than	Yes
lte		Less than or equal	Yes
length		Length of string or array	Yes
present		Key/header/cookie exists	No
absent		Key/header/cookie does not exist	No
empty		Value is empty string or array	No
matches		Matches a regex pattern	Yes

When no matcher is provided, the assertion defaults to a truthiness check - the target must exist, be non-null, non-false, and non-empty.

# Regex Matching

```wurl
assert body.email matches r"^[\w.]+@[\w.]+\.[a-z]{2,}$"
assert header.content-type matches r"application/(json|xml)"
```

Regex values are prefixed with r and wrapped in double quotes.

# Negation

Any assertion can be negated with not:

```wurl
assert not status 404
assert not body.error
assert not header.x-deprecated present
assert not cookie.tracking present
assert not body.user.name equals "root"
```

# Environment Variable Expansion

Assertion values support environment variable expansion using ${} syntax:

```wurl
assert body.user.id equals ${EXPECTED_USER_ID}
assert body.org equals ${ORG_NAME}
assert cookie.session equals ${SESSION_TOKEN}
```

# Full Example

```wurl
include "env.wurl"

group "auth"
    test "login returns token"
        post "/auth/login"
            body.email = "alice@example.com"
            body.password = ${TEST_PASSWORD}
        assert status 200
        assert body.token present
        assert body.token matches r"^[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+$"
        assert header.content-type contains "json"
        assert cookie.session present
        assert cookie.session.httponly
        assert cookie.session.secure
        assert duration lt 800
    end

    test "rejects invalid credentials"
        post "/auth/login"
            body.email = "alice@example.com"
            body.password = "wrong"
        assert status 401
        assert body.error present
        assert body.error equals "invalid_credentials"
        assert not body.token
    end

    test "rejects missing fields"
        post "/auth/login"
            body.email = "alice@example.com"
        assert status 400
        assert body.errors present
        assert body.errors length gt 0
    end
end

group "users"
    test "returns current user"
        get "/users/me"
        assert status 200
        assert body.id present
        assert body.email equals ${TEST_EMAIL}
        assert body.role equals "member"
        assert not body.password
        assert not body.deleted
    end

    test "returns user list"
        get "/users"
        assert status 200
        assert body length gt 0
        assert body[0].id present
        assert body[0].email present
    end
end
```

# Failure Output

When an assertion fails, Wurl will output a clear diagnostic message:

```
FAIL  auth > rejects invalid credentials
  ✗  assert body.error equals "invalid_credentials"
       expected: "invalid_credentials"
            got: "unauthorized"
       at:       POST /auth/login  ->  401
```

All assertions in a test are evaluated and reported - Wurl does not short-circuit on the first failure, giving a complete picture of what went wrong.

# Why Not Just Use Shell Pipes?

Concern	Shell Pipes	Wurl Assertions
Readability	Fragile one-liners	Declarative, self-documenting
Parallel execution	Complex to orchestrate	Native, group-scoped
Error messages	Cryptic exit codes	Rich diagnostics
JSON access	Requires jq install	Built-in dot-path access
Cookie/header checks	Manual grep	First-class targets
Environment vars	Shell expansion only	Integrated ${} expansion

# Open Questions

1. Should length be a standalone target or a matcher?
    Current proposal: usable as both (assert body.items length 5 and assert body.items.length equals 5)

2. Should assertions support capturing values for reuse across tests within a group?
    e.g., capture token from body.token - proposed for a future RFC

3. Should regex flags (case-insensitive, multiline) be supported?
    e.g., matches r"(?i)^json" - open for discussion

# Conclusion

Assertions are the core of Wurl's value as a testing tool. They transform ad-hoc shell scripting into a structured, readable, and parallelizable test suite. This RFC establishes a consistent, extensible syntax that covers the most common HTTP response validation needs while remaining approachable for new users.