// Analyzer tool for detecting nested or recursive mutex read lock statements

package recursivelock

import (
	"errors"
	"fmt"
	"go/ast"
	"go/token"
	"go/types"

	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/passes/inspect"
	"golang.org/x/tools/go/ast/astutil"
	"golang.org/x/tools/go/ast/inspector"
	"golang.org/x/tools/go/types/typeutil"
)

// Analyzer runs static analysis.
var Analyzer = &analysis.Analyzer{
	Name:     "recursivelock",
	Doc:      "Checks for recursive or nested RLock calls",
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	Run:      run,
}

var errNestedRLock = errors.New("found recursive read lock call")
var errNestedLock = errors.New("found recursive lock call")
var errNestedMixedLock = errors.New("found recursive mixed lock call")

type mode int

const (
	LockMode = mode(iota)
	RLockMode
)

func (m mode) LockName() string {
	switch m {
	case LockMode:
		return "Lock"
	case RLockMode:
		return "RLock"
	}
	return ""
}

func (m mode) UnLockName() string {
	switch m {
	case LockMode:
		return "Unlock"
	case RLockMode:
		return "RUnlock"
	}
	return ""
}

func (m mode) ErrorFound() error {
	switch m {
	case LockMode:
		return errNestedLock
	case RLockMode:
		return errNestedRLock
	}
	return nil
}

func run(pass *analysis.Pass) (interface{}, error) {
	inspectResult, ok := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)
	if !ok {
		return nil, errors.New("analyzer is not type *inspector.Inspector")
	}

	nodeFilter := []ast.Node{
		(*ast.GoStmt)(nil),
		(*ast.CallExpr)(nil),
		(*ast.DeferStmt)(nil),
		(*ast.FuncDecl)(nil),
		(*ast.FuncLit)(nil),
		(*ast.File)(nil),
		(*ast.IfStmt)(nil),
		(*ast.ReturnStmt)(nil),
	}

	keepTrackOf := &tracker{
		rLockTrack: &lockTracker{},
		lockTrack:  &lockTracker{},
	}
	inspectResult.Preorder(nodeFilter, func(node ast.Node) {
		if keepTrackOf.rLockTrack.funcLitEnd.IsValid() && node.Pos() <= keepTrackOf.rLockTrack.funcLitEnd &&
			keepTrackOf.lockTrack.funcLitEnd.IsValid() && node.Pos() <= keepTrackOf.lockTrack.funcLitEnd {
			return
		}
		keepTrackOf.rLockTrack.funcLitEnd = token.NoPos
		keepTrackOf.lockTrack.funcLitEnd = token.NoPos

		if keepTrackOf.rLockTrack.deferEnd.IsValid() && node.Pos() > keepTrackOf.rLockTrack.deferEnd {
			keepTrackOf.rLockTrack.deferEnd = token.NoPos
		} else if keepTrackOf.rLockTrack.deferEnd.IsValid() {
			return
		}
		if keepTrackOf.lockTrack.deferEnd.IsValid() && node.Pos() > keepTrackOf.lockTrack.deferEnd {
			keepTrackOf.lockTrack.deferEnd = token.NoPos
		} else if keepTrackOf.lockTrack.deferEnd.IsValid() {
			return
		}

		if keepTrackOf.rLockTrack.retEnd.IsValid() && node.Pos() > keepTrackOf.rLockTrack.retEnd {
			keepTrackOf.rLockTrack.retEnd = token.NoPos
			keepTrackOf.rLockTrack.incFRU()
		}
		if keepTrackOf.lockTrack.retEnd.IsValid() && node.Pos() > keepTrackOf.lockTrack.retEnd {
			keepTrackOf.lockTrack.retEnd = token.NoPos
			keepTrackOf.lockTrack.incFRU()
		}
		keepTrackOf = stmtSelector(node, pass, keepTrackOf, inspectResult)
	})
	return nil, nil
}

