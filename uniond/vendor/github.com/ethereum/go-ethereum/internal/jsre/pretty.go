// Copyright 2016 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package jsre

import (
	"fmt"
	"io"
	"reflect"
	"sort"
	"strconv"
	"strings"

	"github.com/dop251/goja"
	"github.com/fatih/color"
)

const (
	maxPrettyPrintLevel = 3
	indentString        = "  "
)

var (
	FunctionColor = color.New(color.FgMagenta).SprintfFunc()
	SpecialColor  = color.New(color.Bold).SprintfFunc()
	NumberColor   = color.New(color.FgRed).SprintfFunc()
	StringColor   = color.New(color.FgGreen).SprintfFunc()
	ErrorColor    = color.New(color.FgHiRed).SprintfFunc()
)

// these fields are hidden when printing objects.
var boringKeys = map[string]bool{
	"valueOf":              true,
	"toString":             true,
	"toLocaleString":       true,
	"hasOwnProperty":       true,
	"isPrototypeOf":        true,
	"propertyIsEnumerable": true,
	"constructor":          true,
}

// prettyPrint writes value to standard output.
func prettyPrint(vm *goja.Runtime, value goja.Value, w io.Writer) {
	ppctx{vm: vm, w: w}.printValue(value, 0, false)
}

// prettyError writes err to standard output.
func prettyError(vm *goja.Runtime, err error, w io.Writer) {
	failure := err.Error()
	if gojaErr, ok := err.(*goja.Exception); ok {
		failure = gojaErr.String()
	}
	fmt.Fprint(w, ErrorColor("%s", failure))
}

func (re *JSRE) prettyPrintJS(call goja.FunctionCall) goja.Value {
	for _, v := range call.Arguments {
		prettyPrint(re.vm, v, re.output)
		fmt.Fprintln(re.output)
	}
	return goja.Undefined()
}

type ppctx struct {
	vm *goja.Runtime
	w  io.Writer
}

func (ctx ppctx) indent(level int) string {
	return strings.Repeat(indentString, level)
}

func (ctx ppctx) printValue(v goja.Value, level int, inArray bool) {
	if goja.IsNull(v) || goja.IsUndefined(v) {
		fmt.Fprint(ctx.w, SpecialColor(v.String()))
		return
	}
	kind := v.ExportType().Kind()
	switch {
	case kind == reflect.Bool:
		fmt.Fprint(ctx.w, SpecialColor("%t", v.ToBoolean()))
	case kind == reflect.String:
		fmt.Fprint(ctx.w, StringColor("%q", v.String()))
	case kind >= reflect.Int && kind <= reflect.Complex128:
		fmt.Fprint(ctx.w, NumberColor("%s", v.String()))
	default:
		if obj, ok := v.(*goja.Object); ok {
			ctx.printObject(obj, level, inArray)
		} else {
			fmt.Fprintf(ctx.w, "<unprintable %T>", v)
		}
	}
}

// SafeGet attempt to get the value associated to `key`, and
// catches the panic that goja creates if an error occurs in
// key.
func SafeGet(obj *goja.Object, key string) (ret goja.Value) {
	defer func() {
		if r := recover(); r != nil {
			ret = goja.Undefined()
		}
	}()
	ret = obj.Get(key)

	return ret
}

func (ctx ppctx) printObject(obj *goja.Object, level int, inArray bool) {
	switch obj.ClassName() {
	case "Array", "GoArray":
		lv := obj.Get("length")
		len := lv.ToInteger()
		if len == 0 {
			fmt.Fprintf(ctx.w, "[]")
			return
		}
		if level > maxPrettyPrintLevel {
			fmt.Fprint(ctx.w, "[...]")
			return
		}
		fmt.Fprint(ctx.w, "[")
		for i := int64(0); i < len; i++ {
			el := obj.Get(strconv.FormatInt(i, 10))
			if el != nil {
				ctx.printValue(el, level+1, true)
			}
			if i < len-1 {
				fmt.Fprintf(ctx.w, ", ")
			}
		}
		fmt.Fprint(ctx.w, "]")

	case "Object":
		// Print values from bignumber.js as regular numbers.
		if ctx.isBigNumber(obj) {
			fmt.Fprint(ctx.w, NumberColor("%s", toString(obj)))
			return
		}
		// Otherwise, print all fields indented, but stop if we're too deep.
		keys := ctx.fields(obj)
		if len(keys) == 0 {
			fmt.Fprint(ctx.w, "{}")
			return
		}
		if level > maxPrettyPrintLevel {
			fmt.Fprint(ctx.w, "{...}")
			return
		}
		fmt.Fprintln(ctx.w, "{")
		for i, k := range keys {
			v := SafeGet(obj, k)
			fmt.Fprintf(ctx.w, "%s%s: ", ctx.indent(level+1), k)
			ctx.printValue(v, level+1, false)
			if i < len(keys)-1 {
				fmt.Fprintf(ctx.w, ",")
			}
			fmt.Fprintln(ctx.w)
		}
		if inArray {
			level--
		}
		fmt.Fprintf(ctx.w, "%s}", ctx.indent(level))

	case "Function":
		robj := obj.ToString()
		desc := strings.Trim(strings.Split(robj.String(), "{")[0], " \t\n")
		desc = strings.Replace(desc, " (", "(", 1)
		fmt.Fprint(ctx.w, FunctionColor("%s", desc))

	case "RegExp":
		fmt.Fprint(ctx.w, StringColor("%s", toString(obj)))

	default:
		if level <= maxPrettyPrintLevel {
			s := obj.ToString().String()
			fmt.Fprintf(ctx.w, "<%s %s>", obj.ClassName(), s)
		} else {
			fmt.Fprintf(ctx.w, "<%s>", obj.ClassName())
		}
	}
}

