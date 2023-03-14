# mddux

executes a markdown document with excellent commands.

## Examples

Run: `mddux README.spec.md | tee README.md`

In \[1\]:

``` sh
echo 'Hello world!'
echo 'Goodbye world!' >&2
```

Out \[1\]:

``` text
Hello world!
```

Err \[1\]:

``` text
Goodbye world!
```

In \[2\]:

``` sh
mddux -h
```

Out \[2\]:

``` text
Usage: mddux [OPTIONS] [FILE]...

Arguments:
  [FILE]...  

Options:
      --no-caption  
      --caption     
  -h, --help        Print help
  -V, --version     Print version
```

In \[3\]:

``` python
import json
print(json.dumps({"a": "Hello, world!", "b": 42}, indent=4))
```

Out \[3\]:

``` json
{
    "a": "Hello, world!",
    "b": 42
}
```

## License

MIT or Apache-2.0
