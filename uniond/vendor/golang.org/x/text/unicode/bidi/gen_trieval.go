// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build ignore

package main

// Class is the Unicode BiDi class. Each rune has a single class.
type Class uint

const (
	L       Class = iota // LeftToRight
	R                    // RightToLeft
	EN                   // EuropeanNumber
	ES                   // EuropeanSeparator
	ET                   // EuropeanTerminator
	AN                   // ArabicNumber
	CS                   // CommonSeparator
	B                    // ParagraphSeparator
	S                    // SegmentSeparator
	WS                   // WhiteSpace
	ON                   // OtherNeutral
	BN                   // BoundaryNeutral
	NSM                  // NonspacingMark
	AL                   // ArabicLetter
	Control              // Control LRO - PDI

	numClass

	LRO // LeftToRightOverride
	RLO // RightToLeftOverride
	LRE // LeftToRightEmbedding
	RLE // RightToLeftEmbedding
	PDF // PopDirectionalFormat
	LRI // LeftToRightIsolate
	RLI // RightToLeftIsolate
	FSI // FirstStrongIsolate
	PDI // PopDirectionalIsolate

	unknownClass = ^Class(0)
)

// A trie entry has the following bits:
// 7..5  XOR mask for brackets
// 4     1: Bracket open, 0: Bracket close
// 3..0  Class type

const (
	openMask     = 0x10
	xorMaskShift = 5
)