func (ctx ppctx) fields(obj *goja.Object) []string {
	var (
		vals, methods []string
		seen          = make(map[string]bool)
	)
	add := func(k string) {
		if seen[k] || boringKeys[k] || strings.HasPrefix(k, "_") {
			return
		}
		seen[k] = true

		key := SafeGet(obj, k)
		if key == nil {
			// The value corresponding to that key could not be found
			// (typically because it is backed by an RPC call that is
			// not supported by this instance.  Add it to the list of
			// values so that it appears as `undefined` to the user.
			vals = append(vals, k)
		} else {
			if _, callable := goja.AssertFunction(key); callable {
				methods = append(methods, k)
			} else {
				vals = append(vals, k)
			}
		}
	}
	iterOwnAndConstructorKeys(ctx.vm, obj, add)
	sort.Strings(vals)
	sort.Strings(methods)
	return append(vals, methods...)
}

func iterOwnAndConstructorKeys(vm *goja.Runtime, obj *goja.Object, f func(string)) {
	seen := make(map[string]bool)
	iterOwnKeys(vm, obj, func(prop string) {
		seen[prop] = true
		f(prop)
	})
	if cp := constructorPrototype(vm, obj); cp != nil {
		iterOwnKeys(vm, cp, func(prop string) {
			if !seen[prop] {
				f(prop)
			}
		})
	}
}

func iterOwnKeys(vm *goja.Runtime, obj *goja.Object, f func(string)) {
	Object := vm.Get("Object").ToObject(vm)
	getOwnPropertyNames, isFunc := goja.AssertFunction(Object.Get("getOwnPropertyNames"))
	if !isFunc {
		panic(vm.ToValue("Object.getOwnPropertyNames isn't a function"))
	}
	rv, err := getOwnPropertyNames(goja.Null(), obj)
	if err != nil {
		panic(vm.ToValue(fmt.Sprintf("Error getting object properties: %v", err)))
	}
	gv := rv.Export()
	switch gv := gv.(type) {
	case []interface{}:
		for _, v := range gv {
			f(v.(string))
		}
	case []string:
		for _, v := range gv {
			f(v)
		}
	default:
		panic(fmt.Errorf("Object.getOwnPropertyNames returned unexpected type %T", gv))
	}
}

func (ctx ppctx) isBigNumber(v *goja.Object) bool {
	// Handle numbers with custom constructor.
	if obj := v.Get("constructor").ToObject(ctx.vm); obj != nil {
		if strings.HasPrefix(toString(obj), "function BigNumber") {
			return true
		}
	}
	// Handle default constructor.
	BigNumber := ctx.vm.Get("BigNumber").ToObject(ctx.vm)
	if BigNumber == nil {
		return false
	}
	prototype := BigNumber.Get("prototype").ToObject(ctx.vm)
	isPrototypeOf, callable := goja.AssertFunction(prototype.Get("isPrototypeOf"))
	if !callable {
		return false
	}
	bv, _ := isPrototypeOf(prototype, v)
	return bv.ToBoolean()
}

func toString(obj *goja.Object) string {
	return obj.ToString().String()
}

func constructorPrototype(vm *goja.Runtime, obj *goja.Object) *goja.Object {
	if v := obj.Get("constructor"); v != nil {
		if v := v.ToObject(vm).Get("prototype"); v != nil {
			return v.ToObject(vm)
		}
	}
	return nil
}
