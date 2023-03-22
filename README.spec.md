---
mddux:
    runners:
        python:
            command: ["python3"]
---

# mddux

a CLI tool for executing Markdown documents with command and control

## Examples

Run: `mddux run -O README.spec.md`

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
