//go:build windows

// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

import (
	"bufio"
	"bytes"
	"errors"
	"flag"
	"fmt"
	"go/format"
	"go/parser"
	"go/token"
	"io"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"text/template"

	"golang.org/x/sys/windows"
)

const (
	pkgSyscall = "syscall"
	pkgWindows = "windows"

	// common types.

	tBool    = "bool"
	tBoolPtr = "*bool"
	tError   = "error"
	tString  = "string"

	// error variable names.

	varErr         = "err"
	varErrNTStatus = "ntStatus"
	varErrHR       = "hr"
)

var (
	filename       = flag.String("output", "", "output file name (standard output if omitted)")
	printTraceFlag = flag.Bool("trace", false, "generate print statement after every syscall")
	systemDLL      = flag.Bool("systemdll", true, "whether all DLLs should be loaded from the Windows system directory")
	winio          = flag.Bool("winio", false, `import this package ("github.com/Microsoft/go-winio")`)
	utf16          = flag.Bool("utf16", true, "encode string arguments as UTF-16 for syscalls not ending in 'A' or 'W'")
	sortdecls      = flag.Bool("sort", true, "sort DLL and function declarations")
)

func trim(s string) string {
	return strings.Trim(s, " \t")
}

func endsIn(s string, c byte) bool {
	return len(s) >= 1 && s[len(s)-1] == c
}

var packageName string

func packagename() string {
	return packageName
}

func windowsdot() string {
	if packageName == pkgWindows {
		return ""
	}
	return pkgWindows + "."
}

func syscalldot() string {
	if packageName == pkgSyscall {
		return ""
	}
	return pkgSyscall + "."
}

// Param is function parameter.
type Param struct {
	Name      string
	Type      string
	fn        *Fn
	tmpVarIdx int
}

// tmpVar returns temp variable name that will be used to represent p during syscall.
func (p *Param) tmpVar() string {
	if p.tmpVarIdx < 0 {
		p.tmpVarIdx = p.fn.curTmpVarIdx
		p.fn.curTmpVarIdx++
	}
	return fmt.Sprintf("_p%d", p.tmpVarIdx)
}

// BoolTmpVarCode returns source code for bool temp variable.
func (p *Param) BoolTmpVarCode() string {
	const code = `var %[1]s uint32
	if %[2]s {
		%[1]s = 1
	}`
	return fmt.Sprintf(code, p.tmpVar(), p.Name)
}

// BoolPointerTmpVarCode returns source code for bool temp variable.
func (p *Param) BoolPointerTmpVarCode() string {
	const code = `var %[1]s uint32
	if *%[2]s {
		%[1]s = 1
	}`
	return fmt.Sprintf(code, p.tmpVar(), p.Name)
}

// SliceTmpVarCode returns source code for slice temp variable.
func (p *Param) SliceTmpVarCode() string {
	const code = `var %s *%s
	if len(%s) > 0 {
		%s = &%s[0]
	}`
	tmp := p.tmpVar()
	return fmt.Sprintf(code, tmp, p.Type[2:], p.Name, tmp, p.Name)
}

// StringTmpVarCode returns source code for string temp variable.
func (p *Param) StringTmpVarCode() string {
	errvar := p.fn.Rets.ErrorVarName()
	if errvar == "" {
		errvar = "_"
	}
	tmp := p.tmpVar()
	const code = `var %s %s
	%s, %s = %s(%s)`
	s := fmt.Sprintf(code, tmp, p.fn.StrconvType(), tmp, errvar, p.fn.StrconvFunc(), p.Name)
	if errvar == "-" {
		return s
	}
	const morecode = `
	if %s != nil {
		return
	}`
	return s + fmt.Sprintf(morecode, errvar)
}

// TmpVarCode returns source code for temp variable.
func (p *Param) TmpVarCode() string {
	switch {
	case p.Type == tBool:
		return p.BoolTmpVarCode()
	case p.Type == tBoolPtr:
		return p.BoolPointerTmpVarCode()
	case strings.HasPrefix(p.Type, "[]"):
		return p.SliceTmpVarCode()
	default:
		return ""
	}
}

// TmpVarReadbackCode returns source code for reading back the temp variable into the original variable.
func (p *Param) TmpVarReadbackCode() string {
	switch {
	case p.Type == tBoolPtr:
		return fmt.Sprintf("*%s = %s != 0", p.Name, p.tmpVar())
	default:
		return ""
	}
}

