package gocognit

import (
	"errors"
	"fmt"
	"go/ast"

	"github.com/uudashr/gocognit"
	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Tool to ensure go code does not have high cognitive complexity."

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "gocognit",
	Doc:      Doc,
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

// Recommended thresholds according to the 2008 presentation titled
// "Software Quality Metrics to Identify Risk" by Thomas McCabe Jr.
//
//	1 - 10 Simple procedure, little risk
//
// 11 - 20 More complex, moderate risk
// 21 - 50 Complex, high risk
// > 50 Untestable code, very high risk
//
// This threshold should be lowered to 50 over time.
const over = 100

func run(pass *analysis.Pass) (interface{}, error) {
	inspectResult, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.FuncDecl)(nil),
	}
	inspectResult.Preorder(nodeFilter, func(n ast.Node) {
		fnDecl, ok := n.(*ast.FuncDecl)
		if !ok {
			return
		}

		fnName := funcName(fnDecl)
		fnComplexity := gocognit.Complexity(fnDecl)

		if fnComplexity > over {
			pass.Reportf(fnDecl.Pos(), "cognitive complexity %d of func %s is high (> %d)", fnComplexity, fnName, over)
		}
	})

	return nil, nil
}

// funcName returns the name representation of a function or method:
// "(Type).Name" for methods or simply "Name" for functions.
//
// Copied from https://github.com/uudashr/gocognit/blob/5bf67146515e79acd2a8d5728deafa9d91ad48db/gocognit.go
// License: MIT
func funcName(fn *ast.FuncDecl) string {
	if fn.Recv != nil {
		if fn.Recv.NumFields() > 0 {
			typ := fn.Recv.List[0].Type
			return fmt.Sprintf("(%s).%s", recvString(typ), fn.Name)
		}
	}
	return fn.Name.Name
}

// recvString returns a string representation of recv of the
// form "T", "*T", or "BADRECV" (if not a proper receiver type).
//
// Copied from https://github.com/uudashr/gocognit/blob/5bf67146515e79acd2a8d5728deafa9d91ad48db/gocognit.go
// License: MIT
func recvString(recv ast.Expr) string {
	switch t := recv.(type) {
	case *ast.Ident:
		return t.Name
	case *ast.StarExpr:
		return "*" + recvString(t.X)
	}
	return "BADRECV"
}
