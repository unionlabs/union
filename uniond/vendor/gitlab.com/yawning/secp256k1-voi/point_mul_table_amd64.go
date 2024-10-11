// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

//go:build amd64 && !purego

package secp256k1

//go:noescape
func lookupProjectivePoint(tbl *projectivePointMultTable, out *Point, idx uint64)

//go:noescape
func lookupAffinePoint(tbl *affinePointMultTable, out *affinePoint, idx uint64)
