// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

//go:build amd64 && !gccgo && !appengine && !nacl && !noasm
// +build amd64,!gccgo,!appengine,!nacl,!noasm

package highwayhash

import "golang.org/x/sys/cpu"

var (
	useSSE4 = cpu.X86.HasSSE41
	useAVX2 = cpu.X86.HasAVX2
	useNEON = false
	useSVE  = false
	useSVE2 = false
	useVMX  = false
)

//go:noescape
func initializeSSE4(state *[16]uint64, key []byte)

//go:noescape
func initializeAVX2(state *[16]uint64, key []byte)

//go:noescape
func updateSSE4(state *[16]uint64, msg []byte)

//go:noescape
func updateAVX2(state *[16]uint64, msg []byte)

//go:noescape
func finalizeSSE4(out []byte, state *[16]uint64)

//go:noescape
func finalizeAVX2(out []byte, state *[16]uint64)

func initialize(state *[16]uint64, key []byte) {
	switch {
	case useAVX2:
		initializeAVX2(state, key)
	case useSSE4:
		initializeSSE4(state, key)
	default:
		initializeGeneric(state, key)
	}
}

func update(state *[16]uint64, msg []byte) {
	switch {
	case useAVX2:
		updateAVX2(state, msg)
	case useSSE4:
		updateSSE4(state, msg)
	default:
		updateGeneric(state, msg)
	}
}

func finalize(out []byte, state *[16]uint64) {
	switch {
	case useAVX2:
		finalizeAVX2(out, state)
	case useSSE4:
		finalizeSSE4(out, state)
	default:
		finalizeGeneric(out, state)
	}
}
