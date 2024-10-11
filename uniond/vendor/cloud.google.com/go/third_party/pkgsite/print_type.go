// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package pkgsite is not for external use. May change at any time without
// warning.
//
// Copied from
// https://github.com/golang/pkgsite/blob/ff1e697b104e751da362159cf6c7743898eea3fe/internal/fetch/dochtml/internal/render/
// and
// https://github.com/golang/pkgsite/tree/88f8a28ab2102416529d05d11e8135a43e146d46/internal/fetch/dochtml.
package pkgsite

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/doc"
	"go/printer"
	"go/scanner"
	"go/token"
	"strconv"
	"strings"
)

// PrintType returns a string representation of the decl.
//
// PrintType works by:
//  1. Generate a map from every identifier to a URL for the identifier (or no
//     URL if the identifier shouldn't link).
//  2. ast.Inspect the decl to get an ordered slice of every identifier to the
//     link for it, using the map from step 1.
//  3. Print out the plain doc for the decl.
//  4. Use scanner.Scanner to find every identifier (in the same order as step
//     2). If there is a link for the identifier, insert it. Otherwise, print
//     the plain doc.
func PrintType(fset *token.FileSet, decl ast.Decl, toURL func(string, string) string, topLevelDecls map[interface{}]bool) string {
	anchorLinksMap := generateAnchorLinks(decl, toURL, topLevelDecls)
	// Convert the map (keyed by *ast.Ident) to a slice of URLs (or no URL).
	//
	// This relies on the ast.Inspect and scanner.Scanner both
	// visiting *ast.Ident and token.IDENT nodes in the same order.
	var anchorLinks []string
	ast.Inspect(decl, func(node ast.Node) bool {
		if id, ok := node.(*ast.Ident); ok {
			anchorLinks = append(anchorLinks, anchorLinksMap[id])
		}
		return true
	})

	v := &declVisitor{}
	ast.Walk(v, decl)

	var b bytes.Buffer
	p := printer.Config{Mode: printer.UseSpaces | printer.TabIndent, Tabwidth: 4}
	p.Fprint(&b, fset, &printer.CommentedNode{Node: decl, Comments: v.Comments})
	src := b.Bytes()
	var out strings.Builder

	fakeFset := token.NewFileSet()
	file := fakeFset.AddFile("", fakeFset.Base(), b.Len())

	var lastOffset int // last src offset copied to output buffer
	var s scanner.Scanner
	s.Init(file, src, nil, scanner.ScanComments)
	identIdx := 0
scan:
	for {
		p, tok, lit := s.Scan()
		line := file.Line(p) - 1 // current 0-indexed line number
		offset := file.Offset(p) // current offset into source file

		// Add traversed bytes from src to the appropriate line.
		prevLines := strings.SplitAfter(string(src[lastOffset:offset]), "\n")
		for i, ln := range prevLines {
			n := line - len(prevLines) + i + 1
			if n < 0 { // possible at EOF
				n = 0
			}
			out.WriteString(ln)
		}

		lastOffset = offset
		switch tok {
		case token.EOF:
			break scan
		case token.IDENT:
			if identIdx < len(anchorLinks) && anchorLinks[identIdx] != "" {
				fmt.Fprintf(&out, `<a href="%s">%s</a>`, anchorLinks[identIdx], lit)
			} else {
				out.WriteString(lit)
			}
			identIdx++
			lastOffset += len(lit)
		}
	}
	return out.String()
}

// declVisitor is used to walk over the AST and trim large string
// literals and arrays before package documentation is rendered.
// Comments are added to Comments to indicate that a part of the
// original code is not displayed.
type declVisitor struct {
	Comments []*ast.CommentGroup
}

// Visit implements ast.Visitor.
func (v *declVisitor) Visit(n ast.Node) ast.Visitor {
	switch n := n.(type) {
	case *ast.BasicLit:
		if n.Kind == token.STRING && len(n.Value) > 128 {
			v.Comments = append(v.Comments,
				&ast.CommentGroup{List: []*ast.Comment{{
					Slash: n.Pos(),
					Text:  stringBasicLitSize(n.Value),
				}}})
			n.Value = `""`
		}
	case *ast.CompositeLit:
		if len(n.Elts) > 100 {
			v.Comments = append(v.Comments,
				&ast.CommentGroup{List: []*ast.Comment{{
					Slash: n.Lbrace,
					Text:  fmt.Sprintf("/* %d elements not displayed */", len(n.Elts)),
				}}})
			n.Elts = n.Elts[:0]
		}
	}
	return v
}

