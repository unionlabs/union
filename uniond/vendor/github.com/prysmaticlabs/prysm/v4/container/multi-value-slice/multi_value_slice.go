// Package mvslice defines a multi value slice container. The purpose of the container is to be a replacement for a slice
// in scenarios where many objects of the same type share a copy of an identical or nearly identical slice.
// In such case using the multi value slice should result in less memory allocation because many values of the slice can be shared between objects.
//
// The multi value slice should be initialized by calling the Init function and passing the initial values of the slice.
// After initializing the slice, it can be shared between object by using the Copy function.
// Note that simply assigning the same multi value slice to several objects is not enough for it to work properly.
// Calling Copy is required in most circumstances (an exception is when the source object has only shared values).
//
//	s := &Slice[int, *testObject]{}
//	s.Init([]int{1, 2, 3})
//	src := &testObject{id: id1, slice: s} // id1 is some UUID
//	dst := &testObject{id: id2, slice: s} // id2 is some UUID
//	s.Copy(src, dst)
//
// Each Value stores a value of type V along with identifiers to objects that have this value.
// A MultiValueItem is a slice of Value elements. A Slice contains shared items, individual items and appended items.
//
// You can think of a shared value as the original value (i.e. the value at the point in time when the multi value slice was constructed),
// and of an individual value as a changed value.
// There is no notion of a shared appended value because appended values never have an original value (appended values are empty when the slice is created).
//
// Whenever any of the slice’s functions (apart from Init) is called, the function needs to know which object it is dealing with.
// This is because if an object has an individual/appended value, the function must get/set/change this particular value instead of the shared value
// or another individual/appended value.
//
// The way appended items are stored is as follows. Let’s say appended items were a regular slice that is initially empty,
// and we append an item for object0 and then append another item for object1.
// Now we have two items in the slice, but object1 only has an item in index 1. This makes things very confusing and hard to deal with.
// If we make appended items a []*Value, things don’t become much better.
// It is therefore easiest to make appended items a []*MultiValueItem, which allows each object to have its own values starting at index 0
// and not having any “gaps”.
//
// The Detach function should be called when an object gets garbage collected.
// Its purpose is to clean up the slice from individual/appended values of the collected object.
// Otherwise the slice will get polluted with values for non-existing objects.
//
// Example diagram illustrating what happens after copying, updating and detaching:
//
//		Create object o1 with value 10. At this point we only have a shared value.
//
//		===================
//		shared | individual
//		===================
//		10     |
//
//		Copy object o1 to object o2. o2 shares the value with o1, no individual value is created.
//
//		===================
//		shared | individual
//		===================
//		10     |
//
//		Update value of object o2 to 20. An individual value is created.
//
//		===================
//		shared | individual
//		===================
//		10     | 20: [o2]
//
//		 Copy object o2 to object o3. The individual value's object list is updated.
//
//		===================
//		shared | individual
//		===================
//		10     | 20: [o2,o3]
//
//		 Update value of object o3 to 30. There are two individual values now, one for o2 and one for o3.
//
//		===================
//		shared | individual
//		===================
//		10     | 20: [o2]
//		       | 30: [o3]
//
//		 Update value of object o2 to 10. o2 no longer has an individual value
//		 because it got "reverted" to the original, shared value,
//
//		===================
//		shared | individual
//	 ===================
//		10     | 30: [o3]
//
//		 Detach object o3. Individual value for o3 is removed.
//
//		===================
//		shared | individual
//		===================
//		10     |
package mvslice

import (
	"fmt"
	"sync"

	"github.com/pkg/errors"
)

// Amount of references beyond which a multivalue object is considered
// fragmented.
const fragmentationLimit = 50000

// Id is an object identifier.
type Id = uint64

// Identifiable represents an object that can be uniquely identified by its Id.
type Identifiable interface {
	Id() Id
}

// MultiValueSlice defines an abstraction over all concrete implementations of the generic Slice.
type MultiValueSlice[V comparable] interface {
	Len(obj Identifiable) int
	At(obj Identifiable, index uint64) (V, error)
	Value(obj Identifiable) []V
}

// MultiValueSliceComposite describes a struct for which we have access to a multivalue
// slice along with the desired state.
type MultiValueSliceComposite[V comparable] struct {
	Identifiable
	MultiValueSlice[V]
}

// State returns the referenced state.
func (m MultiValueSliceComposite[V]) State() Identifiable {
	return m.Identifiable
}

// Value defines a single value along with one or more IDs that share this value.
type Value[V any] struct {
	val V
	ids []uint64
}

// MultiValueItem defines a collection of Value items.
type MultiValueItem[V any] struct {
	Values []*Value[V]
}