func stmtSelector(node ast.Node, pass *analysis.Pass, keepTrackOf *tracker, inspect *inspector.Inspector) *tracker {
	switch stmt := node.(type) {
	case *ast.GoStmt:
		keepTrackOf.rLockTrack.goroutinePos = stmt.Call.End()
		keepTrackOf.lockTrack.goroutinePos = stmt.Call.End()
	case *ast.CallExpr:
		if stmt.End() == keepTrackOf.rLockTrack.goroutinePos ||
			stmt.End() == keepTrackOf.lockTrack.goroutinePos {
			keepTrackOf.rLockTrack.goroutinePos = 0
			keepTrackOf.lockTrack.goroutinePos = 0
			break
		}
		call := getCallInfo(pass.TypesInfo, stmt)
		if call == nil {
			break
		}
		selMap := mapSelTypes(stmt, pass)
		if selMap == nil {
			break
		}
		checkForRecLocks(node, pass, inspect, RLockMode, call, keepTrackOf.rLockTrack, selMap)
		checkForRecLocks(node, pass, inspect, LockMode, call, keepTrackOf.lockTrack, selMap)

	case *ast.File:
		keepTrackOf = &tracker{
			rLockTrack: &lockTracker{},
			lockTrack:  &lockTracker{},
		}

	case *ast.FuncDecl:
		keepTrackOf = &tracker{
			rLockTrack: &lockTracker{},
			lockTrack:  &lockTracker{},
		}
		keepTrackOf.rLockTrack.funcEnd = stmt.End()

	case *ast.FuncLit:
		if keepTrackOf.rLockTrack.funcLitEnd == token.NoPos {
			keepTrackOf.rLockTrack.funcLitEnd = stmt.End()
		}
		if keepTrackOf.lockTrack.funcLitEnd == token.NoPos {
			keepTrackOf.lockTrack.funcLitEnd = stmt.End()
		}
	case *ast.IfStmt:
		stmts := stmt.Body.List
		for i := 0; i < len(stmts); i++ {
			keepTrackOf = stmtSelector(stmts[i], pass, keepTrackOf, inspect)
		}
		keepTrackOf = stmtSelector(stmt.Else, pass, keepTrackOf, inspect)
	case *ast.DeferStmt:
		call := getCallInfo(pass.TypesInfo, stmt.Call)
		if keepTrackOf.rLockTrack.deferEnd == token.NoPos {
			keepTrackOf.rLockTrack.deferEnd = stmt.End()
		}
		if keepTrackOf.lockTrack.deferEnd == token.NoPos {
			keepTrackOf.lockTrack.deferEnd = stmt.End()
		}

		if call != nil && call.name == RLockMode.UnLockName() {
			keepTrackOf.rLockTrack.deferredRUnlock = true
		}
		if call != nil && call.name == LockMode.UnLockName() {
			keepTrackOf.lockTrack.deferredRUnlock = true
		}

	case *ast.ReturnStmt:
		for i := 0; i < len(stmt.Results); i++ {
			keepTrackOf = stmtSelector(stmt.Results[i], pass, keepTrackOf, inspect)
		}
		if keepTrackOf.rLockTrack.deferredRUnlock && keepTrackOf.rLockTrack.retEnd == token.NoPos {
			keepTrackOf.rLockTrack.deincFRU()
			keepTrackOf.rLockTrack.retEnd = stmt.End()
		}
		if keepTrackOf.lockTrack.deferredRUnlock && keepTrackOf.lockTrack.retEnd == token.NoPos {
			keepTrackOf.lockTrack.deincFRU()
			keepTrackOf.lockTrack.retEnd = stmt.End()
		}
	}
	return keepTrackOf
}

type tracker struct {
	rLockTrack *lockTracker
	lockTrack  *lockTracker
}

type lockTracker struct {
	funcEnd         token.Pos
	retEnd          token.Pos
	deferEnd        token.Pos
	funcLitEnd      token.Pos
	goroutinePos    token.Pos
	deferredRUnlock bool
	foundRLock      int
	rLockSelector   *selIdentList
}

func (t lockTracker) String() string {
	return fmt.Sprintf("funcEnd:%v\nretEnd:%v\ndeferEnd:%v\ndeferredRU:%v\nfoundRLock:%v\n", t.funcEnd, t.retEnd, t.deferEnd, t.deferredRUnlock, t.foundRLock)
}

func (t *lockTracker) deincFRU() {
	if t.foundRLock > 0 {
		t.foundRLock -= 1
	}
}
func (t *lockTracker) incFRU() {
	t.foundRLock += 1
}

