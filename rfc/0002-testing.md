The idea to have multithreaded testing.

# Requirements

- tests run in parallel
- be able to group tests to run sequentailly
- assert output (text and code)

# Syntax

Before evaluating we need to:

1. do replaces to local files
    - if no value is given, then it pulls from the env
2. textually include other files

```
include "foo.wurl"
replace FOO
replace BAR 3

group auth
    test returns authenticated user
        get /foo
        assert status 200
        assert body.foo
    end
    test returns expected foo value
        get /foo
        assert status 200
        assert body.foo bar
    end
end

group foo
    test returns authenticated user
        get /foo
        assert status 200
        assert body.foo
    end
    test returns expected foo value
        get /foo
        assert status 200
        assert body.foo bar
    end

    test returns authenticated user
        get "/foo"
        assert status 200
        assert body.foo
    end

end
```

allow for env expansion for this

allow for cookies
