# filepathx

> A small `filepath` extension library that supports double star globbling.

## Documentation

GoDoc: <https://pkg.go.dev/github.com/yargevad/filepathx>

## Install

```bash
go get github.com/yargevad/filepathx
```

## Usage Example

You can use `a/**/*.*` to match everything under the `a` directory
that contains a dot, like so:

```go
package main

import (
	"fmt"
	"os"

	"github.com/yargevad/filepathx"
)

func main() {
	if 2 != len(os.Args) {
		fmt.Println(len(os.Args), os.Args)
		fmt.Fprintf(os.Stderr, "Usage: go build example/find/*.go; ./find <pattern>\n")
		os.Exit(1)
		return
	}
	pattern := os.Args[1]

	matches, err := filepathx.Glob(pattern)
	if err != nil {
		panic(err)
	}

	for _, match := range matches {
		fmt.Printf("MATCH: [%v]\n", match)
	}
}
```

Given this directory structure:

```bash
find a
```

```txt
a
a/b
a/b/c.d
a/b/c.d/e.f
```

This will be the output:

```bash
go build example/find/*.go
./find 'a/**/*.*'
```

```txt
MATCH: [a/b/c.d]
MATCH: [a/b/c.d/e.f]
```
