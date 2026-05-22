The idea to have multithreaded testing.

# Requirements

- tests run in parallel
- be able to group tests to run sequentailly
- assert output (text and code)

# Syntax

```
include "foo.hurl"

group "auth"
    test "returns authenticated user"
        get "/foo"
        assert status 200
        assert body.foo

    test "returns expected foo value"
        get "/foo"
        assert status 200
        assert body.foo "bar"

group "foo"
    test "returns authenticated user"
        get "/foo"
        assert status 200
        assert body.foo

    test "returns expected foo value"
        get "/foo"
        assert status 200
        assert body.foo "bar"
    test "returns authenticated user"
        get "/foo"
        assert status 200
        assert body.foo
```

allow for env expansion for this

allow for cookies
