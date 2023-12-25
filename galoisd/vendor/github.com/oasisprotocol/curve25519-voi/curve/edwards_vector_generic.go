// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//go:build !amd64 || purego || force32bit

package curve

import (
	"fmt"

	"github.com/oasisprotocol/curve25519-voi/internal/disalloweq"
)

// Stub type definitions filled with my inner urges to panic, to allow
// the non-vector/vector code to be somewhat consolidated to prevent
// an explosion of files.

const supportsVectorizedEdwards = false

var (
	errVectorNotSupported = fmt.Errorf("curve: vector backend not supported")

	// These are not actually used, since the vector code is never called
	// but need to be defined.
	constEXTENDEDPOINT_IDENTITY            extendedPoint
	constVECTOR_ODD_MULTIPLES_OF_BASEPOINT *cachedPointNafLookupTable8
	constVECTOR_ODD_MULTIPLES_OF_B_SHL_128 *cachedPointNafLookupTable8
)

type extendedPoint struct {
	disalloweq.DisallowEqual //nolint:unused
}

func (p *EdwardsPoint) setExtended(ep *extendedPoint) *EdwardsPoint {
	panic(errVectorNotSupported)
}

func (p *EdwardsPoint) setCached(cp *cachedPoint) *EdwardsPoint {
	panic(errVectorNotSupported)
}

func (p *extendedPoint) SetEdwards(ep *EdwardsPoint) *extendedPoint {
	panic(errVectorNotSupported)
}

func (p *extendedPoint) Identity() *extendedPoint {
	panic(errVectorNotSupported)
}

func (p *extendedPoint) Double(t *extendedPoint) *extendedPoint {
	panic(errVectorNotSupported)
}

func (p *extendedPoint) MulByPow2(t *extendedPoint, k uint) *extendedPoint {
	panic(errVectorNotSupported)
}

func (p *extendedPoint) AddExtendedCached(a *extendedPoint, b *cachedPoint) *extendedPoint {
	panic(errVectorNotSupported)
}

func (p *extendedPoint) SubExtendedCached(a *extendedPoint, b *cachedPoint) *extendedPoint {
	panic(errVectorNotSupported)
}

type cachedPoint struct {
	disalloweq.DisallowEqual //nolint:unused
}

func (p *cachedPoint) SetExtended(ep *extendedPoint) *cachedPoint {
	panic(errVectorNotSupported)
}

func (p *cachedPoint) ConditionalNegate(choice int) {
	panic(errVectorNotSupported)
}
