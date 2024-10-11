/*Package opt provides common go-cmp.Options for use with assert.DeepEqual.
 */
package opt // import "gotest.tools/v3/assert/opt"

import (
	"fmt"
	"reflect"
	"strings"
	"time"

	gocmp "github.com/google/go-cmp/cmp"
)

// DurationWithThreshold returns a [gocmp.Comparer] for comparing [time.Duration]. The
// Comparer returns true if the difference between the two Duration values is
// within the threshold and neither value is zero.
func DurationWithThreshold(threshold time.Duration) gocmp.Option {
	return gocmp.Comparer(cmpDuration(threshold))
}

func cmpDuration(threshold time.Duration) func(x, y time.Duration) bool {
	return func(x, y time.Duration) bool {
		if x == 0 || y == 0 {
			return false
		}
		delta := x - y
		return delta <= threshold && delta >= -threshold
	}
}

// TimeWithThreshold returns a [gocmp.Comparer] for comparing [time.Time]. The
// Comparer returns true if the difference between the two Time values is
// within the threshold and neither value is zero.
func TimeWithThreshold(threshold time.Duration) gocmp.Option {
	return gocmp.Comparer(cmpTime(threshold))
}

func cmpTime(threshold time.Duration) func(x, y time.Time) bool {
	return func(x, y time.Time) bool {
		if x.IsZero() || y.IsZero() {
			return false
		}
		delta := x.Sub(y)
		return delta <= threshold && delta >= -threshold
	}
}

// PathString is a [gocmp.FilterPath] filter that returns true when path.String()
// matches any of the specs.
//
// The path spec is a dot separated string where each segment is a field name.
// Slices, Arrays, and Maps are always matched against every element in the
// sequence. [gocmp.Indirect], [gocmp.Transform], and [gocmp.TypeAssertion] are always
// ignored.
//
// Note: this path filter is not type safe. Incorrect paths will be silently
// ignored. Consider using a type safe path filter for more complex paths.
func PathString(specs ...string) func(path gocmp.Path) bool {
	return func(path gocmp.Path) bool {
		for _, spec := range specs {
			if path.String() == spec {
				return true
			}
		}
		return false
	}
}

// PathDebug is a [gocmp.FilterPath] filter that always returns false. It prints
// each path it receives. It can be used to debug path matching problems.
func PathDebug(path gocmp.Path) bool {
	fmt.Printf("PATH string=%s gostring=%s\n", path, path.GoString())
	for _, step := range path {
		fmt.Printf("  STEP %s\ttype=%s\t%s\n",
			formatStepType(step), step.Type(), stepTypeFields(step))
	}
	return false
}

func formatStepType(step gocmp.PathStep) string {
	return strings.Title(strings.TrimPrefix(reflect.TypeOf(step).String(), "*cmp."))
}

func stepTypeFields(step gocmp.PathStep) string {
	switch typed := step.(type) {
	case gocmp.StructField:
		return fmt.Sprintf("name=%s", typed.Name())
	case gocmp.MapIndex:
		return fmt.Sprintf("key=%s", typed.Key().Interface())
	case gocmp.Transform:
		return fmt.Sprintf("name=%s", typed.Name())
	case gocmp.SliceIndex:
		return fmt.Sprintf("name=%d", typed.Key())
	}
	return ""
}

// PathField is a [gocmp.FilterPath] filter that matches a struct field by name.
// PathField will match every instance of the field in a recursive or nested
// structure.
func PathField(structType interface{}, field string) func(gocmp.Path) bool {
	typ := reflect.TypeOf(structType)
	if typ.Kind() != reflect.Struct {
		panic(fmt.Sprintf("type %s is not a struct", typ))
	}
	if _, ok := typ.FieldByName(field); !ok {
		panic(fmt.Sprintf("type %s does not have field %s", typ, field))
	}

	return func(path gocmp.Path) bool {
		return path.Index(-2).Type() == typ && isStructField(path.Index(-1), field)
	}
}

func isStructField(step gocmp.PathStep, name string) bool {
	field, ok := step.(gocmp.StructField)
	return ok && field.Name() == name
}
