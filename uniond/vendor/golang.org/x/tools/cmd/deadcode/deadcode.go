// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.20

package main

import (
	"bytes"
	_ "embed"
	"encoding/json"
	"flag"
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"io"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"runtime/pprof"
	"sort"
	"strings"
	"text/template"

	"golang.org/x/telemetry"
	"golang.org/x/tools/go/callgraph"
	"golang.org/x/tools/go/callgraph/rta"
	"golang.org/x/tools/go/packages"
	"golang.org/x/tools/go/ssa"
	"golang.org/x/tools/go/ssa/ssautil"
	"golang.org/x/tools/internal/typesinternal"
)

//go:embed doc.go
var doc string

// flags
var (
	testFlag = flag.Bool("test", false, "include implicit test packages and executables")
	tagsFlag = flag.String("tags", "", "comma-separated list of extra build tags (see: go help buildconstraint)")

	filterFlag    = flag.String("filter", "<module>", "report only packages matching this regular expression (default: module of first package)")
	generatedFlag = flag.Bool("generated", false, "include dead functions in generated Go files")
	whyLiveFlag   = flag.String("whylive", "", "show a path from main to the named function")
	formatFlag    = flag.String("f", "", "format output records using template")
	jsonFlag      = flag.Bool("json", false, "output JSON records")
	cpuProfile    = flag.String("cpuprofile", "", "write CPU profile to this file")
	memProfile    = flag.String("memprofile", "", "write memory profile to this file")
)

func usage() {
	// Extract the content of the /* ... */ comment in doc.go.
	_, after, _ := strings.Cut(doc, "/*\n")
	doc, _, _ := strings.Cut(after, "*/")
	io.WriteString(flag.CommandLine.Output(), doc+`
Flags:

`)
	flag.PrintDefaults()
}

