# mddux

a CLI tool for executing Markdown documents with command and control

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
a CLI tool for executing Markdown documents with command and control

Usage: mddux <COMMAND>

Commands:
  run          Execute code blocks within a specified Markdown file
  run-console  Execute a console code block content
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
$ mddux run -h
Execute code blocks within a specified Markdown file

Usage: mddux run [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Markdown files to execute

Options:
  -a, --all            Execute all code blocks forcedly
  -s, --state <STATE>  A state file to store or load the execution state
      --no-caption     Disable displaying captions for code blocks
      --caption        Enable displaying captions for code blocks
  -h, --help           Print help
$ mddux run-console -h
Execute a console code block content

Usage: mddux run-console [OPTIONS] [FILE]

Arguments:
  [FILE]  A console code block content file to execute

Options:
  -t, --timeout <TIMEOUT>  A timeout for the execution
  -h, --help               Print help
```

## License

MIT or Apache-2.0
