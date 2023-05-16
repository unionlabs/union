// reflectwalk is a package that allows you to "walk" complex structures
// similar to how you may "walk" a filesystem: visiting every element one
// by one and calling callback functions allowing you to handle and manipulate
// those elements.
//
// this fork is derived from Mitchell Hashimoto work (see LICENSE file)
// https://github.com/mitchellh/reflectwalk
//
// it is adapted to maximize performance in the context of gnark circuits and remove
// un-needed features.
package reflectwalk

import (
	"errors"
	"fmt"
	"reflect"
)

// InterfaceWalker implementations are able to handle interface values as they
// are encountered during the walk.
type InterfaceWalker interface {
	Interface(reflect.Value) error
}

// SliceWalker implementations are able to handle slice elements found
// within complex structures.
type SliceWalker interface {
	Slice(reflect.Value) error
	SliceElem(int, reflect.Value) error
}

// ArrayWalker implementations are able to handle array elements found
// within complex structures.
type ArrayWalker interface {
	Array(reflect.Value) error
	ArrayElem(int, reflect.Value) error
}

// StructWalker is an interface that has methods that are called for
// structs when a Walk is done.
type StructWalker interface {
	Struct(reflect.Value) error
	StructField(reflect.StructField, reflect.Value) error
}

// EnterExitWalker implementations are notified before and after
// they walk deeper into complex structures (into struct fields,
// into slice elements, etc.)
type EnterExitWalker interface {
	Enter(Location) error
	Exit(Location) error
}

// PointerValueWalker implementations are notified with the value of
// a particular pointer when a pointer is walked. Pointer is called
// right before PointerEnter.
type PointerValueWalker interface {
	Pointer(reflect.Value) error
}

// ErrSkipEntry can be returned from walk functions to skip walking
// the value of this field. This is only valid in the following functions:
//
//   - Struct: skips all fields from being walked
//   - StructField: skips walking the struct value
var ErrSkipEntry = errors.New("skip this entry")

// Walk takes an arbitrary value and an interface and traverses the
// value, calling callbacks on the interface if they are supported.
// The interface should implement one or more of the walker interfaces
// in this package, such as PrimitiveWalker, StructWalker, etc.
func Walk(data, walker interface{}) (err error) {
	v := reflect.ValueOf(data)
	ew, ok := walker.(EnterExitWalker)
	if ok {
		err = ew.Enter(WalkLoc)
	}

	if err == nil {
		err = walk(v, walker)
	}

	if ok && err == nil {
		err = ew.Exit(WalkLoc)
	}

	return
}

func walk(v reflect.Value, w interface{}) (err error) {
	// Determine if we're receiving a pointer and if so notify the walker.
	// The logic here is convoluted but very important (tests will fail if
	// almost any part is changed). I will try to explain here.
	//
	// First, we check if the value is an interface, if so, we really need
	// to check the interface's VALUE to see whether it is a pointer.
	//
	// Check whether the value is then a pointer. If so, then set pointer
	// to true to notify the user.
	//
	// If we still have a pointer or an interface after the indirections, then
	// we unwrap another level
	//
	// At this time, we also set "v" to be the dereferenced value. This is
	// because once we've unwrapped the pointer we want to use that value.
	pointer := false
	pointerV := v

	for {
		if pointerV.Kind() == reflect.Interface {
			if iw, ok := w.(InterfaceWalker); ok {
				if err = iw.Interface(pointerV); err != nil {
					if err == ErrSkipEntry {
						// Skip the rest of this entry but clear the error
						return nil
					}
					return
				}
			}

			pointerV = pointerV.Elem()
		}

		if pointerV.Kind() == reflect.Ptr {
			if pw, ok := w.(PointerValueWalker); ok {
				if err = pw.Pointer(pointerV); err != nil {
					if err == ErrSkipEntry {
						// Skip the rest of this entry but clear the error
						return nil
					}

					return
				}
			}
			pointer = true
			v = reflect.Indirect(pointerV)
		}

		if pointer {
			pointerV = v
		}
		pointer = false

		// If we still have a pointer or interface we have to indirect another level.
		switch pointerV.Kind() {
		case reflect.Ptr, reflect.Interface:
			continue
		}
		break
	}

	if v.Kind() == reflect.Interface {
		v = v.Elem()
	}

	k := v.Kind()
	if k >= reflect.Int && k <= reflect.Complex128 {
		k = reflect.Int
	}

	switch k {
	case reflect.Slice:
		err = walkSlice(v, w)
		return
	case reflect.Struct:
		err = walkStruct(v, w)
		return
	case reflect.Array:
		err = walkArray(v, w)
		return
	case reflect.Bool, reflect.Chan, reflect.Func, reflect.Int, reflect.String, reflect.Invalid, reflect.Map:
		err = nil
		return
	default:
		panic("unsupported type: " + k.String())
	}
}