func main() {
	telemetry.Start(telemetry.Config{ReportCrashes: true})

	log.SetPrefix("deadcode: ")
	log.SetFlags(0) // no time prefix

	flag.Usage = usage
	flag.Parse()
	if len(flag.Args()) == 0 {
		usage()
		os.Exit(2)
	}

	if *cpuProfile != "" {
		f, err := os.Create(*cpuProfile)
		if err != nil {
			log.Fatal(err)
		}
		if err := pprof.StartCPUProfile(f); err != nil {
			log.Fatal(err)
		}
		// NB: profile won't be written in case of error.
		defer pprof.StopCPUProfile()
	}

	if *memProfile != "" {
		f, err := os.Create(*memProfile)
		if err != nil {
			log.Fatal(err)
		}
		// NB: profile won't be written in case of error.
		defer func() {
			runtime.GC() // get up-to-date statistics
			if err := pprof.WriteHeapProfile(f); err != nil {
				log.Fatalf("Writing memory profile: %v", err)
			}
			f.Close()
		}()
	}

	// Reject bad output options early.
	if *formatFlag != "" {
		if *jsonFlag {
			log.Fatalf("you cannot specify both -f=template and -json")
		}
		if _, err := template.New("deadcode").Parse(*formatFlag); err != nil {
			log.Fatalf("invalid -f: %v", err)
		}
	}

	// Load, parse, and type-check the complete program(s).
	cfg := &packages.Config{
		BuildFlags: []string{"-tags=" + *tagsFlag},
		Mode:       packages.LoadAllSyntax | packages.NeedModule,
		Tests:      *testFlag,
	}
	initial, err := packages.Load(cfg, flag.Args()...)
	if err != nil {
		log.Fatalf("Load: %v", err)
	}
	if len(initial) == 0 {
		log.Fatalf("no packages")
	}
	if packages.PrintErrors(initial) > 0 {
		log.Fatalf("packages contain errors")
	}

	// If -filter is unset, use first module (if available).
	if *filterFlag == "<module>" {
		if mod := initial[0].Module; mod != nil && mod.Path != "" {
			*filterFlag = "^" + regexp.QuoteMeta(mod.Path) + "\\b"
		} else {
			*filterFlag = "" // match any
		}
	}
	filter, err := regexp.Compile(*filterFlag)
	if err != nil {
		log.Fatalf("-filter: %v", err)
	}

	// Create SSA-form program representation
	// and find main packages.
	prog, pkgs := ssautil.AllPackages(initial, ssa.InstantiateGenerics)
	prog.Build()

	mains := ssautil.MainPackages(pkgs)
	if len(mains) == 0 {
		log.Fatalf("no main packages")
	}
	var roots []*ssa.Function
	for _, main := range mains {
		roots = append(roots, main.Func("init"), main.Func("main"))
	}

	// Gather all source-level functions,
	// as the user interface is expressed in terms of them.
	//
	// We ignore synthetic wrappers, and nested functions. Literal
	// functions passed as arguments to other functions are of
	// course address-taken and there exists a dynamic call of
	// that signature, so when they are unreachable, it is
	// invariably because the parent is unreachable.
	var sourceFuncs []*ssa.Function
	generated := make(map[string]bool)
	packages.Visit(initial, nil, func(p *packages.Package) {
		for _, file := range p.Syntax {
			for _, decl := range file.Decls {
				if decl, ok := decl.(*ast.FuncDecl); ok {
					obj := p.TypesInfo.Defs[decl.Name].(*types.Func)
					fn := prog.FuncValue(obj)
					sourceFuncs = append(sourceFuncs, fn)
				}
			}

			if isGenerated(file) {
				generated[p.Fset.File(file.Pos()).Name()] = true
			}
		}
	})

	// Compute the reachabilty from main.
	// (Build a call graph only for -whylive.)
	res := rta.Analyze(roots, *whyLiveFlag != "")

	// Subtle: the -test flag causes us to analyze test variants
	// such as "package p as compiled for p.test" or even "for q.test".
	// This leads to multiple distinct ssa.Function instances that
	// represent the same source declaration, and it is essentially
	// impossible to discover this from the SSA representation
	// (since it has lost the connection to go/packages.Package.ID).
	//
	// So, we de-duplicate such variants by position:
	// if any one of them is live, we consider all of them live.
	// (We use Position not Pos to avoid assuming that files common
	// to packages "p" and "p [p.test]" were parsed only once.)
	reachablePosn := make(map[token.Position]bool)
	for fn := range res.Reachable {
		if fn.Pos().IsValid() || fn.Name() == "init" {
			reachablePosn[prog.Fset.Position(fn.Pos())] = true
		}
	}

	// The -whylive=fn flag causes deadcode to explain why a function
	// is not dead, by showing a path to it from some root.
	if *whyLiveFlag != "" {
		targets := make(map[*ssa.Function]bool)
		for _, fn := range sourceFuncs {
			if prettyName(fn, true) == *whyLiveFlag {
				targets[fn] = true
			}
		}
		if len(targets) == 0 {
			// Function is not part of the program.
			//
			// TODO(adonovan): improve the UX here in case
			// of spelling or syntax mistakes. Some ideas:
			// - a cmd/callgraph command to enumerate
			//   available functions.
			// - a deadcode -live flag to compute the complement.
			// - a syntax hint: example.com/pkg.Func or (example.com/pkg.Type).Method
			// - report the element of AllFunctions with the smallest
			//   Levenshtein distance from *whyLiveFlag.
			// - permit -whylive=regexp. But beware of spurious
			//   matches (e.g. fmt.Print matches fmt.Println)
			//   and the annoyance of having to quote parens (*T).f.
			log.Fatalf("function %q not found in program", *whyLiveFlag)
		}

		// Opt: remove the unreachable ones.
		for fn := range targets {
			if !reachablePosn[prog.Fset.Position(fn.Pos())] {
				delete(targets, fn)
			}
		}
		if len(targets) == 0 {
			log.Fatalf("function %s is dead code", *whyLiveFlag)
		}

		res.CallGraph.DeleteSyntheticNodes() // inline synthetic wrappers (except inits)
		root, path := pathSearch(roots, res, targets)
		if root == nil {
			// RTA doesn't add callgraph edges for reflective calls.
			log.Fatalf("%s is reachable only through reflection", *whyLiveFlag)
		}
		if len(path) == 0 {
			// No edges => one of the targets is a root.
			// Rather than (confusingly) print nothing, make this an error.
			log.Fatalf("%s is a root", root.Func)
		}

		// Build a list of jsonEdge records
		// to print as -json or -f=template.
		var edges []any
		for _, edge := range path {
			edges = append(edges, jsonEdge{
				Initial:  cond(len(edges) == 0, prettyName(edge.Caller.Func, true), ""),
				Kind:     cond(isStaticCall(edge), "static", "dynamic"),
				Position: toJSONPosition(prog.Fset.Position(edge.Pos())),
				Callee:   prettyName(edge.Callee.Func, true),
			})
		}
		format := `{{if .Initial}}{{printf "%19s%s\n" "" .Initial}}{{end}}{{printf "%8s@L%.4d --> %s" .Kind .Position.Line .Callee}}`
		if *formatFlag != "" {
			format = *formatFlag
		}
		printObjects(format, edges)
		return
	}

	// Group unreachable functions by package path.
	byPkgPath := make(map[string]map[*ssa.Function]bool)
	for _, fn := range sourceFuncs {
		posn := prog.Fset.Position(fn.Pos())

		if !reachablePosn[posn] {
			reachablePosn[posn] = true // suppress dups with same pos

			pkgpath := fn.Pkg.Pkg.Path()
			m, ok := byPkgPath[pkgpath]
			if !ok {
				m = make(map[*ssa.Function]bool)
				byPkgPath[pkgpath] = m
			}
			m[fn] = true
		}
	}

	// Build array of jsonPackage objects.
	var packages []any
	pkgpaths := keys(byPkgPath)
	sort.Strings(pkgpaths)
	for _, pkgpath := range pkgpaths {
		if !filter.MatchString(pkgpath) {
			continue
		}

		m := byPkgPath[pkgpath]

		// Print functions that appear within the same file in
		// declaration order. This tends to keep related
		// methods such as (T).Marshal and (*T).Unmarshal
		// together better than sorting.
		fns := keys(m)
		sort.Slice(fns, func(i, j int) bool {
			xposn := prog.Fset.Position(fns[i].Pos())
			yposn := prog.Fset.Position(fns[j].Pos())
			if xposn.Filename != yposn.Filename {
				return xposn.Filename < yposn.Filename
			}
			return xposn.Line < yposn.Line
		})

		var functions []jsonFunction
		for _, fn := range fns {
			posn := prog.Fset.Position(fn.Pos())

			// Without -generated, skip functions declared in
			// generated Go files.
			// (Functions called by them may still be reported.)
			gen := generated[posn.Filename]
			if gen && !*generatedFlag {
				continue
			}

			functions = append(functions, jsonFunction{
				Name:      prettyName(fn, false),
				Position:  toJSONPosition(posn),
				Generated: gen,
			})
		}
		if len(functions) > 0 {
			packages = append(packages, jsonPackage{
				Name:  fns[0].Pkg.Pkg.Name(),
				Path:  pkgpath,
				Funcs: functions,
			})
		}
	}

	// Default line-oriented format: "a/b/c.go:1:2: unreachable func: T.f"
	format := `{{range .Funcs}}{{printf "%s: unreachable func: %s\n" .Position .Name}}{{end}}`
	if *formatFlag != "" {
		format = *formatFlag
	}
	printObjects(format, packages)
}

