package dot_imports

//go:generate mockgen -package dot_imports -destination mock.go -source input.go

import (
	"bytes"
	. "context"
	. "net/http"
)

type WithDotImports interface {
	Method1() Request
	Method2() *bytes.Buffer
	Method3() Context
}
