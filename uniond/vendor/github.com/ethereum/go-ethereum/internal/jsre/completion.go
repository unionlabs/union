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
	"regexp"
	"sort"
	"strings"

	"github.com/dop251/goja"
)

// JS numerical token
var numerical = regexp.MustCompile(`^(NaN|-?((\d*\.\d+|\d+)([Ee][+-]?\d+)?|Infinity))$`)

// CompleteKeywords returns potential continuations for the given line. Since line is
// evaluated, callers need to make sure that evaluating line does not have side effects.
func (jsre *JSRE) CompleteKeywords(line string) []string {
	var results []string
	jsre.Do(func(vm *goja.Runtime) {
		results = getCompletions(vm, line)
	})
	return results
}

func getCompletions(vm *goja.Runtime, line string) (results []string) {
	parts := strings.Split(line, ".")
	if len(parts) == 0 {
		return nil
	}

	// Find the right-most fully named object in the line. e.g. if line = "x.y.z"
	// and "x.y" is an object, obj will reference "x.y".
	obj := vm.GlobalObject()
	for i := 0; i < len(parts)-1; i++ {
		if numerical.MatchString(parts[i]) {
			return nil
		}
		v := obj.Get(parts[i])
		if v == nil || goja.IsNull(v) || goja.IsUndefined(v) {
			return nil // No object was found
		}
		obj = v.ToObject(vm)
	}

	// Go over the keys of the object and retain the keys matching prefix.
	// Example: if line = "x.y.z" and "x.y" exists and has keys "zebu", "zebra"
	// and "platypus", then "x.y.zebu" and "x.y.zebra" will be added to results.
	prefix := parts[len(parts)-1]
	iterOwnAndConstructorKeys(vm, obj, func(k string) {
		if strings.HasPrefix(k, prefix) {
			if len(parts) == 1 {
				results = append(results, k)
			} else {
				results = append(results, strings.Join(parts[:len(parts)-1], ".")+"."+k)
			}
		}
	})

	// Append opening parenthesis (for functions) or dot (for objects)
	// if the line itself is the only completion.
	if len(results) == 1 && results[0] == line {
		// Accessing the property will cause it to be evaluated.
		// This can cause an error, e.g. in case of web3.eth.protocolVersion
		// which has been dropped from geth. Ignore the error for autocompletion
		// purposes.
		obj := SafeGet(obj, parts[len(parts)-1])
		if obj != nil {
			if _, isfunc := goja.AssertFunction(obj); isfunc {
				results[0] += "("
			} else {
				results[0] += "."
			}
		}
	}

	sort.Strings(results)
	return results
}