// prettyName is a fork of Function.String designed to reduce
// go/ssa's fussy punctuation symbols, e.g. "(*pkg.T).F" -> "pkg.T.F".
//
// It only works for functions that remain after
// callgraph.Graph.DeleteSyntheticNodes: source-level named functions
// and methods, their anonymous functions, and synthetic package
// initializers.
func prettyName(fn *ssa.Function, qualified bool) string {
	var buf strings.Builder

	// optional package qualifier
	if qualified && fn.Pkg != nil {
		fmt.Fprintf(&buf, "%s.", fn.Pkg.Pkg.Path())
	}

	var format func(*ssa.Function)
	format = func(fn *ssa.Function) {
		// anonymous?
		if fn.Parent() != nil {
			format(fn.Parent())
			i := index(fn.Parent().AnonFuncs, fn)
			fmt.Fprintf(&buf, "$%d", i+1)
			return
		}

		// method receiver?
		if recv := fn.Signature.Recv(); recv != nil {
			_, named := typesinternal.ReceiverNamed(recv)
			buf.WriteString(named.Obj().Name())
			buf.WriteByte('.')
		}

		// function/method name
		buf.WriteString(fn.Name())
	}
	format(fn)

	return buf.String()
}

// printObjects formats an array of objects, either as JSON or using a
// template, following the manner of 'go list (-json|-f=template)'.
func printObjects(format string, objects []any) {
	if *jsonFlag {
		out, err := json.MarshalIndent(objects, "", "\t")
		if err != nil {
			log.Fatalf("internal error: %v", err)
		}
		os.Stdout.Write(out)
		return
	}

	// -f=template. Parse can't fail: we checked it earlier.
	tmpl := template.Must(template.New("deadcode").Parse(format))
	for _, object := range objects {
		var buf bytes.Buffer
		if err := tmpl.Execute(&buf, object); err != nil {
			log.Fatal(err)
		}
		if n := buf.Len(); n == 0 || buf.Bytes()[n-1] != '\n' {
			buf.WriteByte('\n')
		}
		os.Stdout.Write(buf.Bytes())
	}
}