// Slice is the main component of the multi-value slice data structure. It has two type parameters:
//   - V comparable - the type of values stored the slice. The constraint is required
//     because certain operations (e.g. updating, appending) have to compare values against each other.
//   - O interfaces.Identifiable - the type of objects sharing the slice. The constraint is required
//     because we need a way to compare objects against each other in order to know which objects
//     values should be accessed.
type Slice[V comparable] struct {
	sharedItems     []V
	individualItems map[uint64]*MultiValueItem[V]
	appendedItems   []*MultiValueItem[V]
	cachedLengths   map[uint64]int
	lock            sync.RWMutex
}

// Init initializes the slice with sensible defaults. Input values are assigned to shared items.
func (s *Slice[V]) Init(items []V) {
	s.sharedItems = items
	s.individualItems = map[uint64]*MultiValueItem[V]{}
	s.appendedItems = []*MultiValueItem[V]{}
	s.cachedLengths = map[uint64]int{}
}

// Len returns the number of items for the input object.
func (s *Slice[V]) Len(obj Identifiable) int {
	s.lock.RLock()
	defer s.lock.RUnlock()

	l, ok := s.cachedLengths[obj.Id()]
	if !ok {
		return len(s.sharedItems)
	}
	return l
}

// Copy copies items between the source and destination.
func (s *Slice[V]) Copy(src, dst Identifiable) {
	s.lock.Lock()
	defer s.lock.Unlock()

	for _, item := range s.individualItems {
		for _, v := range item.Values {
			_, found := containsId(v.ids, src.Id())
			if found {
				v.ids = append(v.ids, dst.Id())
				break
			}
		}
	}

	for _, item := range s.appendedItems {
		found := false
		for _, v := range item.Values {
			_, found = containsId(v.ids, src.Id())
			if found {
				v.ids = append(v.ids, dst.Id())
				break
			}
		}
		if !found {
			// This is an optimization. If we didn't find an appended item at index i,
			// then all larger indices don't have an appended item for the object either.
			break
		}
	}

	srcLen, ok := s.cachedLengths[src.Id()]
	if ok {
		s.cachedLengths[dst.Id()] = srcLen
	}
}

// Value returns all items for the input object.
func (s *Slice[V]) Value(obj Identifiable) []V {
	s.lock.RLock()
	defer s.lock.RUnlock()

	l, ok := s.cachedLengths[obj.Id()]
	if ok {
		result := make([]V, l)
		s.fillOriginalItems(obj, &result)

		sharedLen := len(s.sharedItems)
		for i, item := range s.appendedItems {
			found := false
			for _, v := range item.Values {
				_, found = containsId(v.ids, obj.Id())
				if found {
					result[sharedLen+i] = v.val
					break
				}
			}
			if !found {
				// This is an optimization. If we didn't find an appended item at index i,
				// then all larger indices don't have an appended item for the object either.
				return result
			}
		}
		return result
	} else {
		result := make([]V, len(s.sharedItems))
		s.fillOriginalItems(obj, &result)
		return result
	}
}

// At returns the item at the requested index for the input object.
// Appended items' indices are always larger than shared/individual items' indices.
// We first check if the index is within the length of shared items.
// If it is, then we return an individual value at that index - if it exists - or a shared value otherwise.
// If the index is beyond the length of shared values, it is an appended item and that's what gets returned.
func (s *Slice[V]) At(obj Identifiable, index uint64) (V, error) {
	s.lock.RLock()
	defer s.lock.RUnlock()

	if index >= uint64(len(s.sharedItems)+len(s.appendedItems)) {
		var def V
		return def, fmt.Errorf("index %d out of bounds", index)
	}

	isOriginal := index < uint64(len(s.sharedItems))
	if isOriginal {
		ind, ok := s.individualItems[index]
		if !ok {
			return s.sharedItems[index], nil
		}
		for _, v := range ind.Values {
			for _, id := range v.ids {
				if id == obj.Id() {
					return v.val, nil
				}
			}
		}
		return s.sharedItems[index], nil
	} else {
		item := s.appendedItems[index-uint64(len(s.sharedItems))]
		for _, v := range item.Values {
			for _, id := range v.ids {
				if id == obj.Id() {
					return v.val, nil
				}
			}
		}
		var def V
		return def, fmt.Errorf("index %d out of bounds", index)
	}
}

// UpdateAt updates the item at the required index for the input object to the passed in value.
func (s *Slice[V]) UpdateAt(obj Identifiable, index uint64, val V) error {
	s.lock.Lock()
	defer s.lock.Unlock()

	if index >= uint64(len(s.sharedItems)+len(s.appendedItems)) {
		return fmt.Errorf("index %d out of bounds", index)
	}

	isOriginal := index < uint64(len(s.sharedItems))
	if isOriginal {
		s.updateOriginalItem(obj, index, val)
		return nil
	}
	return s.updateAppendedItem(obj, index, val)
}

