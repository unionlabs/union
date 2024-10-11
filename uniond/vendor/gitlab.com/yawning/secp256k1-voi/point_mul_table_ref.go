// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

//go:build !amd64 || purego

package secp256k1

import "gitlab.com/yawning/secp256k1-voi/internal/helpers"

func lookupProjectivePoint(tbl *projectivePointMultTable, out *Point, idx uint64) {
	out.Identity()
	for i := uint64(1); i < 16; i++ {
		out.uncheckedConditionalSelect(out, &tbl[i-1], helpers.Uint64Equal(idx, i))
	}
}

func lookupAffinePoint(tbl *affinePointMultTable, out *affinePoint, idx uint64) {
	for i := uint64(1); i < 16; i++ {
		ctrl := helpers.Uint64Equal(idx, i)
		out.x.ConditionalSelect(&out.x, &tbl[i-1].x, ctrl)
		out.y.ConditionalSelect(&out.y, &tbl[i-1].y, ctrl)
	}
}
