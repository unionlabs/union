package schema

import (
	"fmt"
	"reflect"
	"strconv"
	"strings"

	"github.com/consensys/gnark/frontend/schema/internal/reflectwalk"
)

// Walk walks through the provided object and stops when it encounters objects of type tLeaf
//
// It returns the number of secret and public leafs encountered during the walk.
func Walk(circuit interface{}, tLeaf reflect.Type, handler LeafHandler) (count LeafCount, err error) {
	w := walker{
		target:      tLeaf,
		targetSlice: reflect.SliceOf(tLeaf),
		handler:     handler,
	}
	err = reflectwalk.Walk(circuit, &w)
	if err == reflectwalk.ErrSkipEntry {
		err = nil
	}
	count.Public = w.nbPublic
	count.Secret = w.nbSecret
	return
}

// walker implements the interfaces defined in internal/reflectwalk
//
// for example;
//
//	StructWalker is an interface that has methods that are called for
//	structs when a Walk is done.
//	type StructWalker interface {
//		Struct(reflect.Value) error
//		StructField(reflect.StructField, reflect.Value) error
//	}
type walker struct {
	handler            LeafHandler
	target             reflect.Type
	targetSlice        reflect.Type
	path               pathStack
	nbPublic, nbSecret int
}

// Interface handles interface values as they are encountered during the walk.
// That's where we handle leaves.
func (w *walker) Interface(value reflect.Value) error {
	if value.Type() != w.target {
		// keep walking.
		return nil
	}
	v := w.visibility()
	if v == Unset {
		v = Secret
	}

	// call the handler.
	if w.handler != nil {
		if err := w.handler(LeafInfo{Visibility: v, FullName: w.name, name: ""}, value); err != nil {
			return err
		}
	}

	if v == Secret {
		w.nbSecret++
	} else if v == Public {
		w.nbPublic++
	}

	// we return SkipEntry here; the walk will not explore further this object (indirections, ...)
	return reflectwalk.ErrSkipEntry
}

func (w *walker) Pointer(value reflect.Value) error {
	return w.Interface(value)
}

// Slice handles slice elements found within complex structures.
func (w *walker) Slice(value reflect.Value) error {
	if value.Type() == w.targetSlice {
		if value.Len() == 0 {
			fmt.Printf("ignoring uninitialized slice: %s %s\n", w.name(), reflect.SliceOf(w.target).String())
			return nil
		}
		return w.handleLeaves(value)
	}

	return nil
}

func (w *walker) arraySliceElem(index int, v reflect.Value) error {
	w.path.push(LeafInfo{Visibility: w.visibility(), name: strconv.Itoa(index)})
	if v.CanAddr() && v.Addr().CanInterface() {
		// TODO @gbotrel don't like that hook, undesirable side effects
		// will be hard to detect; (for example calling Parse multiple times will init multiple times!)
		//
		// ivokub: completely agree, I have had to work around quite a lot in
		// field emulation to "deinitialize" the elements. Maybe we can have a
		// destructor/deinit hook also?
		value := v.Addr().Interface()
		if ih, hasInitHook := value.(InitHook); hasInitHook {
			ih.GnarkInitHook()
		}
	}
	return nil
}

func (w *walker) SliceElem(index int, v reflect.Value) error {
	return w.arraySliceElem(index, v)
}

// Array handles array elements found within complex structures.
func (w *walker) Array(value reflect.Value) error {
	if value.Type() == reflect.ArrayOf(value.Len(), w.target) {
		return w.handleLeaves(value)
	}
	return nil
}
func (w *walker) ArrayElem(index int, v reflect.Value) error {
	return w.arraySliceElem(index, v)
}