func checkForRecLocks(node ast.Node, pass *analysis.Pass, inspect *inspector.Inspector, lockmode mode, call *callInfo,
	lockTracker *lockTracker, selMap *selIdentList) {
	name := call.name
	if lockTracker.rLockSelector != nil {
		if lockTracker.foundRLock > 0 {
			if lockTracker.rLockSelector.isRelated(selMap, 0) {
				pass.Reportf(
					node.Pos(),
					fmt.Sprintf(
						"%v",
						errNestedMixedLock,
					),
				)
			}
			if lockTracker.rLockSelector.isEqual(selMap, 0) {
				pass.Reportf(
					node.Pos(),
					fmt.Sprintf(
						"%v",
						lockmode.ErrorFound(),
					),
				)
			} else {
				if stack := hasNestedlock(lockTracker.rLockSelector, lockTracker.goroutinePos, selMap, call, inspect, pass, make(map[string]bool),
					lockmode.UnLockName()); stack != "" {
					pass.Reportf(
						node.Pos(),
						fmt.Sprintf(
							"%v\n%v",
							lockmode.ErrorFound(),
							stack,
						),
					)
				}
			}
		}
		if name == lockmode.UnLockName() && lockTracker.rLockSelector.isEqual(selMap, 1) {
			lockTracker.deincFRU()
		}
		if name == lockmode.LockName() && lockTracker.foundRLock == 0 && lockTracker.rLockSelector.isEqual(selMap, 0) {
			lockTracker.incFRU()
		}
	} else if name == lockmode.LockName() && lockTracker.foundRLock == 0 {
		lockTracker.rLockSelector = selMap
		lockTracker.incFRU()
	}
}

// Stores the AST and type information of a single item in a selector expression
// For example, "a.b.c()", a selIdentNode might store the information for "a"
type selIdentNode struct {
	next   *selIdentNode
	this   *ast.Ident
	typObj types.Object
}

// a list of selIdentNodes. Stores the information of an entire selector expression
// For example, each item in "a.b.c()" is stored as a node in this list, with the start node being "a"
type selIdentList struct {
	start        *selIdentNode
	length       int
	current      *selIdentNode // used for internal functions
	currentIndex int           // used for internal functions
}

// returns the next item in the list, and increments the counter keeping track of where we are in the list
func (s *selIdentList) next() (n *selIdentNode) {
	n = s.current.next
	if n != nil {
		s.current = n
		s.currentIndex++
	}
	return n
}

// reset resets the current node to the start node in the list
func (s *selIdentList) reset() {
	s.current = s.start
	s.currentIndex = 0
}

// isEqual returns true if two selIdentLists are equal to each other.
// The offset parameter tells how far in the list to check for equality.
// For example, a.b.c() and a.b.d() are equal with an offset of 1.
func (s *selIdentList) isEqual(s2 *selIdentList, offset int) bool {
	if s2 == nil || (s.length != s2.length) {
		return false
	}
	s.reset()
	s2.reset()
	for i := true; i; {
		if !s.current.isEqual(s2.current) {
			return false
		}
		if s.currentIndex < s.length-offset-1 && s.next() != nil {
			s2.next()
		} else {
			i = false
		}
	}
	return true
}

// isRelated checks if our selectors are of the same type and
// reference the same underlying object. If they do we check
// if the provided list is referencing a non-equal but related
// lock. Ex: Lock - RLock, RLock - Lock
// TODO: Use a generalizable method here instead of hardcoding
// the lock definitions here.
func (s *selIdentList) isRelated(s2 *selIdentList, offset int) bool {
	if s2 == nil || (s.length != s2.length) {
		return false
	}
	s.reset()
	s2.reset()
	for i := true; i; {
		if !s.current.isEqual(s2.current) {
			return false
		}
		if s.currentIndex < s.length-offset-1 && s.next() != nil {
			s2.next()
		} else {
			i = false
		}
		// Only check if we are at the last index for
		// related method calls.
		if s.currentIndex == s.length-1 {
			switch s.current.this.String() {
			case LockMode.LockName():
				if s2.current.this.String() == RLockMode.LockName() {
					return true
				}
			case RLockMode.LockName():
				if s2.current.this.String() == LockMode.LockName() {
					return true
				}
			}
		}
	}
	return false
}

// getSub returns the shared beginning selIdentList of s and s2,
// if s contains all elements (except the last) of s2,
// and returns nil otherwise.
// For example, if s represents "a.b.c.d()" and s2 represents
// "a.b.e()", getSub will return a selIdentList representing "a.b".
// getSub returns nil if s2's length is greater than that of s
func (s *selIdentList) getSub(s2 *selIdentList) *selIdentList {
	if s2 == nil || s2.length > s.length {
		return nil
	}
	s.reset()
	s2.reset()
	for i := true; i; {
		if !s.current.isEqual(s2.current) {
			return nil
		}
		if s2.currentIndex != s2.length-2 { // might want to add a selNode.prev() func
			s.next()
			s2.next()
		} else {
			i = false
		}
	}
	return &selIdentList{
		start:        s.current,
		length:       s.length - s.currentIndex,
		current:      s.current,
		currentIndex: 0,
	}
}

