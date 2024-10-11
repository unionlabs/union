package config

var BLS24_315 = Curve{
	Name:         "bls24-315",
	CurvePackage: "bls24315",
	EnumID:       "BLS24_315",
	FrModulus:    "11502027791375260645628074404575422495959608200132055716665986169834464870401",
	FpModulus:    "39705142709513438335025689890408969744933502416914749335064285505637884093126342347073617133569",
	G1: Point{
		CoordType:        "fp.Element",
		CoordExtDegree:   1,
		PointName:        "g1",
		GLV:              true,
		CofactorCleaning: true,
		CRange:           defaultCRange(),
	},
	G2: Point{
		CoordType:        "fptower.E4",
		CoordExtDegree:   4,
		PointName:        "g2",
		GLV:              true,
		CofactorCleaning: true,
		CRange:           defaultCRange(),
		Projective:       true,
	},
	// 2-isogeny
	HashE1: &HashSuiteSswu{
		A: []string{"0x4c23a0197b9ca68541a4cef14af4cfe81cc324cac5626d9ff4ee66df9ea2678877910f40300001f"},
		B: []string{"0x16"},
		Z: []int{13},
		Isogeny: &Isogeny{
			XMap: RationalPolynomial{
				Num: [][]string{
					{"0x2611d014c792a8ffd30982483b3ee757787d35c9e880e096a850c8e24edf5c71f880eff103c0002"},
					{"0x2611d01644a40e35d2dad31956fafeee9f1a0831db5b7b49ac10c81d6ff9afd483bf88000000000"},
					{"0x391ab82082520bc9ef97728ef1d4703e7115c13f9db942831972c63be0e6bc1d3ee023f70240001"},
				},
				Den: [][]string{
					{"0x261b56ebccc821ae82c6025bea42e1d731e2a911e6c66652b682a1f0411fdc017f9ffffe"},
				},
			},
			YMap: RationalPolynomial{
				Num: [][]string{
					{"0x391ab82082520bc9ef97728ef1d4703e7115c13f9db942831972c63be0e6bc1d3ee023f7023ffff"},
					{"0x391ab822bdec239aef516bc89b6e93a12b00fcdb8a012a8f9f12c514928e39310fbe080d7c9fffd"},
					{"0x391ab82166f61550bc483ca602787e65eea70c4ac90938ee82192c2c27f687bec59f4c000000000"},
					{"0x429f2c25ed5fb86b978605a6c4cd2d9e2e996174e2ad78439db091f0866286221eb029f582a0001"},
				},
				Den: [][]string{
					{"0x4c23a02b586d650d3f7498be97c5eafdec1d01aa27a1ae0421ee5da52bde5026fe802ff402ffff9"},
					{"0xe4a40986ccb0ca1710a40e277d914b0b2b4ff66b68a665f0470fcba186bf2808fdbfffe8"},
					{"0x725204c36658650b88520713bec8a58595a7fb35b45332f82387e5d0c35f94047edffffa"},
				},
			},
		},
	},
}

var tBLS24_315 = TwistedEdwardsCurve{
	Name:     BLS24_315.Name,
	Package:  "twistededwards",
	EnumID:   BLS24_315.EnumID,
	A:        "-1",
	D:        "8771873785799030510227956919069912715983412030268481769609515223557738569779",
	Cofactor: "8",
	Order:    "1437753473921907580703509300571927811987591765799164617677716990775193563777",
	BaseX:    "750878639751052675245442739791837325424717022593512121860796337974109802674",
	BaseY:    "1210739767513185331118744674165833946943116652645479549122735386298364723201",
}

func init() {
	addCurve(&BLS24_315)
	addTwistedEdwardCurve(&tBLS24_315)
}
