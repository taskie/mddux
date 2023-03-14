---
mddux:
    runners:
        python:
            command: ["python3"]
---

# mddux

executes a markdown document with excellent commands.

## Examples

Run: `mddux README.spec.md | tee README.md`

```sh
echo 'Hello world!'
echo 'Goodbye world!' >&2
```

```sh
mddux -h
```

```python
# mddux-stdout-info: json
import json
print(json.dumps({"a": "Hello, world!", "b": 42}, indent=4))
```

## License

MIT or Apache-2.0