// TmpVarHelperCode returns source code for helper's temp variable.
func (p *Param) TmpVarHelperCode() string {
	if p.Type != "string" {
		return ""
	}
	return p.StringTmpVarCode()
}

// SyscallArgList returns source code fragments representing p parameter
// in syscall. Slices are translated into 2 syscall parameters: pointer to
// the first element and length.
func (p *Param) SyscallArgList() []string {
	t := p.HelperType()
	var s string
	switch {
	case t == tBoolPtr:
		s = fmt.Sprintf("unsafe.Pointer(&%s)", p.tmpVar())
	case t[0] == '*':
		s = fmt.Sprintf("unsafe.Pointer(%s)", p.Name)
	case t == tBool:
		s = p.tmpVar()
	case strings.HasPrefix(t, "[]"):
		return []string{
			fmt.Sprintf("uintptr(unsafe.Pointer(%s))", p.tmpVar()),
			fmt.Sprintf("uintptr(len(%s))", p.Name),
		}
	default:
		s = p.Name
	}
	return []string{fmt.Sprintf("uintptr(%s)", s)}
}

// IsError determines if p parameter is used to return error.
func (p *Param) IsError() bool {
	return p.Name == varErr && p.Type == tError
}

// HelperType returns type of parameter p used in helper function.
func (p *Param) HelperType() string {
	if p.Type == tString {
		return p.fn.StrconvType()
	}
	return p.Type
}

// join concatenates parameters ps into a string with sep separator.
// Each parameter is converted into string by applying fn to it
// before conversion.
func join(ps []*Param, fn func(*Param) string, sep string) string {
	if len(ps) == 0 {
		return ""
	}
	a := make([]string, 0)
	for _, p := range ps {
		a = append(a, fn(p))
	}
	return strings.Join(a, sep)
}

// Rets describes function return parameters.
type Rets struct {
	Name          string
	Type          string
	ReturnsError  bool
	FailCond      string
	fnMaybeAbsent bool
}

// ErrorVarName returns error variable name for r.
func (r *Rets) ErrorVarName() string {
	if r.ReturnsError {
		return varErr
	}
	if r.Type == tError {
		return r.Name
	}
	return ""
}

// ToParams converts r into slice of *Param.
func (r *Rets) ToParams() []*Param {
	ps := make([]*Param, 0)
	if len(r.Name) > 0 {
		ps = append(ps, &Param{Name: r.Name, Type: r.Type})
	}
	if r.ReturnsError {
		ps = append(ps, &Param{Name: varErr, Type: tError})
	}
	return ps
}

// List returns source code of syscall return parameters.
func (r *Rets) List() string {
	s := join(r.ToParams(), func(p *Param) string { return p.Name + " " + p.Type }, ", ")
	if len(s) > 0 {
		s = "(" + s + ")"
	} else if r.fnMaybeAbsent {
		s = "(err error)"
	}
	return s
}

// PrintList returns source code of trace printing part correspondent
// to syscall return values.
func (r *Rets) PrintList() string {
	return join(r.ToParams(), func(p *Param) string { return fmt.Sprintf(`"%s=", %s, `, p.Name, p.Name) }, `", ", `)
}

// SetReturnValuesCode returns source code that accepts syscall return values.
func (r *Rets) SetReturnValuesCode() string {
	if r.Name == "" && !r.ReturnsError {
		return ""
	}
	retvar := "r0"
	if r.Name == "" {
		retvar = "r1"
	}
	errvar := "_"
	if r.ReturnsError {
		errvar = "e1"
	}
	return fmt.Sprintf("%s, _, %s := ", retvar, errvar)
}

func (r *Rets) useLongHandleErrorCode(retvar string) string {
	const code = `if %s {
		err = errnoErr(e1)
	}`
	cond := retvar + " == 0"
	if r.FailCond != "" {
		cond = strings.Replace(r.FailCond, "failretval", retvar, 1)
	}
	return fmt.Sprintf(code, cond)
}

