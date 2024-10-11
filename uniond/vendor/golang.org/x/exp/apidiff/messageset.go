// TODO: show that two-non-empty dotjoin can happen, by using an anon struct as a field type
// TODO: don't report removed/changed methods for both value and pointer method sets?

package apidiff

import (
	"fmt"
	"go/types"
	"sort"
	"strings"
)

// objectWithSide contains an object, and information on which side (old or new)
// of the comparison it relates to. This matters when need to express the object's
// package path, relative to the root path of the comparison, as the old and new
// sides can have different roots (e.g. comparing somepackage/v2 vs. somepackage/v3).
type objectWithSide struct {
	object types.Object
	isNew  bool
}

// There can be at most one message for each object or part thereof.
// Parts include interface methods and struct fields.
//
// The part thing is necessary. Method (Func) objects have sufficient info, but field
// Vars do not: they just have a field name and a type, without the enclosing struct.
type messageSet map[objectWithSide]map[string]string

// Add a message for obj and part, overwriting a previous message
// (shouldn't happen).
// obj is required but part can be empty.
func (m messageSet) add(obj objectWithSide, part, msg string) {
	s := m[obj]
	if s == nil {
		s = map[string]string{}
		m[obj] = s
	}
	if f, ok := s[part]; ok && f != msg {
		fmt.Printf("! second, different message for obj %s, isNew %v, part %q\n", obj.object, obj.isNew, part)
		fmt.Printf("  first:  %s\n", f)
		fmt.Printf("  second: %s\n", msg)
	}
	s[part] = msg
}

func (m messageSet) collect(oldRootPackagePath, newRootPackagePath string) []string {
	var s []string
	for obj, parts := range m {
		rootPackagePath := oldRootPackagePath
		if obj.isNew {
			rootPackagePath = newRootPackagePath
		}

		// Format each object name relative to its own package.
		objstring := objectString(obj.object, rootPackagePath)
		for part, msg := range parts {
			var p string

			if strings.HasPrefix(part, ",") {
				p = objstring + part
			} else {
				p = dotjoin(objstring, part)
			}
			s = append(s, p+": "+msg)
		}
	}
	sort.Strings(s)
	return s
}

func objectString(obj types.Object, rootPackagePath string) string {
	thisPackagePath := obj.Pkg().Path()

	var packagePrefix string
	if thisPackagePath == rootPackagePath {
		// obj is in same package as the diff operation root - no prefix
		packagePrefix = ""
	} else if strings.HasPrefix(thisPackagePath, rootPackagePath+"/") {
		// obj is in a child package compared to the diff operation root - use a
		// prefix starting with "./" to emphasise the relative nature
		packagePrefix = "./" + thisPackagePath[len(rootPackagePath)+1:] + "."
	} else {
		// obj is outside the diff operation root - display full path. This can
		// happen if there is a need to report a change in a type in an unrelated
		// package, because it has been used as the underlying type in a type
		// definition in the package being processed, for example.
		packagePrefix = thisPackagePath + "."
	}

	if f, ok := obj.(*types.Func); ok {
		sig := f.Type().(*types.Signature)
		if recv := sig.Recv(); recv != nil {
			tn := types.TypeString(recv.Type(), types.RelativeTo(obj.Pkg()))
			if tn[0] == '*' {
				tn = "(" + tn + ")"
			}
			return fmt.Sprintf("%s%s.%s", packagePrefix, tn, obj.Name())
		}
	}
	return fmt.Sprintf("%s%s", packagePrefix, obj.Name())
}

func dotjoin(s1, s2 string) string {
	if s1 == "" {
		return s2
	}
	if s2 == "" {
		return s1
	}
	return s1 + "." + s2
}
