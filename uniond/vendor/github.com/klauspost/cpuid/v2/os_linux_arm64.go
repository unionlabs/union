// Copyright (c) 2020 Klaus Post, released under MIT License. See LICENSE file.

// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file located
// here https://github.com/golang/sys/blob/master/LICENSE

package cpuid

import (
	"encoding/binary"
	"io/ioutil"
	"runtime"
)

// HWCAP bits.
const (
	hwcap_FP       = 1 << 0
	hwcap_ASIMD    = 1 << 1
	hwcap_EVTSTRM  = 1 << 2
	hwcap_AES      = 1 << 3
	hwcap_PMULL    = 1 << 4
	hwcap_SHA1     = 1 << 5
	hwcap_SHA2     = 1 << 6
	hwcap_CRC32    = 1 << 7
	hwcap_ATOMICS  = 1 << 8
	hwcap_FPHP     = 1 << 9
	hwcap_ASIMDHP  = 1 << 10
	hwcap_CPUID    = 1 << 11
	hwcap_ASIMDRDM = 1 << 12
	hwcap_JSCVT    = 1 << 13
	hwcap_FCMA     = 1 << 14
	hwcap_LRCPC    = 1 << 15
	hwcap_DCPOP    = 1 << 16
	hwcap_SHA3     = 1 << 17
	hwcap_SM3      = 1 << 18
	hwcap_SM4      = 1 << 19
	hwcap_ASIMDDP  = 1 << 20
	hwcap_SHA512   = 1 << 21
	hwcap_SVE      = 1 << 22
	hwcap_ASIMDFHM = 1 << 23
)

func detectOS(c *CPUInfo) bool {
	// For now assuming no hyperthreading is reasonable.
	c.LogicalCores = runtime.NumCPU()
	c.PhysicalCores = c.LogicalCores
	c.ThreadsPerCore = 1
	if hwcap == 0 {
		// We did not get values from the runtime.
		// Try reading /proc/self/auxv

		// From https://github.com/golang/sys
		const (
			_AT_HWCAP  = 16
			_AT_HWCAP2 = 26

			uintSize = int(32 << (^uint(0) >> 63))
		)

		buf, err := ioutil.ReadFile("/proc/self/auxv")
		if err != nil {
			// e.g. on android /proc/self/auxv is not accessible, so silently
			// ignore the error and leave Initialized = false. On some
			// architectures (e.g. arm64) doinit() implements a fallback
			// readout and will set Initialized = true again.
			return false
		}
		bo := binary.LittleEndian
		for len(buf) >= 2*(uintSize/8) {
			var tag, val uint
			switch uintSize {
			case 32:
				tag = uint(bo.Uint32(buf[0:]))
				val = uint(bo.Uint32(buf[4:]))
				buf = buf[8:]
			case 64:
				tag = uint(bo.Uint64(buf[0:]))
				val = uint(bo.Uint64(buf[8:]))
				buf = buf[16:]
			}
			switch tag {
			case _AT_HWCAP:
				hwcap = val
			case _AT_HWCAP2:
				// Not used
			}
		}
		if hwcap == 0 {
			return false
		}
	}

	// HWCap was populated by the runtime from the auxiliary vector.
	// Use HWCap information since reading aarch64 system registers
	// is not supported in user space on older linux kernels.
	c.featureSet.setIf(isSet(hwcap, hwcap_AES), AESARM)
	c.featureSet.setIf(isSet(hwcap, hwcap_ASIMD), ASIMD)
	c.featureSet.setIf(isSet(hwcap, hwcap_ASIMDDP), ASIMDDP)
	c.featureSet.setIf(isSet(hwcap, hwcap_ASIMDHP), ASIMDHP)
	c.featureSet.setIf(isSet(hwcap, hwcap_ASIMDRDM), ASIMDRDM)
	c.featureSet.setIf(isSet(hwcap, hwcap_CPUID), ARMCPUID)
	c.featureSet.setIf(isSet(hwcap, hwcap_CRC32), CRC32)
	c.featureSet.setIf(isSet(hwcap, hwcap_DCPOP), DCPOP)
	c.featureSet.setIf(isSet(hwcap, hwcap_EVTSTRM), EVTSTRM)
	c.featureSet.setIf(isSet(hwcap, hwcap_FCMA), FCMA)
	c.featureSet.setIf(isSet(hwcap, hwcap_FP), FP)
	c.featureSet.setIf(isSet(hwcap, hwcap_FPHP), FPHP)
	c.featureSet.setIf(isSet(hwcap, hwcap_JSCVT), JSCVT)
	c.featureSet.setIf(isSet(hwcap, hwcap_LRCPC), LRCPC)
	c.featureSet.setIf(isSet(hwcap, hwcap_PMULL), PMULL)
	c.featureSet.setIf(isSet(hwcap, hwcap_SHA1), SHA1)
	c.featureSet.setIf(isSet(hwcap, hwcap_SHA2), SHA2)
	c.featureSet.setIf(isSet(hwcap, hwcap_SHA3), SHA3)
	c.featureSet.setIf(isSet(hwcap, hwcap_SHA512), SHA512)
	c.featureSet.setIf(isSet(hwcap, hwcap_SM3), SM3)
	c.featureSet.setIf(isSet(hwcap, hwcap_SM4), SM4)
	c.featureSet.setIf(isSet(hwcap, hwcap_SVE), SVE)

	// The Samsung S9+ kernel reports support for atomics, but not all cores
	// actually support them, resulting in SIGILL. See issue #28431.
	// TODO(elias.naur): Only disable the optimization on bad chipsets on android.
	c.featureSet.setIf(isSet(hwcap, hwcap_ATOMICS) && runtime.GOOS != "android", ATOMICS)

	return true
}

func isSet(hwc uint, value uint) bool {
	return hwc&value != 0
}
