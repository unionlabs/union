// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

// +build noasm !amd64,!arm64,!ppc64le

package highwayhash

var (
	useSSE4 = false
	useAVX2 = false
	useNEON = false
	useVMX  = false
)

func initialize(state *[16]uint64, k []byte) {
	initializeGeneric(state, k)
}

func update(state *[16]uint64, msg []byte) {
	updateGeneric(state, msg)
}

func finalize(out []byte, state *[16]uint64) {
	finalizeGeneric(out, state)
}
