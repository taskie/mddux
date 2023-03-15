---
mddux:
    runners:
        python:
            command: ["python3"]
---

# mddux

executes a markdown document with excellent commands.

## Examples

Run: `mddux run README.spec.md | tee README.md`

```sh
echo 'Hello world!'
echo 'Goodbye world!' >&2
```

```python
# mddux-stdout-info: json
import json
print(json.dumps({"a": "Hello, world!", "b": 42}, indent=4))
```

### Usage

```console
$ mddux -h
$ mddux run -h
$ mddux run-console -h
```

## License

MIT or Apache-2.0
