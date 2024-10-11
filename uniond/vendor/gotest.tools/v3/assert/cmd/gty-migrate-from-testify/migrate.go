package main

import (
	"go/ast"
	"go/token"
	"go/types"
	"log"
	"path"

	"golang.org/x/tools/go/ast/astutil"
)

const (
	pkgTestifyAssert       = "github.com/stretchr/testify/assert"
	pkgGopkgTestifyAssert  = "gopkg.in/stretchr/testify.v1/assert"
	pkgTestifyRequire      = "github.com/stretchr/testify/require"
	pkgGopkgTestifyRequire = "gopkg.in/stretchr/testify.v1/require"
	pkgAssert              = "gotest.tools/v3/assert"
	pkgCmp                 = "gotest.tools/v3/assert/cmp"
)

const (
	funcNameAssert = "Assert"
	funcNameCheck  = "Check"
)

var allTestifyPks = []string{
	pkgTestifyAssert,
	pkgTestifyRequire,
	pkgGopkgTestifyAssert,
	pkgGopkgTestifyRequire,
}

type migration struct {
	file        *ast.File
	fileset     *token.FileSet
	importNames importNames
	pkgInfo     *types.Info
}

func migrateFile(migration migration) {
	astutil.Apply(migration.file, nil, replaceCalls(migration))
	updateImports(migration)
}

func updateImports(migration migration) {
	for _, remove := range allTestifyPks {
		astutil.DeleteImport(migration.fileset, migration.file, remove)
	}

	var alias string
	if migration.importNames.assert != path.Base(pkgAssert) {
		alias = migration.importNames.assert
	}
	astutil.AddNamedImport(migration.fileset, migration.file, alias, pkgAssert)

	if migration.importNames.cmp != path.Base(pkgCmp) {
		alias = migration.importNames.cmp
	}
	astutil.AddNamedImport(migration.fileset, migration.file, alias, pkgCmp)
}

type emptyNode struct{}

func (n emptyNode) Pos() token.Pos {
	return 0
}

func (n emptyNode) End() token.Pos {
	return 0
}

var removeNode = emptyNode{}

func replaceCalls(migration migration) func(cursor *astutil.Cursor) bool {
	return func(cursor *astutil.Cursor) bool {
		var newNode ast.Node
		switch typed := cursor.Node().(type) {
		case *ast.SelectorExpr:
			newNode = getReplacementTestingT(typed, migration.importNames)
		case *ast.CallExpr:
			newNode = getReplacementAssertion(typed, migration)
		case *ast.AssignStmt:
			newNode = getReplacementAssignment(typed, migration)
		}

		switch newNode {
		case nil:
		case removeNode:
			cursor.Delete()
		default:
			cursor.Replace(newNode)
		}
		return true
	}
}

func getReplacementTestingT(selector *ast.SelectorExpr, names importNames) ast.Node {
	xIdent, ok := selector.X.(*ast.Ident)
	if !ok {
		return nil
	}
	if selector.Sel.Name != "TestingT" || !names.matchesTestify(xIdent) {
		return nil
	}
	return &ast.SelectorExpr{
		X:   &ast.Ident{Name: names.assert, NamePos: xIdent.NamePos},
		Sel: selector.Sel,
	}
}

func getReplacementAssertion(callExpr *ast.CallExpr, migration migration) ast.Node {
	tcall, ok := newTestifyCallFromNode(callExpr, migration)
	if !ok {
		return nil
	}
	if len(tcall.expr.Args) < 2 {
		return convertTestifySingleArgCall(tcall)
	}
	return convertTestifyAssertion(tcall, migration)
}

func getReplacementAssignment(assign *ast.AssignStmt, migration migration) ast.Node {
	if isAssignmentFromAssertNew(assign, migration) {
		return removeNode
	}
	return nil
}

func convertTestifySingleArgCall(tcall call) ast.Node {
	switch tcall.selExpr.Sel.Name {
	case "TestingT":
		// handled as SelectorExpr
		return nil
	case "New":
		// handled by getReplacementAssignment
		return nil
	default:
		log.Printf("%s: skipping unknown selector", tcall.StringWithFileInfo())
		return nil
	}
}