// Append adds a new item to the input object.
func (s *Slice[V]) Append(obj Identifiable, val V) {
	s.lock.Lock()
	defer s.lock.Unlock()

	if len(s.appendedItems) == 0 {
		s.appendedItems = append(s.appendedItems, &MultiValueItem[V]{Values: []*Value[V]{{val: val, ids: []uint64{obj.Id()}}}})
		s.cachedLengths[obj.Id()] = len(s.sharedItems) + 1
		return
	}

	for _, item := range s.appendedItems {
		found := false
		for _, v := range item.Values {
			_, found = containsId(v.ids, obj.Id())
			if found {
				break
			}
		}
		if !found {
			newValue := true
			for _, v := range item.Values {
				if v.val == val {
					v.ids = append(v.ids, obj.Id())
					newValue = false
					break
				}
			}
			if newValue {
				item.Values = append(item.Values, &Value[V]{val: val, ids: []uint64{obj.Id()}})
			}

			l, ok := s.cachedLengths[obj.Id()]
			if ok {
				s.cachedLengths[obj.Id()] = l + 1
			} else {
				s.cachedLengths[obj.Id()] = len(s.sharedItems) + 1
			}

			return
		}
	}

	s.appendedItems = append(s.appendedItems, &MultiValueItem[V]{Values: []*Value[V]{{val: val, ids: []uint64{obj.Id()}}}})

	s.cachedLengths[obj.Id()] = s.cachedLengths[obj.Id()] + 1
}

// Detach removes the input object from the multi-value slice.
// What this means in practice is that we remove all individual and appended values for that object and clear the cached length.
func (s *Slice[V]) Detach(obj Identifiable) {
	s.lock.Lock()
	defer s.lock.Unlock()

	for i, ind := range s.individualItems {
		for vi, v := range ind.Values {
			foundIndex, found := containsId(v.ids, obj.Id())
			if found {
				if len(v.ids) == 1 {
					if len(ind.Values) == 1 {
						delete(s.individualItems, i)
					} else {
						ind.Values = deleteElemFromSlice(ind.Values, vi)
					}
				} else {
					v.ids = deleteElemFromSlice(v.ids, foundIndex)
				}
				break
			}
		}
	}

	for _, item := range s.appendedItems {
		found := false
		for vi, v := range item.Values {
			var foundIndex int
			foundIndex, found = containsId(v.ids, obj.Id())
			if found {
				if len(v.ids) == 1 {
					item.Values = deleteElemFromSlice(item.Values, vi)
				} else {
					v.ids = deleteElemFromSlice(v.ids, foundIndex)
				}
				break
			}
		}
		if !found {
			// This is an optimization. If we didn't find an appended item at index i,
			// then all larger indices don't have an appended item for the object either.
			break
		}
	}

	delete(s.cachedLengths, obj.Id())
}

// MultiValueStatistics generates the multi-value stats object for the respective
// multivalue slice.
func (s *Slice[V]) MultiValueStatistics() MultiValueStatistics {
	s.lock.RLock()
	defer s.lock.RUnlock()

	stats := MultiValueStatistics{}
	stats.TotalIndividualElements = len(s.individualItems)
	totalIndRefs := 0

	for _, v := range s.individualItems {
		for _, ival := range v.Values {
			totalIndRefs += len(ival.ids)
		}
	}

	stats.TotalAppendedElements = len(s.appendedItems)
	totalAppRefs := 0

	for _, v := range s.appendedItems {
		for _, ival := range v.Values {
			totalAppRefs += len(ival.ids)
		}
	}
	stats.TotalIndividualElemReferences = totalIndRefs
	stats.TotalAppendedElemReferences = totalAppRefs

	return stats
}

// IsFragmented checks if our mutlivalue object is fragmented (individual references held).
// If the number of references is higher than our threshold we return true.
func (s *Slice[V]) IsFragmented() bool {
	stats := s.MultiValueStatistics()
	return stats.TotalIndividualElemReferences+stats.TotalAppendedElemReferences >= fragmentationLimit
}

// Reset builds a new multivalue object with respect to the
// provided object's id. The base slice will be based on this
// particular id.
func (s *Slice[V]) Reset(obj Identifiable) *Slice[V] {
	s.lock.RLock()
	defer s.lock.RUnlock()

	l, ok := s.cachedLengths[obj.Id()]
	if !ok {
		l = len(s.sharedItems)
	}

	items := make([]V, l)
	copy(items, s.sharedItems)
	for i, ind := range s.individualItems {
		for _, v := range ind.Values {
			_, found := containsId(v.ids, obj.Id())
			if found {
				items[i] = v.val
				break
			}
		}
	}

	index := len(s.sharedItems)
	for _, app := range s.appendedItems {
		found := true
		for _, v := range app.Values {
			_, found = containsId(v.ids, obj.Id())
			if found {
				items[index] = v.val
				index++
				break
			}
		}
		if !found {
			break
		}
	}

	reset := &Slice[V]{}
	reset.Init(items)
	return reset
}

