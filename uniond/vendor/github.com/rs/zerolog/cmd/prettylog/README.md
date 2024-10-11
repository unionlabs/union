# Zerolog PrettyLog

This is a basic CLI utility that will colorize and pretty print your structured JSON logs.

## Usage

You can compile it or run it directly. The only issue is that by default Zerolog does not output to `stdout`
but rather to `stderr` so we must pipe `stderr` stream to this CLI tool.

### Linux

These commands will redirect `stderr` to our `prettylog` tool and `stdout` will remain unaffected.

1. Compiled version

```shell
some_program_with_zerolog 2> >(prettylog)
```

2. Run it directly with `go run`

```shell
some_program_with_zerolog 2> >(go run cmd/prettylog/prettylog.go)
```

### Windows

These commands will redirect `stderr` to `stdout` and then pipe it to our `prettylog` tool.

1. Compiled version

```shell
some_program_with_zerolog 2>&1 | prettylog
```

2. Run it directly with `go run`

```shell
some_program_with_zerolog 2>&1 | go run cmd/prettylog/prettylog.go
```
