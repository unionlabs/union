package uintcast

import (
	"errors"
	"go/ast"
	"go/types"
	"strings"

	"github.com/gostaticanalysis/comment"
	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/inspector"
)

// Doc explaining the tool.
const Doc = "Ensure that uint variables are not cast improperly where the value could overflow. " +
	"This check can be suppressed with the `lint:ignore uintcast` comment with proper justification."

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "uintcast",
	Doc:      Doc,
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

func run(pass *analysis.Pass) (interface{}, error) {
	inspection, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.CallExpr)(nil),
	}

	commentMap := comment.New(pass.Fset, pass.Files)

	inspection.Preorder(nodeFilter, func(node ast.Node) {
		cg := commentMap.CommentsByPosLine(pass.Fset, node.Pos())
		for _, c := range cg {
			if strings.Contains(c.Text(), "lint:ignore uintcast") {
				return
			}
		}

		switch node := node.(type) {
		case *ast.CallExpr:
			// Cast/conversion calls have one argument and no ellipsis.
			if len(node.Args) != 1 || node.Ellipsis.IsValid() {
				return
			}

			var typ *types.Basic
			switch arg := node.Args[0].(type) {
			case *ast.Ident:
				typ, ok = basicType(pass.TypesInfo.Types[arg].Type)
			case *ast.CallExpr:
				// Check if the call is a builtin conversion/anon identifier.
				typ, ok = basicType(pass.TypesInfo.Types[arg].Type)
				if !ok {
					// Otherwise, it might be a declared function call with a return type.
					typ, ok = funcReturnType(pass.TypesInfo.Types[arg.Fun].Type)
				}
			}
			if typ == nil || !ok {
				return
			}

			// Ignore types that are not uint variants.
			if typ.Kind() < types.Uint || typ.Kind() > types.Uint64 {
				return
			}

			if fnTyp, ok := pass.TypesInfo.Types[node.Fun].Type.(*types.Basic); ok {
				if fnTyp.Kind() >= types.Int && fnTyp.Kind() <= types.Int64 {
					pass.Reportf(node.Args[0].Pos(), "Unsafe cast from %s to %s.", typ, fnTyp)
				}
			}
		}
	})

	return nil, nil
}

func basicType(obj types.Type) (*types.Basic, bool) {
	if obj == nil {
		return nil, false
	}
	fromTyp, ok := obj.(*types.Basic)
	if !ok && obj.Underlying() != nil {
		// Try to get the underlying type
		fromTyp, ok = obj.Underlying().(*types.Basic)
	}
	return fromTyp, ok
}

func funcReturnType(obj types.Type) (*types.Basic, bool) {
	if obj == nil {
		return nil, false
	}
	fnTyp, ok := obj.(*types.Signature)
	if !ok {
		return nil, ok
	}
	if fnTyp.Results().Len() == 0 {
		return nil, false
	}
	return basicType(fnTyp.Results().At(0).Type())
}