// SetErrorCode returns source code that sets return parameters.
func (r *Rets) SetErrorCode() string {
	const code = `if r0 != 0 {
		%s = %sErrno(r0)
	}`
	const ntStatus = `if r0 != 0 {
		%s = %sNTStatus(r0)
	}`
	const hrCode = `if int32(r0) < 0 {
		if r0&0x1fff0000 == 0x00070000 {
			r0 &= 0xffff
		}
		%s = %sErrno(r0)
	}`

	if r.Name == "" && !r.ReturnsError {
		return ""
	}
	if r.Name == "" {
		return r.useLongHandleErrorCode("r1")
	}
	if r.Type == tError {
		switch r.Name {
		case varErrNTStatus, strings.ToLower(varErrNTStatus): // allow ntstatus to work
			return fmt.Sprintf(ntStatus, r.Name, windowsdot())
		case varErrHR:
			return fmt.Sprintf(hrCode, r.Name, syscalldot())
		default:
			return fmt.Sprintf(code, r.Name, syscalldot())
		}
	}

	var s string
	switch {
	case r.Type[0] == '*':
		s = fmt.Sprintf("%s = (%s)(unsafe.Pointer(r0))", r.Name, r.Type)
	case r.Type == tBool:
		s = fmt.Sprintf("%s = r0 != 0", r.Name)
	default:
		s = fmt.Sprintf("%s = %s(r0)", r.Name, r.Type)
	}
	if !r.ReturnsError {
		return s
	}
	return s + "\n\t" + r.useLongHandleErrorCode(r.Name)
}

// Fn describes syscall function.
type Fn struct {
	Name        string
	Params      []*Param
	Rets        *Rets
	PrintTrace  bool
	dllname     string
	dllfuncname string
	src         string
	// TODO: get rid of this field and just use parameter index instead
	curTmpVarIdx int // insure tmp variables have uniq names
}

// extractParams parses s to extract function parameters.
func extractParams(s string, f *Fn) ([]*Param, error) {
	s = trim(s)
	if s == "" {
		return nil, nil
	}
	a := strings.Split(s, ",")
	ps := make([]*Param, len(a))
	for i := range ps {
		s2 := trim(a[i])
		b := strings.Split(s2, " ")
		if len(b) != 2 {
			b = strings.Split(s2, "\t")
			if len(b) != 2 {
				return nil, errors.New("Could not extract function parameter from \"" + s2 + "\"")
			}
		}
		ps[i] = &Param{
			Name:      trim(b[0]),
			Type:      trim(b[1]),
			fn:        f,
			tmpVarIdx: -1,
		}
	}
	return ps, nil
}

// extractSection extracts text out of string s starting after start
// and ending just before end. found return value will indicate success,
// and prefix, body and suffix will contain correspondent parts of string s.
func extractSection(s string, start, end rune) (prefix, body, suffix string, found bool) {
	s = trim(s)
	if strings.HasPrefix(s, string(start)) {
		// no prefix
		body = s[1:]
	} else {
		a := strings.SplitN(s, string(start), 2)
		if len(a) != 2 {
			return "", "", s, false
		}
		prefix = a[0]
		body = a[1]
	}
	a := strings.SplitN(body, string(end), 2)
	if len(a) != 2 {
		return "", "", "", false
	}
	return prefix, a[0], a[1], true
}

// newFn parses string s and return created function Fn.
func newFn(s string) (*Fn, error) {
	s = trim(s)
	f := &Fn{
		Rets:       &Rets{},
		src:        s,
		PrintTrace: *printTraceFlag,
	}
	// function name and args
	prefix, body, s, found := extractSection(s, '(', ')')
	if !found || prefix == "" {
		return nil, errors.New("Could not extract function name and parameters from \"" + f.src + "\"")
	}
	f.Name = prefix
	var err error
	f.Params, err = extractParams(body, f)
	if err != nil {
		return nil, err
	}
	// return values
	_, body, s, found = extractSection(s, '(', ')')
	if found {
		r, err := extractParams(body, f)
		if err != nil {
			return nil, err
		}
		switch len(r) {
		case 0:
		case 1:
			if r[0].IsError() {
				f.Rets.ReturnsError = true
			} else {
				f.Rets.Name = r[0].Name
				f.Rets.Type = r[0].Type
			}
		case 2:
			if !r[1].IsError() {
				return nil, errors.New("Only last windows error is allowed as second return value in \"" + f.src + "\"")
			}
			f.Rets.ReturnsError = true
			f.Rets.Name = r[0].Name
			f.Rets.Type = r[0].Type
		default:
			return nil, errors.New("Too many return values in \"" + f.src + "\"")
		}
	}
	// fail condition
	_, body, s, found = extractSection(s, '[', ']')
	if found {
		f.Rets.FailCond = body
	}
	// dll and dll function names
	s = trim(s)
	if s == "" {
		return f, nil
	}
	if !strings.HasPrefix(s, "=") {
		return nil, errors.New("Could not extract dll name from \"" + f.src + "\"")
	}
	s = trim(s[1:])
	if i := strings.LastIndex(s, "."); i >= 0 {
		f.dllname = s[:i]
		f.dllfuncname = s[i+1:]
	} else {
		f.dllfuncname = s
	}
	if f.dllfuncname == "" {
		return nil, fmt.Errorf("function name is not specified in %q", s)
	}
	if n := f.dllfuncname; endsIn(n, '?') {
		f.dllfuncname = n[:len(n)-1]
		f.Rets.fnMaybeAbsent = true
	}
	return f, nil
}