func convertTestifyAssertion(tcall call, migration migration) ast.Node {
	imports := migration.importNames

	switch tcall.selExpr.Sel.Name {
	case "NoError", "NoErrorf":
		return convertNoError(tcall, imports)
	case "True", "Truef":
		return convertTrue(tcall, imports)
	case "False", "Falsef":
		return convertFalse(tcall, imports)
	case "Equal", "Equalf", "Exactly", "Exactlyf", "EqualValues", "EqualValuesf":
		return convertEqual(tcall, migration)
	case "Contains", "Containsf":
		return convertTwoArgComparison(tcall, imports, "Contains")
	case "Len", "Lenf":
		return convertTwoArgComparison(tcall, imports, "Len")
	case "Panics", "Panicsf":
		return convertOneArgComparison(tcall, imports, "Panics")
	case "EqualError", "EqualErrorf":
		return convertEqualError(tcall, imports)
	case "Error", "Errorf":
		return convertError(tcall, imports)
	case "ErrorContains", "ErrorContainsf":
		return convertErrorContains(tcall, imports)
	case "Empty", "Emptyf":
		return convertEmpty(tcall, imports)
	case "Nil", "Nilf":
		return convertNil(tcall, migration)
	case "NotNil", "NotNilf":
		return convertNegativeComparison(tcall, imports, &ast.Ident{Name: "nil"}, 2)
	case "NotEqual", "NotEqualf":
		return convertNegativeComparison(tcall, imports, tcall.arg(2), 3)
	case "Fail", "Failf":
		return convertFail(tcall, "Error")
	case "FailNow", "FailNowf":
		return convertFail(tcall, "Fatal")
	case "NotEmpty", "NotEmptyf":
		return convertNotEmpty(tcall, imports)
	case "NotZero", "NotZerof":
		zero := &ast.BasicLit{Kind: token.INT, Value: "0"}
		return convertNegativeComparison(tcall, imports, zero, 2)
	}
	log.Printf("%s: skipping unsupported assertion", tcall.StringWithFileInfo())
	return nil
}

func newCallExpr(x, sel string, args []ast.Expr) *ast.CallExpr {
	return &ast.CallExpr{
		Fun: &ast.SelectorExpr{
			X:   &ast.Ident{Name: x},
			Sel: &ast.Ident{Name: sel},
		},
		Args: args,
	}
}

func newCallExprArgs(t ast.Expr, cmp ast.Expr, extra ...ast.Expr) []ast.Expr {
	return append(append([]ast.Expr{t}, cmp), extra...)
}

func newCallExprWithPosition(tcall call, imports importNames, args []ast.Expr) *ast.CallExpr {
	return &ast.CallExpr{
		Fun: &ast.SelectorExpr{
			X: &ast.Ident{
				Name:    imports.assert,
				NamePos: tcall.xIdent.NamePos,
			},
			Sel: &ast.Ident{Name: tcall.assert},
		},
		Args: args,
	}
}

func convertNoError(tcall call, imports importNames) ast.Node {
	// use assert.NilError() for require.NoError()
	if tcall.assert == funcNameAssert {
		return newCallExprWithoutComparison(tcall, imports, "NilError")
	}
	// use assert.Check() for assert.NoError()
	return newCallExprWithoutComparison(tcall, imports, "Check")
}

func convertEqualError(tcall call, imports importNames) ast.Node {
	if tcall.assert == funcNameAssert {
		return newCallExprWithoutComparison(tcall, imports, "Error")
	}
	return convertTwoArgComparison(tcall, imports, "Error")
}

func newCallExprWithoutComparison(tcall call, imports importNames, name string) ast.Node {
	return &ast.CallExpr{
		Fun: &ast.SelectorExpr{
			X: &ast.Ident{
				Name:    imports.assert,
				NamePos: tcall.xIdent.NamePos,
			},
			Sel: &ast.Ident{Name: name},
		},
		Args: tcall.expr.Args,
	}
}

func convertOneArgComparison(tcall call, imports importNames, cmpName string) ast.Node {
	return newCallExprWithPosition(tcall, imports,
		newCallExprArgs(
			tcall.testingT(),
			newCallExpr(imports.cmp, cmpName, []ast.Expr{tcall.arg(1)}),
			tcall.extraArgs(2)...))
}

func convertTrue(tcall call, imports importNames) ast.Node {
	return newCallExprWithPosition(tcall, imports, tcall.expr.Args)
}

func convertFalse(tcall call, imports importNames) ast.Node {
	return newCallExprWithPosition(tcall, imports,
		newCallExprArgs(
			tcall.testingT(),
			&ast.UnaryExpr{Op: token.NOT, X: tcall.arg(1)},
			tcall.extraArgs(2)...))
}

