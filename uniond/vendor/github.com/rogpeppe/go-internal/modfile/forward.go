// Package modfile implements parsing and formatting for go.mod files.
//
// This is now just a simple forwarding layer over golang.org/x/mod/modfile
// apart from the ParseGopkgIn function which doesn't exist there.
//
// See that package for documentation.
//
// Deprecated: use [golang.org/x/mod/modfile] instead.
package modfile

import (
	"golang.org/x/mod/modfile"
)

type Position = modfile.Position
type Expr = modfile.Expr
type Comment = modfile.Comment
type Comments = modfile.Comments
type FileSyntax = modfile.FileSyntax
type CommentBlock = modfile.CommentBlock
type Line = modfile.Line
type LineBlock = modfile.LineBlock
type LParen = modfile.LParen
type RParen = modfile.RParen
type File = modfile.File
type Module = modfile.Module
type Go = modfile.Go
type Require = modfile.Require
type Exclude = modfile.Exclude
type Replace = modfile.Replace
type VersionFixer = modfile.VersionFixer

func Format(f *FileSyntax) []byte {
	return modfile.Format(f)
}

func ModulePath(mod []byte) string {
	return modfile.ModulePath(mod)
}

func Parse(file string, data []byte, fix VersionFixer) (*File, error) {
	return modfile.Parse(file, data, fix)
}

func ParseLax(file string, data []byte, fix VersionFixer) (*File, error) {
	return modfile.ParseLax(file, data, fix)
}

func IsDirectoryPath(ns string) bool {
	return modfile.IsDirectoryPath(ns)
}

func MustQuote(s string) bool {
	return modfile.MustQuote(s)
}

func AutoQuote(s string) string {
	return modfile.AutoQuote(s)
}
