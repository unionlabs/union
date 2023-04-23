// Copyright 2011 Huan Du. All rights reserved.
// Licensed under the MIT license that can be found in the LICENSE file.

package skiplist

import (
	"unsafe"
)

// Element is an element node of a skip list.
type Element struct {
	elementHeader

	Value interface{}
	key   interface{}
	score float64

	prev         *Element  // Points to previous adjacent elem.
	prevTopLevel *Element  // Points to previous element which points to this element's top most level.
	list         *SkipList // The list contains this elem.
}

// elementHeader is the header of an element or a skip list.
// It must be the first anonymous field in a type to make Element() work correctly.
type elementHeader struct {
	levels []*Element // Next element at all levels.
}

func (header *elementHeader) Element() *Element {
	return (*Element)(unsafe.Pointer(header))
}

func newElement(list *SkipList, level int, score float64, key, value interface{}) *Element {
	return &Element{
		elementHeader: elementHeader{
			levels: make([]*Element, level),
		},
		Value: value,
		key:   key,
		score: score,
		list:  list,
	}
}

// Next returns next adjacent elem.
func (elem *Element) Next() *Element {
	if len(elem.levels) == 0 {
		return nil
	}

	return elem.levels[0]
}

// Prev returns previous adjacent elem.
func (elem *Element) Prev() *Element {
	return elem.prev
}

// NextLevel returns next element at specific level.
// If level is invalid, returns nil.
func (elem *Element) NextLevel(level int) *Element {
	if level < 0 || level >= len(elem.levels) {
		return nil
	}

	return elem.levels[level]
}

// PrevLevel returns previous element which points to this element at specific level.
// If level is invalid, returns nil.
func (elem *Element) PrevLevel(level int) *Element {
	if level < 0 || level >= len(elem.levels) {
		return nil
	}

	if level == 0 {
		return elem.prev
	}

	if level == len(elem.levels)-1 {
		return elem.prevTopLevel
	}

	prev := elem.prev

	for prev != nil {
		if level < len(prev.levels) {
			return prev
		}

		prev = prev.prevTopLevel
	}

	return prev
}

// Key returns the key of the elem.
func (elem *Element) Key() interface{} {
	return elem.key
}

// Score returns the score of this element.
// The score is a hint when comparing elements.
// Skip list keeps all elements sorted by score from smallest to largest.
func (elem *Element) Score() float64 {
	return elem.score
}

// Level returns the level of this elem.
func (elem *Element) Level() int {
	return len(elem.levels)
}

func (elem *Element) reset() {
	elem.list = nil
	elem.prev = nil
	elem.prevTopLevel = nil
	elem.levels = nil
}
