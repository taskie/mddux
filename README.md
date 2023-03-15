# mddux

executes a markdown document with excellent commands.

## Examples

Run: `mddux run README.spec.md | tee README.md`

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

``` python
import json
print(json.dumps({"a": "Hello, world!", "b": 42}, indent=4))
```

Out \[2\]:

``` json
{
    "a": "Hello, world!",
    "b": 42
}
```

### Usage

In \[3\]:

``` console
$ mddux -h
$ mddux run -h
$ mddux run-console -h
```

Out \[3\]:

``` console
$ mddux -h
Usage: mddux <COMMAND>

Commands:
  run          
  run-console  
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
$ mddux run -h
Usage: mddux run [OPTIONS] [FILE]...

Arguments:
  [FILE]...  

Options:
      --no-caption  
      --caption     
  -h, --help        Print help
$ mddux run-console -h
Usage: mddux run-console [OPTIONS] [FILE]

Arguments:
  [FILE]  

Options:
  -t, --timeout <TIMEOUT>  
  -h, --help               Print help
  -V, --version            Print version
```

## License

MIT or Apache-2.0
