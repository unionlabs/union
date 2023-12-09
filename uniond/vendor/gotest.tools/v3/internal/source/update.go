package source

import (
	"bytes"
	"errors"
	"flag"
	"fmt"
	"go/ast"
	"go/format"
	"go/parser"
	"go/token"
	"os"
	"runtime"
	"strings"
)

// IsUpdate is returns true if the -update flag is set. It indicates the user
// running the tests would like to update any golden values.
func IsUpdate() bool {
	if Update {
		return true
	}
	return flag.Lookup("update").Value.(flag.Getter).Get().(bool)
}

// Update is a shim for testing, and for compatibility with the old -update-golden
// flag.
var Update bool

func init() {
	if f := flag.Lookup("update"); f != nil {
		getter, ok := f.Value.(flag.Getter)
		msg := "some other package defined an incompatible -update flag, expected a flag.Bool"
		if !ok {
			panic(msg)
		}
		if _, ok := getter.Get().(bool); !ok {
			panic(msg)
		}
		return
	}
	flag.Bool("update", false, "update golden values")
}

// ErrNotFound indicates that UpdateExpectedValue failed to find the
// variable to update, likely because it is not a package level variable.
var ErrNotFound = fmt.Errorf("failed to find variable for update of golden value")

// UpdateExpectedValue looks for a package-level variable with a name that
// starts with expected in the arguments to the caller. If the variable is
// found, the value of the variable will be updated to value of the other
// argument to the caller.
func UpdateExpectedValue(stackIndex int, x, y interface{}) error {
	_, filename, line, ok := runtime.Caller(stackIndex + 1)
	if !ok {
		return errors.New("failed to get call stack")
	}
	debug("call stack position: %s:%d", filename, line)

	fileset := token.NewFileSet()
	astFile, err := parser.ParseFile(fileset, filename, nil, parser.AllErrors|parser.ParseComments)
	if err != nil {
		return fmt.Errorf("failed to parse source file %s: %w", filename, err)
	}

	expr, err := getCallExprArgs(fileset, astFile, line)
	if err != nil {
		return fmt.Errorf("call from %s:%d: %w", filename, line, err)
	}

	if len(expr) < 3 {
		debug("not enough arguments %d: %v",
			len(expr), debugFormatNode{Node: &ast.CallExpr{Args: expr}})
		return ErrNotFound
	}

	argIndex, ident := getIdentForExpectedValueArg(expr)
	if argIndex < 0 || ident == nil {
		debug("no arguments started with the word 'expected': %v",
			debugFormatNode{Node: &ast.CallExpr{Args: expr}})
		return ErrNotFound
	}

	value := x
	if argIndex == 1 {
		value = y
	}

	strValue, ok := value.(string)
	if !ok {
		debug("value must be type string, got %T", value)
		return ErrNotFound
	}
	return UpdateVariable(filename, fileset, astFile, ident, strValue)
}

// UpdateVariable writes to filename the contents of astFile with the value of
// the variable updated to value.
func UpdateVariable(
	filename string,
	fileset *token.FileSet,
	astFile *ast.File,
	ident *ast.Ident,
	value string,
) error {
	obj := ident.Obj
	if obj == nil {
		return ErrNotFound
	}
	if obj.Kind != ast.Con && obj.Kind != ast.Var {
		debug("can only update var and const, found %v", obj.Kind)
		return ErrNotFound
	}

	switch decl := obj.Decl.(type) {
	case *ast.ValueSpec:
		if len(decl.Names) != 1 {
			debug("more than one name in ast.ValueSpec")
			return ErrNotFound
		}

		decl.Values[0] = &ast.BasicLit{
			Kind:  token.STRING,
			Value: "`" + value + "`",
		}

	case *ast.AssignStmt:
		if len(decl.Lhs) != 1 {
			debug("more than one name in ast.AssignStmt")
			return ErrNotFound
		}

		decl.Rhs[0] = &ast.BasicLit{
			Kind:  token.STRING,
			Value: "`" + value + "`",
		}

	default:
		debug("can only update *ast.ValueSpec, found %T", obj.Decl)
		return ErrNotFound
	}

	var buf bytes.Buffer
	if err := format.Node(&buf, fileset, astFile); err != nil {
		return fmt.Errorf("failed to format file after update: %w", err)
	}

	fh, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to open file %v: %w", filename, err)
	}
	if _, err = fh.Write(buf.Bytes()); err != nil {
		return fmt.Errorf("failed to write file %v: %w", filename, err)
	}
	if err := fh.Sync(); err != nil {
		return fmt.Errorf("failed to sync file %v: %w", filename, err)
	}
	return nil
}

func getIdentForExpectedValueArg(expr []ast.Expr) (int, *ast.Ident) {
	for i := 1; i < 3; i++ {
		switch e := expr[i].(type) {
		case *ast.Ident:
			if strings.HasPrefix(strings.ToLower(e.Name), "expected") {
				return i, e
			}
		}
	}
	return -1, nil
}