// DLLName returns DLL name for function f.
func (f *Fn) DLLName() string {
	if f.dllname == "" {
		return "kernel32"
	}
	return f.dllname
}

// DLLVar returns a valid Go identifier that represents DLLName.
func (f *Fn) DLLVar() string {
	id := strings.Map(func(r rune) rune {
		switch r {
		case '.', '-':
			return '_'
		default:
			return r
		}
	}, f.DLLName())
	if !token.IsIdentifier(id) {
		panic(fmt.Errorf("could not create Go identifier for DLLName %q", f.DLLName()))
	}
	return id
}

// DLLFuncName returns DLL function name for function f.
func (f *Fn) DLLFuncName() string {
	if f.dllfuncname == "" {
		return f.Name
	}
	return f.dllfuncname
}

// ParamList returns source code for function f parameters.
func (f *Fn) ParamList() string {
	return join(f.Params, func(p *Param) string { return p.Name + " " + p.Type }, ", ")
}

// HelperParamList returns source code for helper function f parameters.
func (f *Fn) HelperParamList() string {
	return join(f.Params, func(p *Param) string { return p.Name + " " + p.HelperType() }, ", ")
}

// ParamPrintList returns source code of trace printing part correspondent
// to syscall input parameters.
func (f *Fn) ParamPrintList() string {
	return join(f.Params, func(p *Param) string { return fmt.Sprintf(`"%s=", %s, `, p.Name, p.Name) }, `", ", `)
}

// ParamCount return number of syscall parameters for function f.
func (f *Fn) ParamCount() int {
	n := 0
	for _, p := range f.Params {
		n += len(p.SyscallArgList())
	}
	return n
}

// SyscallParamCount determines which version of Syscall/Syscall6/Syscall9/...
// to use. It returns parameter count for correspondent SyscallX function.
func (f *Fn) SyscallParamCount() int {
	n := f.ParamCount()
	switch {
	case n <= 3:
		return 3
	case n <= 6:
		return 6
	case n <= 9:
		return 9
	case n <= 12:
		return 12
	case n <= 15:
		return 15
	default:
		panic("too many arguments to system call")
	}
}

// Syscall determines which SyscallX function to use for function f.
func (f *Fn) Syscall() string {
	c := f.SyscallParamCount()
	if c == 3 {
		return syscalldot() + "Syscall"
	}
	return syscalldot() + "Syscall" + strconv.Itoa(c)
}

// SyscallParamList returns source code for SyscallX parameters for function f.
func (f *Fn) SyscallParamList() string {
	a := make([]string, 0)
	for _, p := range f.Params {
		a = append(a, p.SyscallArgList()...)
	}
	for len(a) < f.SyscallParamCount() {
		a = append(a, "0")
	}
	return strings.Join(a, ", ")
}

// HelperCallParamList returns source code of call into function f helper.
func (f *Fn) HelperCallParamList() string {
	a := make([]string, 0, len(f.Params))
	for _, p := range f.Params {
		s := p.Name
		if p.Type == tString {
			s = p.tmpVar()
		}
		a = append(a, s)
	}
	return strings.Join(a, ", ")
}

// MaybeAbsent returns source code for handling functions that are possibly unavailable.
func (f *Fn) MaybeAbsent() string {
	if !f.Rets.fnMaybeAbsent {
		return ""
	}
	const code = `%[1]s = proc%[2]s.Find()
	if %[1]s != nil {
		return
	}`
	errorVar := f.Rets.ErrorVarName()
	if errorVar == "" {
		errorVar = varErr
	}
	return fmt.Sprintf(code, errorVar, f.DLLFuncName())
}