// changeRoot changes the first selIdentNode of a selIdentList
// to one with given *ast.Ident and types.Object
func (s *selIdentList) changeRoot(r *ast.Ident, t types.Object) {
	selNode := &selIdentNode{
		this:   r,
		next:   s.start.next,
		typObj: t,
	}
	if s.start == s.current {
		s.start = selNode
		s.current = selNode
	} else {
		s.start = selNode
	}
}

func (s selIdentList) String() (str string) {
	var temp = s.start
	str = fmt.Sprintf("length: %v\n[\n", s.length)
	for i := 0; temp != nil; i++ {
		if i == s.currentIndex {
			str += "*"
		}
		str += fmt.Sprintf("%v: %v\n", i, temp)
		temp = temp.next
	}
	str += "]"
	return str
}

func (s *selIdentNode) isEqual(s2 *selIdentNode) bool {
	return (s.this.Name == s2.this.Name) && (s.typObj == s2.typObj)
}

func (s selIdentNode) String() string {
	return fmt.Sprintf("{ ident: '%v', type: '%v' }", s.this, s.typObj)
}

// mapSelTypes returns a selIdentList representation of the given call expression
func mapSelTypes(c *ast.CallExpr, pass *analysis.Pass) *selIdentList {
	list := &selIdentList{}
	valid := list.recurMapSelTypes(c.Fun, nil, pass.TypesInfo)
	if !valid {
		return nil
	}
	return list
}

// recursively identifies the type of each identity node in a selector expression
func (l *selIdentList) recurMapSelTypes(e ast.Expr, next *selIdentNode, t *types.Info) bool {
	expr := astutil.Unparen(e)
	l.length++
	s := &selIdentNode{next: next}
	switch stmt := expr.(type) {
	case *ast.Ident:
		s.this = stmt
		s.typObj = t.ObjectOf(stmt)
	case *ast.SelectorExpr:
		s.this = stmt.Sel
		if sel, ok := t.Selections[stmt]; ok {
			s.typObj = sel.Obj() // method or field
		} else {
			s.typObj = t.Uses[stmt.Sel] // qualified identifier?
		}
		return l.recurMapSelTypes(stmt.X, s, t)
	default:
		return false
	}
	l.current = s
	l.start = s
	return true
}

type callInfo struct {
	call *ast.CallExpr
	id   string // String representation of the type object
	name string // type ID [either the name (if the function is exported) or the package/name if otherwise] of the function/method
}

// getCallInfo returns a *callInfo struct with call info
func getCallInfo(tInfo *types.Info, call *ast.CallExpr) (c *callInfo) {
	c = &callInfo{}
	c.call = call
	f := typeutil.Callee(tInfo, call)
	if f == nil {
		return nil
	}
	if _, isBuiltin := f.(*types.Builtin); isBuiltin {
		return nil
	}
	s, ok := f.Type().(*types.Signature)
	if ok && interfaceMethod(s) {
		return nil
	}
	c.id = f.String()
	c.name = f.Id()
	return c
}

func interfaceMethod(s *types.Signature) bool {
	recv := s.Recv()
	return recv != nil && types.IsInterface(recv.Type())
}

