
## Hurl Cli Tool

The idea is to have a nicer interface for
curl like requests.

Some Nice Builtins It will include:

- pretty printing (for json also html)
- get/post/update/... cmds
- testing suite (declaritive testing)

```
hurl get "https://foo.com/foo" --pretty-print
```

Need a nice way to hanlde paths, query args, and body

Paths
```
hurl get "https://foo.com" --path foo/bar
```

Query Args
```
hurl get "https://foo.com" --args foo=1,bar=2
hurl get "https://foo.com" -a foo=1,bar=2
```

Body
```
:= is values
= is strings
foo.bar = foo // this is tables

hurl post "http://foo.com" --json foo.bar=jimmy --json foo:=true
```
