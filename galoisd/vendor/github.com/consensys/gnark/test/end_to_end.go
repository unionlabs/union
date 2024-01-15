package test

import (
	"reflect"
	"strings"

	"github.com/consensys/gnark/frontend"
)

// hollow takes a gnark circuit and removes all the witness data. The resulting circuit can be used for compilation purposes
// Its purpose is to make testing more convenient. For example, as opposed to SolvingSucceeded(circuit, assignment),
// one can write SolvingSucceeded(hollow(assignment), assignment), obviating the creation of a separate circuit object.
func hollow(c frontend.Circuit) frontend.Circuit {
	cV := reflect.ValueOf(c).Elem()
	t := reflect.TypeOf(c).Elem()
	res := reflect.New(t) // a new object of the same type as c
	resE := res.Elem()
	resC := res.Interface().(frontend.Circuit)

	frontendVar := reflect.TypeOf((*frontend.Variable)(nil)).Elem()

	for i := 0; i < t.NumField(); i++ {
		fieldT := t.Field(i).Type
		if fieldT.Kind() == reflect.Slice && fieldT.Elem().Implements(frontendVar) { // create empty slices for witness slices
			resE.Field(i).Set(reflect.ValueOf(make([]frontend.Variable, cV.Field(i).Len())))
		} else if fieldT != frontendVar { // copy non-witness variables
			resE.Field(i).Set(cV.Field(i))
		}
	}

	return resC
}

func removePackageName(s string) string {
	return s[strings.LastIndex(s, ".")+1:]
}