// process an array or slice of leaves; since it's quite common to have large array/slices
// of frontend.Variable, this speeds up considerably performance.
func (w *walker) handleLeaves(value reflect.Value) error {
	v := w.visibility()
	if v == Unset {
		v = Secret
	}

	// call the handler.
	if w.handler != nil {
		n := w.name()
		for i := 0; i < value.Len(); i++ {
			fName := func() string {
				return n + "_" + strconv.Itoa(i)
			}
			vv := value.Index(i)
			if err := w.handler(LeafInfo{Visibility: v, FullName: fName, name: ""}, vv); err != nil {
				return err
			}
		}
	}

	if v == Secret {
		w.nbSecret += value.Len()
	} else if v == Public {
		w.nbPublic += value.Len()
	}

	return reflectwalk.ErrSkipEntry
}

func (w *walker) Struct(reflect.Value) error {
	return nil
}

func (w *walker) StructField(sf reflect.StructField, v reflect.Value) error {
	// check if the gnark tag is set
	tag, ok := sf.Tag.Lookup(string(tagKey))
	if ok && tag == string(TagOptOmit) {
		return reflectwalk.ErrSkipEntry // skipping "-"
	}

	if v.CanAddr() && v.Addr().CanInterface() {
		// TODO @gbotrel don't like that hook, undesirable side effects
		// will be hard to detect; (for example calling Parse multiple times will init multiple times!)
		//
		// ivokub: completely agree, I have had to work around quite a lot in
		// field emulation to "deinitialize" the elements. Maybe we can have a
		// destructor/deinit hook also?
		value := v.Addr().Interface()
		if ih, hasInitHook := value.(InitHook); hasInitHook {
			ih.GnarkInitHook()
		}
	}

	// default visibility: parent (or unset)
	parentVisibility := w.visibility()
	info := LeafInfo{
		name:       sf.Name,
		Visibility: parentVisibility,
	}

	var nameInTag string

	if ok && tag != "" {
		// gnark tag is set
		var opts tagOptions
		nameInTag, opts = parseTag(tag)
		if !isValidTag(nameInTag) {
			nameInTag = ""
		}
		if nameInTag != "" {
			info.name = nameInTag
		}
		opts = tagOptions(strings.TrimSpace(string(opts)))
		switch {
		case opts.contains(TagOptSecret):
			info.Visibility = Secret
		case opts.contains(TagOptPublic):
			info.Visibility = Public
		}
	}

	if parentVisibility != Unset && parentVisibility != info.Visibility {
		parentName := w.name()
		if parentName == "" {
			parentName = info.name
		} else {
			parentName += "_" + info.name
		}
		return fmt.Errorf("conflicting visibility. %s (%s) has a parent with different visibility attribute", parentName, info.Visibility.String())
	}

	w.path.push(info)

	return nil
}

func (w *walker) Enter(l reflectwalk.Location) error {
	return nil
}

func (w *walker) Exit(l reflectwalk.Location) error {
	if l == reflectwalk.StructField || l == reflectwalk.ArrayElem || l == reflectwalk.SliceElem {
		w.path.pop()
	}
	return nil
}

// defaults to unset
func (w *walker) visibility() Visibility {
	if !w.path.isEmpty() {
		return w.path.top().Visibility
	}
	return Unset
}

func (w *walker) name() string {
	if w.path.isEmpty() {
		return ""
	}
	var sbb strings.Builder
	sbb.Grow(w.path.len() * 10)
	first := true
	for i := 0; i < w.path.len(); i++ {
		if !first {
			sbb.WriteByte('_')
		} else {
			first = false
		}
		sbb.WriteString(w.path[i].name)
	}
	return sbb.String()
}

type pathStack []LeafInfo

func (s *pathStack) len() int {
	return len(*s)
}

func (s *pathStack) isEmpty() bool {
	return len(*s) == 0
}

func (s *pathStack) push(l LeafInfo) {
	*s = append(*s, l)
}

func (s *pathStack) pop() {
	if !s.isEmpty() {
		*s = (*s)[:len(*s)-1]
	}
}

func (s *pathStack) top() LeafInfo {
	return (*s)[len(*s)-1]
}
