// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

//+build !noasm,!appengine

package highwayhash

var (
	useSSE4 = false
	useAVX2 = false
	useNEON = true
	useVMX  = false
)

//go:noescape
func initializeArm64(state *[16]uint64, key []byte)

//go:noescape
func updateArm64(state *[16]uint64, msg []byte)

//go:noescape
func finalizeArm64(out []byte, state *[16]uint64)

func initialize(state *[16]uint64, key []byte) {
	if useNEON {
		initializeArm64(state, key)
	} else {
		initializeGeneric(state, key)
	}
}

func update(state *[16]uint64, msg []byte) {
	if useNEON {
		updateArm64(state, msg)
	} else {
		updateGeneric(state, msg)
	}
}

func finalize(out []byte, state *[16]uint64) {
	if useNEON {
		finalizeArm64(out, state)
	} else {
		finalizeGeneric(out, state)
	}
}
