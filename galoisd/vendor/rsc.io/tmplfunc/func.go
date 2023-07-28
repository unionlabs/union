// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package tmplfunc

import (
	"bytes"
	"fmt"
	"reflect"
	"regexp"
	"strings"

	"rsc.io/tmplfunc/internal/parse"

	htmltemplate "html/template"
	texttemplate "text/template"
)

var validNameRE = regexp.MustCompile(`\A[_\pL][_\pL\p{Nd}]*\z`)
var validArgNameRE = regexp.MustCompile(`\A[_\pL][_\pL\p{Nd}]*(\.\.\.|\?)?\z`)

func funcs(t Template, names, texts []string) error {
	var leftDelim, rightDelim string
	switch t := t.(type) {
	case nil:
		return fmt.Errorf("tmplfunc: nil Template")
	default:
		return fmt.Errorf("tmplfunc: non-template type %T", t)
	case *texttemplate.Template:
		leftDelim = reflect.ValueOf(t).Elem().FieldByName("leftDelim").String()
		rightDelim = reflect.ValueOf(t).Elem().FieldByName("rightDelim").String()
	case *htmltemplate.Template:
		leftDelim = reflect.ValueOf(t).Elem().FieldByName("text").Elem().FieldByName("leftDelim").String()
		rightDelim = reflect.ValueOf(t).Elem().FieldByName("text").Elem().FieldByName("rightDelim").String()
	}

	trees := make(map[string]*parse.Tree)
	for i, text := range texts {
		t := parse.New(names[i], nil)
		t.Mode = parse.SkipFuncCheck
		_, err := t.Parse(text, leftDelim, rightDelim, trees)
		if err != nil {
			return err
		}
	}

	// Install functions for named templates as appropriate.
	funcs := make(map[string]interface{})
	for name := range trees {
		if err := addFunc(t, name, funcs); err != nil {
			return err
		}
	}

	switch t := t.(type) {
	case *texttemplate.Template:
		t.Funcs(funcs)
	case *htmltemplate.Template:
		t.Funcs(funcs)
	}

	return nil
}

// Funcs installs functions for all the templates in the set containing t.
// After using t.Clone it is necessary to call Funcs on the result to arrange
// for the functions to invoke the cloned templates and not the originals.
func Funcs(t Template) error {
	funcs := make(map[string]interface{})
	switch t := t.(type) {
	case *texttemplate.Template:
		for _, t1 := range t.Templates() {
			if err := addFunc(t, t1.Name(), funcs); err != nil {
				return err
			}
		}
		t.Funcs(funcs)
	case *htmltemplate.Template:
		for _, t1 := range t.Templates() {
			if err := addFunc(t, t1.Name(), funcs); err != nil {
				return err
			}
		}
		t.Funcs(funcs)
	}
	return nil
}

func addFunc(t Template, name string, funcs map[string]interface{}) error {
	fn, bundle, err := bundler(name)
	if err != nil {
		return err
	}
	if fn == "" {
		return nil
	}
	switch t := t.(type) {
	case *texttemplate.Template:
		funcs[fn] = func(args ...interface{}) (string, error) {
			t := t.Lookup(name)
			if t == nil {
				return "", fmt.Errorf("lost template %q", name)
			}
			arg, err := bundle(args)
			if err != nil {
				return "", err
			}
			var buf bytes.Buffer
			err = t.Execute(&buf, arg)
			if err != nil {
				return "", err
			}
			return buf.String(), nil
		}
	case *htmltemplate.Template:
		funcs[fn] = func(args ...interface{}) (htmltemplate.HTML, error) {
			t := t.Lookup(name)
			if t == nil {
				return "", fmt.Errorf("lost template %q", name)
			}
			arg, err := bundle(args)
			if err != nil {
				return "", err
			}
			var buf bytes.Buffer
			err = t.Execute(&buf, arg)
			if err != nil {
				return "", err
			}
			return htmltemplate.HTML(buf.String()), nil
		}
	}
	return nil
}

func bundler(name string) (fn string, bundle func(args []interface{}) (interface{}, error), err error) {
	f := strings.Fields(name)
	if len(f) == 0 || !validNameRE.MatchString(f[0]) {
		return "", nil, nil
	}

	fn = f[0]
	if len(f) == 1 {
		bundle = func(args []interface{}) (interface{}, error) {
			if len(args) == 0 {
				return nil, nil
			}
			if len(args) == 1 {
				return args[0], nil
			}
			return nil, fmt.Errorf("too many arguments in call to template %s", fn)
		}
	} else {
		sawQ := false
		for i, argName := range f[1:] {
			if !validArgNameRE.MatchString(argName) {
				return "", nil, fmt.Errorf("invalid template name %q: invalid argument name %s", name, argName)
			}
			if strings.HasSuffix(argName, "...") {
				if i != len(f)-2 {
					return "", nil, fmt.Errorf("invalid template name %q: %s is not last argument", name, argName)
				}
				break
			}
			if strings.HasSuffix(argName, "?") {
				sawQ = true
				continue
			}
			if sawQ {
				return "", nil, fmt.Errorf("invalid template name %q: required %s after optional %s", name, argName, f[i])
			}
		}

		bundle = func(args []interface{}) (interface{}, error) {
			m := make(map[string]interface{})
			for _, argName := range f[1:] {
				if strings.HasSuffix(argName, "...") {
					m[strings.TrimSuffix(argName, "...")] = args
					args = nil
					break
				}
				if strings.HasSuffix(argName, "?") {
					prefix := strings.TrimSuffix(argName, "?")
					if len(args) == 0 {
						m[prefix] = nil
					} else {
						m[prefix], args = args[0], args[1:]
					}
					continue
				}
				if len(args) == 0 {
					return nil, fmt.Errorf("too few arguments in call to template %s", fn)
				}
				m[argName], args = args[0], args[1:]
			}
			if len(args) > 0 {
				return nil, fmt.Errorf("too many arguments in call to template %s", fn)
			}
			return m, nil
		}
	}

	return fn, bundle, nil
}