// TODO(adonovan): use go1.21's ast.IsGenerated.

// isGenerated reports whether the file was generated by a program,
// not handwritten, by detecting the special comment described
// at https://go.dev/s/generatedcode.
//
// The syntax tree must have been parsed with the ParseComments flag.
// Example:
//
//	f, err := parser.ParseFile(fset, filename, src, parser.ParseComments|parser.PackageClauseOnly)
//	if err != nil { ... }
//	gen := ast.IsGenerated(f)
func isGenerated(file *ast.File) bool {
	_, ok := generator(file)
	return ok
}

func generator(file *ast.File) (string, bool) {
	for _, group := range file.Comments {
		for _, comment := range group.List {
			if comment.Pos() > file.Package {
				break // after package declaration
			}
			// opt: check Contains first to avoid unnecessary array allocation in Split.
			const prefix = "// Code generated "
			if strings.Contains(comment.Text, prefix) {
				for _, line := range strings.Split(comment.Text, "\n") {
					if rest, ok := strings.CutPrefix(line, prefix); ok {
						if gen, ok := strings.CutSuffix(rest, " DO NOT EDIT."); ok {
							return gen, true
						}
					}
				}
			}
		}
	}
	return "", false
}

// pathSearch returns the shortest path from one of the roots to one
// of the targets (along with the root itself), or zero if no path was found.
func pathSearch(roots []*ssa.Function, res *rta.Result, targets map[*ssa.Function]bool) (*callgraph.Node, []*callgraph.Edge) {
	// Search breadth-first (for shortest path) from the root.
	//
	// We don't use the virtual CallGraph.Root node as we wish to
	// choose the order in which we search entrypoints:
	// non-test packages before test packages,
	// main functions before init functions.

	// Sort roots into preferred order.
	importsTesting := func(fn *ssa.Function) bool {
		isTesting := func(p *types.Package) bool { return p.Path() == "testing" }
		return containsFunc(fn.Pkg.Pkg.Imports(), isTesting)
	}
	sort.Slice(roots, func(i, j int) bool {
		x, y := roots[i], roots[j]
		xtest := importsTesting(x)
		ytest := importsTesting(y)
		if xtest != ytest {
			return !xtest // non-tests before tests
		}
		xinit := x.Name() == "init"
		yinit := y.Name() == "init"
		if xinit != yinit {
			return !xinit // mains before inits
		}
		return false
	})

	search := func(allowDynamic bool) (*callgraph.Node, []*callgraph.Edge) {
		// seen maps each encountered node to its predecessor on the
		// path to a root node, or to nil for root itself.
		seen := make(map[*callgraph.Node]*callgraph.Edge)
		bfs := func(root *callgraph.Node) []*callgraph.Edge {
			queue := []*callgraph.Node{root}
			seen[root] = nil
			for len(queue) > 0 {
				node := queue[0]
				queue = queue[1:]

				// found a path?
				if targets[node.Func] {
					path := []*callgraph.Edge{} // non-nil in case len(path)=0
					for {
						edge := seen[node]
						if edge == nil {
							reverse(path)
							return path
						}
						path = append(path, edge)
						node = edge.Caller
					}
				}

				for _, edge := range node.Out {
					if allowDynamic || isStaticCall(edge) {
						if _, ok := seen[edge.Callee]; !ok {
							seen[edge.Callee] = edge
							queue = append(queue, edge.Callee)
						}
					}
				}
			}
			return nil
		}
		for _, rootFn := range roots {
			root := res.CallGraph.Nodes[rootFn]
			if root == nil {
				// Missing call graph node for root.
				// TODO(adonovan): seems like a bug in rta.
				continue
			}
			if path := bfs(root); path != nil {
				return root, path
			}
		}
		return nil, nil
	}

	for _, allowDynamic := range []bool{false, true} {
		if root, path := search(allowDynamic); path != nil {
			return root, path
		}
	}

	return nil, nil
}

