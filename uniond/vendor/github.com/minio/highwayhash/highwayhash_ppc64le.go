// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

//+build !noasm,!appengine

package highwayhash

var (
	useSSE4 = false
	useAVX2 = false
	useNEON = false
	useVMX  = true
)

//go:noescape
func updatePpc64Le(state *[16]uint64, msg []byte)

func initialize(state *[16]uint64, key []byte) {
	initializeGeneric(state, key)
}

func update(state *[16]uint64, msg []byte) {
	if useVMX {
		updatePpc64Le(state, msg)
	} else {
		updateGeneric(state, msg)
	}
}

func finalize(out []byte, state *[16]uint64) {
	finalizeGeneric(out, state)
}
