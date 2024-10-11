package main

import (
	"io"
	"os"

	"github.com/mattn/go-colorable"
)

func main() {
	io.Copy(colorable.NewColorableStdout(), os.Stdin)
}