// IsUTF16 is true, if f is W (UTF-16) function and false for all A (ASCII) functions.
// Functions ending in neither will default to UTF-16, unless the `-utf16` flag is set
// to `false`.
func (f *Fn) IsUTF16() bool {
	s := f.DLLFuncName()
	return endsIn(s, 'W') || (*utf16 && !endsIn(s, 'A'))
}

// StrconvFunc returns name of Go string to OS string function for f.
func (f *Fn) StrconvFunc() string {
	if f.IsUTF16() {
		return syscalldot() + "UTF16PtrFromString"
	}
	return syscalldot() + "BytePtrFromString"
}

// StrconvType returns Go type name used for OS string for f.
func (f *Fn) StrconvType() string {
	if f.IsUTF16() {
		return "*uint16"
	}
	return "*byte"
}

// HasStringParam is true, if f has at least one string parameter.
// Otherwise it is false.
func (f *Fn) HasStringParam() bool {
	for _, p := range f.Params {
		if p.Type == tString {
			return true
		}
	}
	return false
}

// HelperName returns name of function f helper.
func (f *Fn) HelperName() string {
	if !f.HasStringParam() {
		return f.Name
	}
	return "_" + f.Name
}

// DLL is a DLL's filename and a string that is valid in a Go identifier that should be used when
// naming a variable that refers to the DLL.
type DLL struct {
	Name string
	Var  string
}

// Source files and functions.
type Source struct {
	Funcs           []*Fn
	DLLFuncNames    []*Fn
	Files           []string
	StdLibImports   []string
	ExternalImports []string
}

func (src *Source) Import(pkg string) {
	src.StdLibImports = append(src.StdLibImports, pkg)
	sort.Strings(src.StdLibImports)
}

func (src *Source) ExternalImport(pkg string) {
	src.ExternalImports = append(src.ExternalImports, pkg)
	sort.Strings(src.ExternalImports)
}

// ParseFiles parses files listed in fs and extracts all syscall
// functions listed in sys comments. It returns source files
// and functions collection *Source if successful.
func ParseFiles(fs []string) (*Source, error) {
	src := &Source{
		Funcs: make([]*Fn, 0),
		Files: make([]string, 0),
		StdLibImports: []string{
			"unsafe",
		},
		ExternalImports: make([]string, 0),
	}
	for _, file := range fs {
		if err := src.ParseFile(file); err != nil {
			return nil, err
		}
	}
	src.DLLFuncNames = make([]*Fn, 0, len(src.Funcs))
	uniq := make(map[string]bool, len(src.Funcs))
	for _, fn := range src.Funcs {
		name := fn.DLLFuncName()
		if !uniq[name] {
			src.DLLFuncNames = append(src.DLLFuncNames, fn)
			uniq[name] = true
		}
	}
	return src, nil
}

// DLLs return dll names for a source set src.
func (src *Source) DLLs() []DLL {
	uniq := make(map[string]bool)
	r := make([]DLL, 0)
	for _, f := range src.Funcs {
		id := f.DLLVar()
		if _, found := uniq[id]; !found {
			uniq[id] = true
			r = append(r, DLL{f.DLLName(), id})
		}
	}
	if *sortdecls {
		sort.Slice(r, func(i, j int) bool {
			return r[i].Var < r[j].Var
		})
	}
	return r
}

// ParseFile adds additional file (or files, if path is a glob pattern) path to a source set src.
func (src *Source) ParseFile(path string) error {
	file, err := os.Open(path)
	if err == nil {
		defer file.Close()
		return src.parseFile(file)
	} else if !(errors.Is(err, os.ErrNotExist) || errors.Is(err, windows.ERROR_INVALID_NAME)) {
		return err
	}

	paths, err := filepath.Glob(path)
	if err != nil {
		return err
	}

	for _, path := range paths {
		file, err := os.Open(path)
		if err != nil {
			return err
		}
		err = src.parseFile(file)
		file.Close()
		if err != nil {
			return err
		}
	}

	return nil
}

