package config

var BLS12_378 = Curve{
	Name:         "bls12-378",
	CurvePackage: "bls12378",
	EnumID:       "BLS12_378",
	FrModulus:    "14883435066912132899950318861128167269793560281114003360875131245101026639873",
	FpModulus:    "605248206075306171733248481581800960739847691770924913753520744034740935903401304776283802348837311170974282940417",
	G1: Point{
		CoordType:        "fp.Element",
		CoordExtDegree:   1,
		PointName:        "g1",
		GLV:              true,
		CofactorCleaning: true,
		CRange:           defaultCRange(),
	},
	G2: Point{
		CoordType:        "fptower.E2",
		CoordExtDegree:   2,
		PointName:        "g2",
		GLV:              true,
		CofactorCleaning: true,
		CRange:           defaultCRange(),
		Projective:       true,
	},
	// 2-isogeny
	HashE1: &HashSuiteSswu{
		A: []string{"0x3eeb0416684d18f2c41f0ac56b4172c97877b1f2170ca6f42387dd67a2cc5c175e179b1a06ffff79e0723fffffffff2"},
		B: []string{"0x16"},
		Z: []int{11},
		Isogeny: &Isogeny{
			XMap: RationalPolynomial{
				Num: [][]string{
					{"0x2f304310ce39d2c3011a6d50eb4ece730cab541269dbc53c7594241b1c244eff01c0ce03cbe00000000000000000000"},
					{"0x9d9ea03fd9a908c76d1012fb4743eb0c720b5849c7b761ff1e3f31fc34200004ca4510000000001"},
					{"0x2f304310ce39d2c3ed885db0b1cc5b9e3043708b54c1a5cf20a52889c7b761fdaf1f98fe1a1000072f6798000000001"},
				},
				Den: [][]string{
					{"0x2767a80ff66a4231db4404bed1d0fac31c82d61271edd87fc78fcc7f0d0800013291440000000004"},
				},
			},
			YMap: RationalPolynomial{
				Num: [][]string{
					{"0x2f304310ce39d2c3ed885db0b1cc5b9e3043708b54c1a5cf20a52889c7b761fdaf1f98fe1a1000072f6797fffffffff"},
					{"0x7dd6082cd09a322f6a993378ddbf030e107849ad95ef7bbdbc611d64e38ea7c4e9cea46c7d00013291440000000002"},
					{"0x1f75820b34268c838ac8d9803d05ca3f43c5122b2366f9c76b7f1f7530b7fefd221e864e5f90000bf9aca8000000002"},
					{"0x370da3939b4375e4951f17f8cf6e6ae3384eadf7e2e1ec1c50c0af4b69009cfd4c4f87d31e68000861f8dc000000001"},
				},
				Den: [][]string{
					{"0x3eeb0416684d19053cb5d240ed107a284059eb647102326980dc360d0a49d7fce97f76a822c00009948a1fffffffff9"},
					{"0xec6df05fc67d8d2b23981c78eae5e092ab11046eab9312fead5ecafa4e3000072f6798000000000c"},
					{"0x7636f82fe33ec69591cc0e3c7572f0495588823755c9897f56af657d2718000397b3cc000000000c"},
				},
			},
		},
	},
}

var tBLS12_78 = TwistedEdwardsCurve{
	Name:     BLS12_378.Name,
	Package:  "twistededwards",
	EnumID:   BLS12_378.EnumID,
	A:        "16249",
	D:        "826857503717340716663906603396009292766308904506333520048618402505612607353",
	Cofactor: "8",
	Order:    "1860429383364016612493789857641020908721690454530426945748883177201355593303",
	BaseX:    "6772953896463446981848394912418300623023000177913479948380771331313783560843",
	BaseY:    "9922290044608088599966879240752111513195706854076002240583420830067351093249",
}

func init() {
	addCurve(&BLS12_378)
	addTwistedEdwardCurve(&tBLS12_78)
}
