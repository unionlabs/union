//go:build tools

package testing

// Trick go mod into requiring protoc-gen-go-cast and therefore Gazelle won't prune it.
import (
	_ "github.com/prysmaticlabs/protoc-gen-go-cast"
)