// hasNestedlock returns a stack trace of the nested or recursive lock within the declaration of a function/method call (given by call).
// If the call expression does not contain a nested or recursive lock, hasNestedlock returns an empty string.
// hasNestedlock finds a nested or recursive lock by recursively calling itself on any functions called by the function/method represented
// by callInfo.
func hasNestedlock(fullRLockSelector *selIdentList, goPos token.Pos, compareMap *selIdentList, call *callInfo, inspect *inspector.Inspector,
	pass *analysis.Pass, hist map[string]bool, lockName string) (retStack string) {
	var rLockSelector *selIdentList
	f := pass.Fset
	tInfo := pass.TypesInfo
	cH := callHelper{
		call: call.call,
		fset: pass.Fset,
	}
	var node ast.Node = cH.identifyFuncLitBlock(cH.call.Fun) // this seems a bit redundant
	var recv *ast.Ident
	if node == (*ast.BlockStmt)(nil) {
		subMap := fullRLockSelector.getSub(compareMap)
		if subMap != nil {
			rLockSelector = subMap
		} else {
			return "" // if this is not a local function literal call, and the selectors don't match up, then we can just return
		}
		node = findCallDeclarationNode(call, inspect, pass.TypesInfo)
		if node == (*ast.FuncDecl)(nil) {
			return ""
		} else if castedNode, ok := node.(*ast.FuncDecl); ok && castedNode.Recv != nil {
			recv = castedNode.Recv.List[0].Names[0]
			rLockSelector.changeRoot(recv, pass.TypesInfo.ObjectOf(recv))
		}
	} else {
		rLockSelector = fullRLockSelector // no need to find a submap, since this is a local function call
	}
	addition := fmt.Sprintf("\t%q at %v\n", call.name, f.Position(call.call.Pos()))
	ast.Inspect(node, func(iNode ast.Node) bool {
		switch stmt := iNode.(type) {
		case *ast.GoStmt:
			goPos = stmt.End()
		case *ast.CallExpr:
			if stmt.End() == goPos {
				goPos = 0
				return false
			}
			c := getCallInfo(tInfo, stmt)
			if c == nil {
				return false
			}
			name := c.name
			selMap := mapSelTypes(stmt, pass)
			if rLockSelector.isEqual(selMap, 0) || rLockSelector.isRelated(selMap, 0) { // if the method found is an RLock method
				retStack += addition + fmt.Sprintf("\t%q at %v\n", name, f.Position(iNode.Pos()))
			} else if name != lockName { // name should not equal the previousName to prevent infinite recursive loop
				nt := c.id
				if !hist[nt] { // make sure we are not in an infinite recursive loop
					hist[nt] = true
					stack := hasNestedlock(rLockSelector, goPos, selMap, c, inspect, pass, hist, lockName)
					delete(hist, nt)
					if stack != "" {
						retStack += addition + stack
					}
				}
			}
		}
		return true
	})
	return retStack
}

// findCallDeclarationNode takes a callInfo struct and inspects the AST of the package
// to find a matching method or function declaration. It returns this declaration of type *ast.FuncDecl
func findCallDeclarationNode(c *callInfo, inspect *inspector.Inspector, tInfo *types.Info) *ast.FuncDecl {
	var retNode *ast.FuncDecl = nil
	nodeFilter := []ast.Node{
		(*ast.FuncDecl)(nil),
	}
	inspect.Preorder(nodeFilter, func(node ast.Node) {
		funcDec, ok := node.(*ast.FuncDecl)
		if !ok {
			return
		}
		compareId := tInfo.ObjectOf(funcDec.Name).String()
		if c.id == compareId {
			retNode = funcDec
		}
	})
	return retNode
}

type callHelper struct {
	call *ast.CallExpr
	fset *token.FileSet
}

// identifyFuncLitBlock returns the AST block statement of the function literal called by the given expression,
// or nil if no function literal block statement could be identified.
func (c callHelper) identifyFuncLitBlock(expr ast.Expr) *ast.BlockStmt {
	switch stmt := expr.(type) {
	case *ast.FuncLit:
		return stmt.Body
	case *ast.Ident:
		if stmt.Obj != nil {
			switch objDecl := stmt.Obj.Decl.(type) {
			case *ast.ValueSpec:
				identIndex := findIdentIndex(stmt, objDecl.Names)
				if identIndex != -1 && len(objDecl.Names) == len(objDecl.Values) {
					value := objDecl.Values[identIndex]
					return c.identifyFuncLitBlock(value)
				}
			case *ast.AssignStmt:
				exprIndex := findIdentIndexFromExpr(stmt, objDecl.Lhs)
				if exprIndex != -1 && len(objDecl.Lhs) == len(objDecl.Rhs) { // only deals with simple func lit assignments
					value := objDecl.Rhs[exprIndex]
					return c.identifyFuncLitBlock(value)
				}
			}
		}
	}
	return nil
}

func findIdentIndex(id *ast.Ident, exprs []*ast.Ident) int {
	for i, v := range exprs {
		if v.Name == id.Name {
			return i
		}
	}
	return -1
}

func findIdentIndexFromExpr(id *ast.Ident, exprs []ast.Expr) int {
	for i, v := range exprs {
		if val, ok := v.(*ast.Ident); ok && val.Name == id.Name {
			return i
		}
	}
	return -1
}