// stringBasicLitSize computes the number of bytes in the given string basic literal.
//
// See noder.basicLit and syntax.StringLit cases in cmd/compile/internal/gc/noder.go.
func stringBasicLitSize(s string) string {
	if len(s) > 0 && s[0] == '`' {
		// strip carriage returns from raw string
		s = strings.ReplaceAll(s, "\r", "")
	}
	u, err := strconv.Unquote(s)
	if err != nil {
		return fmt.Sprintf("/* invalid %d byte string literal not displayed */", len(s))
	}
	return fmt.Sprintf("/* %d byte string literal not displayed */", len(u))
}

// generateAnchorLinks returns a mapping of *ast.Ident objects to the URL
// that the identifier should link to.
func generateAnchorLinks(decl ast.Decl, toURL func(string, string) string, topLevelDecls map[interface{}]bool) map[*ast.Ident]string {
	m := map[*ast.Ident]string{}
	ignore := map[ast.Node]bool{}
	ast.Inspect(decl, func(node ast.Node) bool {
		if ignore[node] {
			return false
		}
		switch node := node.(type) {
		case *ast.SelectorExpr:
			// Package qualified identifier (e.g., "io.EOF").
			if prefix, _ := node.X.(*ast.Ident); prefix != nil {
				if obj := prefix.Obj; obj != nil && obj.Kind == ast.Pkg {
					if spec, _ := obj.Decl.(*ast.ImportSpec); spec != nil {
						if path, err := strconv.Unquote(spec.Path.Value); err == nil {
							// Register two links, one for the package
							// and one for the qualified identifier.
							m[prefix] = toURL(path, "")
							m[node.Sel] = toURL(path, node.Sel.Name)
							return false
						}
					}
				}
			}
		case *ast.Ident:
			if node.Obj == nil && doc.IsPredeclared(node.Name) {
				m[node] = toURL("builtin", node.Name)
			} else if node.Obj != nil && topLevelDecls[node.Obj.Decl] {
				m[node] = toURL("", node.Name)
			}
		case *ast.FuncDecl:
			ignore[node.Name] = true // E.g., "func NoLink() int"
		case *ast.TypeSpec:
			ignore[node.Name] = true // E.g., "type NoLink int"
		case *ast.ValueSpec:
			for _, n := range node.Names {
				ignore[n] = true // E.g., "var NoLink1, NoLink2 int"
			}
		case *ast.AssignStmt:
			for _, n := range node.Lhs {
				ignore[n] = true // E.g., "NoLink1, NoLink2 := 0, 1"
			}
		}
		return true
	})
	return m
}

// TopLevelDecls returns the top level declarations in the package.
func TopLevelDecls(pkg *doc.Package) map[interface{}]bool {
	topLevelDecls := map[interface{}]bool{}
	forEachPackageDecl(pkg, func(decl ast.Decl) {
		topLevelDecls[decl] = true
		if gd, _ := decl.(*ast.GenDecl); gd != nil {
			for _, sp := range gd.Specs {
				topLevelDecls[sp] = true
			}
		}
	})
	return topLevelDecls
}

// forEachPackageDecl iterates though every top-level declaration in a package.
func forEachPackageDecl(pkg *doc.Package, fnc func(decl ast.Decl)) {
	for _, c := range pkg.Consts {
		fnc(c.Decl)
	}
	for _, v := range pkg.Vars {
		fnc(v.Decl)
	}
	for _, f := range pkg.Funcs {
		fnc(f.Decl)
	}
	for _, t := range pkg.Types {
		fnc(t.Decl)
		for _, c := range t.Consts {
			fnc(c.Decl)
		}
		for _, v := range t.Vars {
			fnc(v.Decl)
		}
		for _, f := range t.Funcs {
			fnc(f.Decl)
		}
		for _, m := range t.Methods {
			fnc(m.Decl)
		}
	}
}
