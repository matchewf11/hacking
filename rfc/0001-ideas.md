
## Wurl Cli Tool

The idea is to have a nicer interface for
curl like requests.

Some Nice Builtins It will include:

- pretty printing (for json also html)
- get/post/update/... cmds
- testing suite (declaritive testing)
- json extracter
- take in stdin for chaining

``wurl
wurl get "https://foo.com/foo" --pretty-print
```

Need a nice way to hanlde paths, query args, and body

Paths
```
wurl get "https://foo.com" --path foo/bar
```

Query Args
```
wurl get "https://foo.com" --args foo=1,bar=2
wurl get "https://foo.com" -a foo=1,bar=2
```

Body
```
:= is values // auto converts
= is strings
foo.bar = foo // this is tables

wurl post "http://foo.com" --json foo.bar=jimmy --json foo:=true
```