func (src *Source) parseFile(file *os.File) error {
	s := bufio.NewScanner(file)
	for s.Scan() {
		t := trim(s.Text())
		if len(t) < 7 {
			continue
		}
		if !strings.HasPrefix(t, "//sys") {
			continue
		}
		t = t[5:]
		if !(t[0] == ' ' || t[0] == '\t') {
			continue
		}
		f, err := newFn(t[1:])
		if err != nil {
			return err
		}
		src.Funcs = append(src.Funcs, f)
	}
	if err := s.Err(); err != nil {
		return err
	}
	src.Files = append(src.Files, file.Name())
	if *sortdecls {
		sort.Slice(src.Funcs, func(i, j int) bool {
			fi, fj := src.Funcs[i], src.Funcs[j]
			if fi.DLLName() == fj.DLLName() {
				return fi.DLLFuncName() < fj.DLLFuncName()
			}
			return fi.DLLName() < fj.DLLName()
		})
	}

	// get package name
	fset := token.NewFileSet()
	_, err := file.Seek(0, 0)
	if err != nil {
		return err
	}
	pkg, err := parser.ParseFile(fset, "", file, parser.PackageClauseOnly)
	if err != nil {
		return err
	}
	packageName = pkg.Name.Name

	return nil
}

// IsStdRepo reports whether src is part of standard library.
func (src *Source) IsStdRepo() (bool, error) {
	if len(src.Files) == 0 {
		return false, errors.New("no input files provided")
	}
	abspath, err := filepath.Abs(src.Files[0])
	if err != nil {
		return false, err
	}
	goroot := runtime.GOROOT()
	if runtime.GOOS == "windows" {
		abspath = strings.ToLower(abspath)
		goroot = strings.ToLower(goroot)
	}
	sep := string(os.PathSeparator)
	if !strings.HasSuffix(goroot, sep) {
		goroot += sep
	}
	return strings.HasPrefix(abspath, goroot), nil
}

// Generate output source file from a source set src.
func (src *Source) Generate(w io.Writer) error {
	const (
		pkgStd         = iota // any package in std library
		pkgXSysWindows        // x/sys/windows package
		pkgOther
	)
	isStdRepo, err := src.IsStdRepo()
	if err != nil {
		return err
	}
	var pkgtype int
	switch {
	case isStdRepo:
		pkgtype = pkgStd
	case packageName == "windows":
		// TODO: this needs better logic than just using package name
		pkgtype = pkgXSysWindows
	default:
		pkgtype = pkgOther
	}
	if *systemDLL {
		switch pkgtype {
		case pkgStd:
			src.Import("internal/syscall/windows/sysdll")
		case pkgXSysWindows:
		default:
			src.ExternalImport("golang.org/x/sys/windows")
		}
	}
	if *winio {
		src.ExternalImport("github.com/Microsoft/go-winio")
	}
	if packageName != "syscall" {
		src.Import("syscall")
	}
	funcMap := template.FuncMap{
		"packagename": packagename,
		"syscalldot":  syscalldot,
		"newlazydll": func(dll string) string {
			arg := "\"" + dll + ".dll\""
			if !*systemDLL {
				return syscalldot() + "NewLazyDLL(" + arg + ")"
			}
			if strings.HasPrefix(dll, "api_") || strings.HasPrefix(dll, "ext_") {
				arg = strings.Replace(arg, "_", "-", -1)
			}
			switch pkgtype {
			case pkgStd:
				return syscalldot() + "NewLazyDLL(sysdll.Add(" + arg + "))"
			case pkgXSysWindows:
				return "NewLazySystemDLL(" + arg + ")"
			default:
				return "windows.NewLazySystemDLL(" + arg + ")"
			}
		},
	}
	t := template.Must(template.New("main").Funcs(funcMap).Parse(srcTemplate))
	err = t.Execute(w, src)
	if err != nil {
		return errors.New("Failed to execute template: " + err.Error())
	}
	return nil
}

func writeTempSourceFile(data []byte) (string, error) {
	f, err := os.CreateTemp("", "mkwinsyscall-generated-*.go")
	if err != nil {
		return "", err
	}
	_, err = f.Write(data)
	if closeErr := f.Close(); err == nil {
		err = closeErr
	}
	if err != nil {
		os.Remove(f.Name()) // best effort
		return "", err
	}
	return f.Name(), nil
}

func usage() {
	fmt.Fprintf(os.Stderr, "usage: mkwinsyscall [flags] [path ...]\n")
	flag.PrintDefaults()
	os.Exit(1)
}

