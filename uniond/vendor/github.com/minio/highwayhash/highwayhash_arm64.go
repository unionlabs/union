// Copyright (c) 2017-2024 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

//go:build !noasm && !appengine
// +build !noasm,!appengine

package highwayhash

import (
	"golang.org/x/sys/cpu"
)

var (
	useSSE4 = false
	useAVX2 = false
	useNEON = cpu.ARM64.HasASIMD
	useSVE  = cpu.ARM64.HasSVE
	useSVE2 = false // cpu.ARM64.HasSVE2 -- disable until tested on real hardware
	useVMX  = false
)

func init() {
	if useSVE {
		if vl, _ := getVectorLength(); vl != 256 {
			//
			// Since HighwahHash is designed for AVX2,
			// SVE/SVE2 instructions only run correctly
			// for vector length of 256
			//
			useSVE2 = false
			useSVE = false
		}
	}
}

//go:noescape
func initializeArm64(state *[16]uint64, key []byte)

//go:noescape
func updateArm64(state *[16]uint64, msg []byte)

//go:noescape
func getVectorLength() (vl, pl uint64)

//go:noescape
func updateArm64Sve(state *[16]uint64, msg []byte)

//go:noescape
func updateArm64Sve2(state *[16]uint64, msg []byte)

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
	if useSVE2 {
		updateArm64Sve2(state, msg)
	} else if useSVE {
		updateArm64Sve(state, msg)
	} else if useNEON {
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