func walkSlice(v reflect.Value, w interface{}) (err error) {
	ew, ok := w.(EnterExitWalker)
	if ok {
		ew.Enter(Slice)
	}

	if sw, ok := w.(SliceWalker); ok {
		if err := sw.Slice(v); err != nil {
			return err
		}
	}

	for i := 0; i < v.Len(); i++ {
		elem := v.Index(i)

		if sw, ok := w.(SliceWalker); ok {
			if err := sw.SliceElem(i, elem); err != nil {
				return err
			}
		}

		ew, ok := w.(EnterExitWalker)
		if ok {
			ew.Enter(SliceElem)
		}

		if err := walk(elem, w); err != nil && err != ErrSkipEntry {
			return err
		}

		if ok {
			ew.Exit(SliceElem)
		}
	}

	ew, ok = w.(EnterExitWalker)
	if ok {
		ew.Exit(Slice)
	}

	return nil
}

func walkArray(v reflect.Value, w interface{}) (err error) {
	ew, ok := w.(EnterExitWalker)
	if ok {
		ew.Enter(Array)
	}

	if aw, ok := w.(ArrayWalker); ok {
		if err := aw.Array(v); err != nil {
			return err
		}
	}

	for i := 0; i < v.Len(); i++ {
		elem := v.Index(i)

		if aw, ok := w.(ArrayWalker); ok {
			if err := aw.ArrayElem(i, elem); err != nil {
				return err
			}
		}

		ew, ok := w.(EnterExitWalker)
		if ok {
			ew.Enter(ArrayElem)
		}

		if err := walk(elem, w); err != nil && err != ErrSkipEntry {
			return err
		}

		if ok {
			ew.Exit(ArrayElem)
		}
	}

	ew, ok = w.(EnterExitWalker)
	if ok {
		ew.Exit(Array)
	}

	return nil
}

func walkStruct(v reflect.Value, w interface{}) (err error) {
	ew, ewok := w.(EnterExitWalker)
	if ewok {
		ew.Enter(Struct)
	}

	skip := false
	if sw, ok := w.(StructWalker); ok {
		err = sw.Struct(v)
		if err == ErrSkipEntry {
			skip = true
			err = nil
		}
		if err != nil {
			return
		}
	}

	if !skip {
		vt := v.Type()
		for i := 0; i < vt.NumField(); i++ {
			sf := vt.Field(i)
			f := v.FieldByIndex([]int{i})
			if sf.Anonymous { // TODO @gbotrel check this
				err = walk(f, w)
				if err != nil && err != ErrSkipEntry {
					return
				}
				continue
			}

			if sw, ok := w.(StructWalker); ok {
				err = sw.StructField(sf, f)

				// SkipEntry just pretends this field doesn't even exist
				if err == ErrSkipEntry {
					continue
				}

				if err != nil {
					return
				}
			}

			ew, ok := w.(EnterExitWalker)
			if ok {
				ew.Enter(StructField)
			}

			err = walk(f, w)
			if err != nil && err != ErrSkipEntry {
				return
			}

			if ok {
				ew.Exit(StructField)
			}
		}
	}

	if ewok {
		ew.Exit(Struct)
	}

	return nil
}

type Location uint

const (
	None Location = iota
	Map
	MapKey
	MapValue
	Slice
	SliceElem
	Array
	ArrayElem
	Struct
	StructField
	WalkLoc
)

// generated by stringer -type=Location location.go
const _Location_name = "NoneMapMapKeyMapValueSliceSliceElemArrayArrayElemStructStructFieldWalkLoc"

var _Location_index = [...]uint8{0, 4, 7, 13, 21, 26, 35, 40, 49, 55, 66, 73}

func (i Location) String() string {
	if i >= Location(len(_Location_index)-1) {
		return fmt.Sprintf("Location(%d)", i)
	}
	return _Location_name[_Location_index[i]:_Location_index[i+1]]
}