func main() {
	flag.Usage = usage
	flag.Parse()
	if len(flag.Args()) <= 0 {
		fmt.Fprintf(os.Stderr, "no files to parse provided\n")
		usage()
	}

	src, err := ParseFiles(flag.Args())
	if err != nil {
		log.Fatal(err)
	}

	var buf bytes.Buffer
	if err := src.Generate(&buf); err != nil {
		log.Fatal(err)
	}

	data, err := format.Source(buf.Bytes())
	if err != nil {
		log.Printf("failed to format source: %v", err)
		f, err := writeTempSourceFile(buf.Bytes())
		if err != nil {
			log.Fatalf("failed to write unformatted source to file: %v", err)
		}
		log.Fatalf("for diagnosis, wrote unformatted source to %v", f)
	}
	if *filename == "" {
		_, err = os.Stdout.Write(data)
	} else {
		//nolint:gosec // G306: code file, no need for wants 0600
		err = os.WriteFile(*filename, data, 0644)
	}
	if err != nil {
		log.Fatal(err)
	}
}

// TODO: use println instead to print in the following template

const srcTemplate = `
{{define "main"}} //go:build windows

// Code generated by 'go generate' using "github.com/Microsoft/go-winio/tools/mkwinsyscall"; DO NOT EDIT.

package {{packagename}}

import (
{{range .StdLibImports}}"{{.}}"
{{end}}

{{range .ExternalImports}}"{{.}}"
{{end}}
)

var _ unsafe.Pointer

// Do the interface allocations only once for common
// Errno values.
const (
	errnoERROR_IO_PENDING = 997
)

var (
	errERROR_IO_PENDING error = {{syscalldot}}Errno(errnoERROR_IO_PENDING)
	errERROR_EINVAL error     = {{syscalldot}}EINVAL
)

// errnoErr returns common boxed Errno values, to prevent
// allocations at runtime.
func errnoErr(e {{syscalldot}}Errno) error {
	switch e {
	case 0:
		return errERROR_EINVAL
	case errnoERROR_IO_PENDING:
		return errERROR_IO_PENDING
	}
	// TODO: add more here, after collecting data on the common
	// error values see on Windows. (perhaps when running
	// all.bat?)
	return e
}

var (
{{template "dlls" .}}
{{template "funcnames" .}})
{{range .Funcs}}{{if .HasStringParam}}{{template "helperbody" .}}{{end}}{{template "funcbody" .}}{{end}}
{{end}}

{{/* help functions */}}

{{define "dlls"}}{{range .DLLs}}	mod{{.Var}} = {{newlazydll .Name}}
{{end}}{{end}}

{{define "funcnames"}}{{range .DLLFuncNames}}	proc{{.DLLFuncName}} = mod{{.DLLVar}}.NewProc("{{.DLLFuncName}}")
{{end}}{{end}}

{{define "helperbody"}}
func {{.Name}}({{.ParamList}}) {{template "results" .}}{
{{template "helpertmpvars" .}}	return {{.HelperName}}({{.HelperCallParamList}})
}
{{end}}

{{define "funcbody"}}
func {{.HelperName}}({{.HelperParamList}}) {{template "results" .}}{
{{template "maybeabsent" .}}	{{template "tmpvars" .}}	{{template "syscall" .}}	{{template "tmpvarsreadback" .}}
{{template "seterror" .}}{{template "printtrace" .}}	return
}
{{end}}

{{define "helpertmpvars"}}{{range .Params}}{{if .TmpVarHelperCode}}	{{.TmpVarHelperCode}}
{{end}}{{end}}{{end}}

{{define "maybeabsent"}}{{if .MaybeAbsent}}{{.MaybeAbsent}}
{{end}}{{end}}

{{define "tmpvars"}}{{range .Params}}{{if .TmpVarCode}}	{{.TmpVarCode}}
{{end}}{{end}}{{end}}

{{define "results"}}{{if .Rets.List}}{{.Rets.List}} {{end}}{{end}}

{{define "syscall"}}{{.Rets.SetReturnValuesCode}}{{.Syscall}}(proc{{.DLLFuncName}}.Addr(), {{.ParamCount}}, {{.SyscallParamList}}){{end}}

{{define "tmpvarsreadback"}}{{range .Params}}{{if .TmpVarReadbackCode}}
{{.TmpVarReadbackCode}}{{end}}{{end}}{{end}}

{{define "seterror"}}{{if .Rets.SetErrorCode}}	{{.Rets.SetErrorCode}}
{{end}}{{end}}

{{define "printtrace"}}{{if .PrintTrace}}	print("SYSCALL: {{.Name}}(", {{.ParamPrintList}}") (", {{.Rets.PrintList}}")\n")
{{end}}{{end}}

`