// -- utilities --

func isStaticCall(edge *callgraph.Edge) bool {
	return edge.Site != nil && edge.Site.Common().StaticCallee() != nil
}

var cwd, _ = os.Getwd()

func toJSONPosition(posn token.Position) jsonPosition {
	// Use cwd-relative filename if possible.
	filename := posn.Filename
	if rel, err := filepath.Rel(cwd, filename); err == nil && !strings.HasPrefix(rel, "..") {
		filename = rel
	}

	return jsonPosition{filename, posn.Line, posn.Column}
}

func cond[T any](cond bool, t, f T) T {
	if cond {
		return t
	} else {
		return f
	}
}

// -- output protocol (for JSON or text/template) --

// Keep in sync with doc comment!

type jsonFunction struct {
	Name      string       // name (sans package qualifier)
	Position  jsonPosition // file/line/column of declaration
	Generated bool         // function is declared in a generated .go file
}

func (f jsonFunction) String() string { return f.Name }

type jsonPackage struct {
	Name  string         // declared name
	Path  string         // full import path
	Funcs []jsonFunction // non-empty list of package's dead functions
}

func (p jsonPackage) String() string { return p.Path }

// The Initial and Callee names are package-qualified.
type jsonEdge struct {
	Initial  string `json:",omitempty"` // initial entrypoint (main or init); first edge only
	Kind     string // = static | dynamic
	Position jsonPosition
	Callee   string
}

type jsonPosition struct {
	File      string
	Line, Col int
}

func (p jsonPosition) String() string {
	return fmt.Sprintf("%s:%d:%d", p.File, p.Line, p.Col)
}

// -- from the future --

// TODO(adonovan): use go1.22's slices and maps packages.

func containsFunc[S ~[]E, E any](s S, f func(E) bool) bool {
	return indexFunc(s, f) >= 0
}

func indexFunc[S ~[]E, E any](s S, f func(E) bool) int {
	for i := range s {
		if f(s[i]) {
			return i
		}
	}
	return -1
}

func index[S ~[]E, E comparable](s S, v E) int {
	for i := range s {
		if v == s[i] {
			return i
		}
	}
	return -1
}

func reverse[S ~[]E, E any](s S) {
	for i, j := 0, len(s)-1; i < j; i, j = i+1, j-1 {
		s[i], s[j] = s[j], s[i]
	}
}

func keys[M ~map[K]V, K comparable, V any](m M) []K {
	r := make([]K, 0, len(m))
	for k := range m {
		r = append(r, k)
	}
	return r
}
