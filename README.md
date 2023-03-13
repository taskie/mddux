# mddux

`mddux README.spec.md | tee README.md`

## Usage

In \[1\]:

``` sh
echo 'Hello world!'
```

Out \[1\]:

``` text:stdout
Hello world!
```

In \[2\]:

``` sh
echo 'Hello world!' >&2
```

Err \[2\]:

``` text:stderr
Hello world!
```

In \[3\]:

``` sh
mddux -h
```

Out \[3\]:

``` text:stdout
Usage: mddux [FILE]...

Arguments:
  [FILE]...  

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## License

MIT or Apache-2.0
