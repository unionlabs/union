// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

//go:build ignore

package main

import (
	"fmt"
	"os"

	"gitlab.com/yawning/secp256k1-voi"
)

func main() {
	// base = G.
	base := secp256k1.NewPointFrom(secp256k1.NewGeneratorPoint())

	// Calculate a series of 32 tables, of precomputed multiples of
	// G [1G, ... 255G].  Each successive table is the previous table
	// doubled 8 times.
	tbl := new([secp256k1.ScalarSize][255]secp256k1.Point)
	for i := range tbl {
		tbl[i][0].Set(base)

		tmp := secp256k1.NewPointFrom(base)
		for j := 1; j < 255; j++ {
			tmp.Add(base, tmp)
			tbl[i][j].Set(tmp)
		}

		base.Double(base)
		base.Double(base)
		base.Double(base)
		base.Double(base)
		base.Double(base)
		base.Double(base)
		base.Double(base)
		base.Double(base)
	}

	b := make([]byte, 0, len(tbl)*255*(secp256k1.UncompressedPointSize-1))
	for _, subTbl := range tbl {
		for _, p := range subTbl {
			pBytes := p.UncompressedBytes()
			b = append(b, pBytes[1:]...) // Skip the stupid prefix.
		}
	}
	if err := os.WriteFile("point_mul_table.bin", b, 0o600); err != nil {
		fmt.Printf("failed to write output file: %v\n", err)
		os.Exit(1)
	}
}
