package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/format"
	"go/token"
)

// call wraps a testify/assert ast.CallExpr and exposes properties of the
// expression to facilitate migrating the expression to a gotest.tools/v3/assert
type call struct {
	fileset *token.FileSet
	expr    *ast.CallExpr
	xIdent  *ast.Ident
	selExpr *ast.SelectorExpr
	// assert is Assert (if the testify package was require), or Check (if the
	// testify package was assert).
	assert string
}

func (c call) String() string {
	buf := new(bytes.Buffer)
	_ = format.Node(buf, token.NewFileSet(), c.expr)
	return buf.String()
}

func (c call) StringWithFileInfo() string {
	if c.fileset.File(c.expr.Pos()) == nil {
		return fmt.Sprintf("%s at unknown file", c)
	}
	return fmt.Sprintf("%s at %s:%d", c,
		relativePath(c.fileset.File(c.expr.Pos()).Name()),
		c.fileset.Position(c.expr.Pos()).Line)
}

// testingT returns the first argument of the call, which is assumed to be a
// *ast.Ident of *testing.T or a compatible interface.
func (c call) testingT() ast.Expr {
	if len(c.expr.Args) == 0 {
		return nil
	}
	return c.expr.Args[0]
}

// extraArgs returns the arguments of the expression starting from index
func (c call) extraArgs(index int) []ast.Expr {
	if len(c.expr.Args) <= index {
		return nil
	}
	return c.expr.Args[index:]
}

// args returns a range of arguments from the expression
func (c call) args(from, to int) []ast.Expr {
	return c.expr.Args[from:to]
}

// arg returns a single argument from the expression
func (c call) arg(index int) ast.Expr {
	return c.expr.Args[index]
}

// newCallFromCallExpr returns a new call build from a ast.CallExpr. Returns false
// if the call expression does not have the expected ast nodes.
func newCallFromCallExpr(callExpr *ast.CallExpr, migration migration) (call, bool) {
	c := call{}
	selector, ok := callExpr.Fun.(*ast.SelectorExpr)
	if !ok {
		return c, false
	}
	ident, ok := selector.X.(*ast.Ident)
	if !ok {
		return c, false
	}

	return call{
		fileset: migration.fileset,
		xIdent:  ident,
		selExpr: selector,
		expr:    callExpr,
	}, true
}

// newTestifyCallFromNode returns a call that wraps a valid testify assertion.
// Returns false if the call expression is not a testify assertion.
func newTestifyCallFromNode(callExpr *ast.CallExpr, migration migration) (call, bool) {
	tcall, ok := newCallFromCallExpr(callExpr, migration)
	if !ok {
		return tcall, false
	}

	testifyNewAssignStmt := testifyAssertionsAssignment(tcall, migration)
	switch {
	case testifyNewAssignStmt != nil:
		return updateCallForTestifyNew(tcall, testifyNewAssignStmt, migration)
	case isTestifyPkgCall(tcall, migration):
		tcall.assert = migration.importNames.funcNameFromTestifyName(tcall.xIdent.Name)
		return tcall, true
	}
	return tcall, false
}

// isTestifyPkgCall returns true if the call is a testify package-level assertion
// (as apposed to an assertion method on the Assertions type)
//
// TODO: check if the xIdent.Obj.Decl is an import declaration instead of
// assuming that a name matching the import name is always an import. Some code
// may shadow import names, which could lead to incorrect results.
func isTestifyPkgCall(tcall call, migration migration) bool {
	return migration.importNames.matchesTestify(tcall.xIdent)
}

// testifyAssertionsAssignment returns an ast.AssignStmt if the call is a testify
// call from an Assertions object returned from assert.New(t) (not a package level
// assert). Otherwise returns nil.
func testifyAssertionsAssignment(tcall call, migration migration) *ast.AssignStmt {
	if tcall.xIdent.Obj == nil {
		return nil
	}

	assignStmt, ok := tcall.xIdent.Obj.Decl.(*ast.AssignStmt)
	if !ok {
		return nil
	}

	if isAssignmentFromAssertNew(assignStmt, migration) {
		return assignStmt
	}
	return nil
}

func updateCallForTestifyNew(
	tcall call,
	testifyNewAssignStmt *ast.AssignStmt,
	migration migration,
) (call, bool) {
	testifyNewCallExpr := callExprFromAssignment(testifyNewAssignStmt)
	if testifyNewCallExpr == nil {
		return tcall, false
	}
	testifyNewCall, ok := newCallFromCallExpr(testifyNewCallExpr, migration)
	if !ok {
		return tcall, false
	}

	tcall.assert = migration.importNames.funcNameFromTestifyName(testifyNewCall.xIdent.Name)
	tcall.expr = addMissingTestingTArgToCallExpr(tcall.expr, testifyNewCall.testingT())
	return tcall, true
}

// addMissingTestingTArgToCallExpr adds a testingT arg as the first arg of the
// ast.CallExpr and returns a copy of the ast.CallExpr
func addMissingTestingTArgToCallExpr(callExpr *ast.CallExpr, testingT ast.Expr) *ast.CallExpr {
	return &ast.CallExpr{
		Fun:  callExpr.Fun,
		Args: append([]ast.Expr{removePos(testingT)}, callExpr.Args...),
	}
}

func removePos(node ast.Expr) ast.Expr {
	switch typed := node.(type) {
	case *ast.Ident:
		return &ast.Ident{Name: typed.Name}
	}
	return node
}

// TODO: use pkgInfo and walkForType instead?
func isAssignmentFromAssertNew(assign *ast.AssignStmt, migration migration) bool {
	callExpr := callExprFromAssignment(assign)
	if callExpr == nil {
		return false
	}
	tcall, ok := newCallFromCallExpr(callExpr, migration)
	if !ok {
		return false
	}
	if !migration.importNames.matchesTestify(tcall.xIdent) {
		return false
	}

	if len(tcall.expr.Args) != 1 {
		return false
	}
	return tcall.selExpr.Sel.Name == "New"
}

func callExprFromAssignment(assign *ast.AssignStmt) *ast.CallExpr {
	if len(assign.Rhs) != 1 {
		return nil
	}

	callExpr, ok := assign.Rhs[0].(*ast.CallExpr)
	if !ok {
		return nil
	}
	return callExpr
}