func (s *Slice[V]) fillOriginalItems(obj Identifiable, items *[]V) {
	for i, item := range s.sharedItems {
		ind, ok := s.individualItems[uint64(i)]
		if !ok {
			(*items)[i] = item
		} else {
			found := false
			for _, v := range ind.Values {
				_, found = containsId(v.ids, obj.Id())
				if found {
					(*items)[i] = v.val
					break
				}
			}
			if !found {
				(*items)[i] = item
			}
		}
	}
}

func (s *Slice[V]) updateOriginalItem(obj Identifiable, index uint64, val V) {
	ind, ok := s.individualItems[index]
	if ok {
		for mvi, v := range ind.Values {
			// if we find an existing value, we remove it
			foundIndex, found := containsId(v.ids, obj.Id())
			if found {
				if len(v.ids) == 1 {
					// There is an improvement to be made here. If len(ind.Values) == 1,
					// then after removing the item from the slice s.individualItems[i]
					// will be a useless map entry whose value is an empty slice.
					ind.Values = deleteElemFromSlice(ind.Values, mvi)
				} else {
					v.ids = deleteElemFromSlice(v.ids, foundIndex)
				}
				break
			}
		}
	}

	if val == s.sharedItems[index] {
		return
	}

	if !ok {
		s.individualItems[index] = &MultiValueItem[V]{Values: []*Value[V]{{val: val, ids: []uint64{obj.Id()}}}}
	} else {
		newValue := true
		for _, v := range ind.Values {
			if v.val == val {
				v.ids = append(v.ids, obj.Id())
				newValue = false
				break
			}
		}
		if newValue {
			ind.Values = append(ind.Values, &Value[V]{val: val, ids: []uint64{obj.Id()}})
		}
	}
}

func (s *Slice[V]) updateAppendedItem(obj Identifiable, index uint64, val V) error {
	item := s.appendedItems[index-uint64(len(s.sharedItems))]
	found := false
	for vi, v := range item.Values {
		var foundIndex int
		// if we find an existing value, we remove it
		foundIndex, found = containsId(v.ids, obj.Id())
		if found {
			if len(v.ids) == 1 {
				item.Values = deleteElemFromSlice(item.Values, vi)
			} else {
				v.ids = deleteElemFromSlice(v.ids, foundIndex)
			}
			break
		}
	}
	if !found {
		return fmt.Errorf("index %d out of bounds", index)
	}

	newValue := true
	for _, v := range item.Values {
		if v.val == val {
			v.ids = append(v.ids, obj.Id())
			newValue = false
			break
		}
	}
	if newValue {
		item.Values = append(item.Values, &Value[V]{val: val, ids: []uint64{obj.Id()}})
	}

	return nil
}

func containsId(ids []uint64, wanted uint64) (int, bool) {
	for i, id := range ids {
		if id == wanted {
			return i, true
		}
	}
	return 0, false
}

// deleteElemFromSlice does not relocate the slice, but it also does not preserve the order of items.
// This is not a problem here because the order of values in a MultiValueItem and object IDs doesn't matter.
func deleteElemFromSlice[T any](s []T, i int) []T {
	s[i] = s[len(s)-1] // Copy last element to index i.
	s = s[:len(s)-1]   // Truncate slice.
	return s
}

// EmptyMVSlice specifies a type which allows a normal slice to conform
// to the multivalue slice interface.
type EmptyMVSlice[V comparable] struct {
	fullSlice []V
}

func (e EmptyMVSlice[V]) Len(_ Identifiable) int {
	return len(e.fullSlice)
}

func (e EmptyMVSlice[V]) At(_ Identifiable, index uint64) (V, error) {
	if index >= uint64(len(e.fullSlice)) {
		var def V
		return def, errors.Errorf("index %d out of bounds", index)
	}
	return e.fullSlice[index], nil
}

func (e EmptyMVSlice[V]) Value(_ Identifiable) []V {
	return e.fullSlice
}

// BuildEmptyCompositeSlice builds a composite multivalue object with a native
// slice.
func BuildEmptyCompositeSlice[V comparable](values []V) MultiValueSliceComposite[V] {
	return MultiValueSliceComposite[V]{
		Identifiable:    nil,
		MultiValueSlice: EmptyMVSlice[V]{fullSlice: values},
	}
}

// MultiValueStatistics represents the internal properties of a multivalue slice.
type MultiValueStatistics struct {
	TotalIndividualElements       int
	TotalAppendedElements         int
	TotalIndividualElemReferences int
	TotalAppendedElemReferences   int
}