func convertEqual(tcall call, migration migration) ast.Node {
	imports := migration.importNames

	hasExtraArgs := len(tcall.extraArgs(3)) > 0

	cmpEqual := convertTwoArgComparison(tcall, imports, "Equal")
	if tcall.assert == funcNameAssert {
		cmpEqual = newCallExprWithoutComparison(tcall, imports, "Equal")
	}
	cmpDeepEqual := convertTwoArgComparison(tcall, imports, "DeepEqual")
	if tcall.assert == funcNameAssert && !hasExtraArgs {
		cmpDeepEqual = newCallExprWithoutComparison(tcall, imports, "DeepEqual")
	}

	gotype := walkForType(migration.pkgInfo, tcall.arg(1))
	if isUnknownType(gotype) {
		gotype = walkForType(migration.pkgInfo, tcall.arg(2))
	}
	if isUnknownType(gotype) {
		return cmpDeepEqual
	}

	switch gotype.Underlying().(type) {
	case *types.Basic:
		return cmpEqual
	default:
		return cmpDeepEqual
	}
}

func convertTwoArgComparison(tcall call, imports importNames, cmpName string) ast.Node {
	return newCallExprWithPosition(tcall, imports,
		newCallExprArgs(
			tcall.testingT(),
			newCallExpr(imports.cmp, cmpName, tcall.args(1, 3)),
			tcall.extraArgs(3)...))
}

func convertError(tcall call, imports importNames) ast.Node {
	cmpArgs := []ast.Expr{
		tcall.arg(1),
		&ast.BasicLit{Kind: token.STRING, Value: `""`}}

	return newCallExprWithPosition(tcall, imports,
		newCallExprArgs(
			tcall.testingT(),
			newCallExpr(imports.cmp, "ErrorContains", cmpArgs),
			tcall.extraArgs(2)...))
}

func convertErrorContains(tcall call, imports importNames) ast.Node {
	return &ast.CallExpr{
		Fun: &ast.SelectorExpr{
			X: &ast.Ident{
				Name:    imports.assert,
				NamePos: tcall.xIdent.NamePos,
			},
			Sel: &ast.Ident{Name: "ErrorContains"},
		},
		Args: tcall.expr.Args,
	}
}

func convertEmpty(tcall call, imports importNames) ast.Node {
	cmpArgs := []ast.Expr{
		tcall.arg(1),
		&ast.BasicLit{Kind: token.INT, Value: "0"},
	}
	return newCallExprWithPosition(tcall, imports,
		newCallExprArgs(
			tcall.testingT(),
			newCallExpr(imports.cmp, "Len", cmpArgs),
			tcall.extraArgs(2)...))
}

func convertNil(tcall call, migration migration) ast.Node {
	gotype := walkForType(migration.pkgInfo, tcall.arg(1))
	if gotype != nil && gotype.String() == "error" {
		return convertNoError(tcall, migration.importNames)
	}
	return convertOneArgComparison(tcall, migration.importNames, "Nil")
}

func convertNegativeComparison(
	tcall call,
	imports importNames,
	right ast.Expr,
	extra int,
) ast.Node {
	return newCallExprWithPosition(tcall, imports,
		newCallExprArgs(
			tcall.testingT(),
			&ast.BinaryExpr{X: tcall.arg(1), Op: token.NEQ, Y: right},
			tcall.extraArgs(extra)...))
}

func convertFail(tcall call, selector string) ast.Node {
	extraArgs := tcall.extraArgs(1)
	if len(extraArgs) > 1 {
		selector += "f"
	}

	return &ast.CallExpr{
		Fun: &ast.SelectorExpr{
			X:   tcall.testingT(),
			Sel: &ast.Ident{Name: selector},
		},
		Args: extraArgs,
	}
}

func convertNotEmpty(tcall call, imports importNames) ast.Node {
	lenExpr := &ast.CallExpr{
		Fun:  &ast.Ident{Name: "len"},
		Args: tcall.args(1, 2),
	}
	zeroExpr := &ast.BasicLit{Kind: token.INT, Value: "0"}
	return newCallExprWithPosition(tcall, imports,
		newCallExprArgs(
			tcall.testingT(),
			&ast.BinaryExpr{X: lenExpr, Op: token.NEQ, Y: zeroExpr},
			tcall.extraArgs(2)...))
}
