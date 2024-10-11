package element

const InverseTests = `

{{if $.UsingP20Inverse}}

func Test{{.ElementName}}InversionApproximation(t *testing.T) {
	var x {{.ElementName}}
	for i := 0; i < 1000; i++ {
		x.SetRandom()

		// Normally small elements are unlikely. Here we give them a higher chance
		xZeros := mrand.Int() % Limbs //#nosec G404 weak rng is fine here
		for j := 1; j < xZeros; j++ {
			x[Limbs - j] = 0
		}

		a := approximate(&x, x.BitLen())
		aRef := approximateRef(&x)

		if a != aRef {
			t.Error("Approximation mismatch")
		}
	}
}

func Test{{.ElementName}}InversionCorrectionFactorFormula(t *testing.T) {
	const kLimbs = k * Limbs
	const power = kLimbs*6 + invIterationsN*(kLimbs-k+1)
	factorInt := big.NewInt(1)
	factorInt.Lsh(factorInt, power)
	factorInt.Mod(factorInt, Modulus())

	var refFactorInt big.Int
	inversionCorrectionFactor := {{.ElementName}}{
		{{- range $i := .NbWordsIndexesFull }}
		inversionCorrectionFactorWord{{$i}},
		{{- end}}
	}
	inversionCorrectionFactor.toBigInt(&refFactorInt)

	if refFactorInt.Cmp(factorInt) != 0 {
		t.Error("mismatch")
	}
}

func Test{{.ElementName}}LinearComb(t *testing.T) {
	var x {{.ElementName}}
	var y {{.ElementName}}

	for i := 0; i < 1000; i++ {
		x.SetRandom()
		y.SetRandom()
		testLinearComb(t, &x, mrand.Int63(), &y, mrand.Int63()) //#nosec G404 weak rng is fine here
	}
}

// Probably unnecessary post-dev. In case the output of inv is wrong, this checks whether it's only off by a constant factor.
func Test{{.ElementName}}InversionCorrectionFactor(t *testing.T) {

	// (1/x)/inv(x) = (1/1)/inv(1) ⇔ inv(1) = x inv(x)

	var one {{.ElementName}}
	var oneInv {{.ElementName}}
	one.SetOne()
	oneInv.Inverse(&one)

	for i := 0; i < 100; i++ {
		var x {{.ElementName}}
		var xInv {{.ElementName}}
		x.SetRandom()
		xInv.Inverse(&x)

		x.Mul(&x, &xInv)
		if !x.Equal(&oneInv) {
			t.Error("Correction factor is inconsistent")
		}
	}

	if !oneInv.Equal(&one) {
		var i big.Int
		oneInv.BigInt(&i)	// no montgomery
		i.ModInverse(&i, Modulus())
		var fac {{.ElementName}}
		fac.setBigInt(&i)	// back to montgomery

		var facTimesFac {{.ElementName}}
		facTimesFac.Mul(&fac, &{{.ElementName}}{
			{{- range $i := .NbWordsIndexesFull }}
			inversionCorrectionFactorWord{{$i}},
			{{- end}}
		})

		t.Error("Correction factor is consistently off by", fac, "Should be", facTimesFac)
	}
}

func Test{{.ElementName}}BigNumNeg(t *testing.T) {
	var a {{.ElementName}}
	aHi := negL(&a, 0)
	if !a.IsZero() || aHi != 0 {
		t.Error("-0 != 0")
	}
}

func Test{{.ElementName}}BigNumWMul(t *testing.T) {
	var x {{.ElementName}}

	for i := 0; i < 1000; i++ {
		x.SetRandom()
		w := mrand.Int63() //#nosec G404 weak rng is fine here
		testBigNumWMul(t, &x, w)
	}
}

func Test{{.ElementName}}VeryBigIntConversion(t *testing.T) {
	xHi := mrand.Uint64() //#nosec G404 weak rng is fine here
	var x {{.ElementName}}
	x.SetRandom()
	var xInt big.Int
	x.toVeryBigIntSigned(&xInt, xHi)
	x.assertMatchVeryBigInt(t, xHi, &xInt)
}

type veryBigInt struct {
	asInt big.Int
	low {{.ElementName}}
	hi uint64
}

// genVeryBigIntSigned if sign == 0, no sign is forced
func genVeryBigIntSigned(sign int) gopter.Gen {
	return func(genParams *gopter.GenParameters) *gopter.GenResult {
		var g veryBigInt

		g.low = {{.ElementName}}{
			{{- range $i := .NbWordsIndexesFull}}
			genParams.NextUint64(),
			{{- end}}
		}

		g.hi = genParams.NextUint64()

		if sign < 0 {
			g.hi |= signBitSelector
		} else if sign > 0 {
			g.hi &= ^signBitSelector
		}

		g.low.toVeryBigIntSigned(&g.asInt, g.hi)

		genResult := gopter.NewGenResult(g, gopter.NoShrinker)
		return genResult
	}
}

func Test{{.ElementName}}MontReduce(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	if testing.Short() {
		parameters.MinSuccessfulTests = nbFuzzShort
	} else {
		parameters.MinSuccessfulTests = nbFuzz
	}

	properties := gopter.NewProperties(parameters)

	gen := genVeryBigIntSigned(0)

	properties.Property("Montgomery reduction is correct", prop.ForAll(
		func(g veryBigInt) bool {
			var res {{.ElementName}}
			var resInt big.Int

			montReduce(&resInt, &g.asInt)
			res.montReduceSigned(&g.low, g.hi)

			return res.matchVeryBigInt(0, &resInt) == nil
		},
		gen,
	))



	properties.TestingRun(t, gopter.ConsoleReporter(false))
}

func Test{{.ElementName}}MontReduceMultipleOfR(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	if testing.Short() {
		parameters.MinSuccessfulTests = nbFuzzShort
	} else {
		parameters.MinSuccessfulTests = nbFuzz
	}

	properties := gopter.NewProperties(parameters)

	gen := ggen.UInt64()

	properties.Property("Montgomery reduction is correct", prop.ForAll(
		func(hi uint64) bool {
			var zero, res {{.ElementName}}
			var asInt, resInt big.Int

			zero.toVeryBigIntSigned(&asInt, hi)

			montReduce(&resInt, &asInt)
			res.montReduceSigned(&zero, hi)

			return res.matchVeryBigInt(0, &resInt) == nil
		},
		gen,
	))

	

	properties.TestingRun(t, gopter.ConsoleReporter(false))
}

func Test{{.ElementName}}0Inverse(t *testing.T) {
	var x {{.ElementName}}
	x.Inverse(&x)
	if !x.IsZero() {
		t.Fail()
	}
}

//TODO: Tests like this (update factor related) are common to all fields. Move them to somewhere non-autogen
func TestUpdateFactorSubtraction(t *testing.T) {
	for i := 0; i < 1000; i++ {

		f0, g0 := randomizeUpdateFactors()
		f1, g1 := randomizeUpdateFactors()

		for f0-f1 > 1<<31 || f0-f1 <= -1<<31 {
			f1 /= 2
		}

		for g0-g1 > 1<<31 || g0-g1 <= -1<<31 {
			g1 /= 2
		}

		c0 := updateFactorsCompose(f0, g0)
		c1 := updateFactorsCompose(f1, g1)

		cRes := c0 - c1
		fRes, gRes := updateFactorsDecompose(cRes)

		if fRes != f0-f1 || gRes != g0-g1 {
			t.Error(i)
		}
	}
}

func TestUpdateFactorsDouble(t *testing.T) {
	for i := 0; i < 1000; i++ {
		f, g := randomizeUpdateFactors()

		if f > 1<<30 || f < (-1<<31+1)/2 {
			f /= 2
			if g <= 1<<29 && g >= (-1<<31+1)/4 {
				g *= 2 //g was kept small on f's account. Now that we're halving f, we can double g
			}
		}

		if g > 1<<30 || g < (-1<<31+1)/2 {
			g /= 2

			if f <= 1<<29 && f >= (-1<<31+1)/4 {
				f *= 2 //f was kept small on g's account. Now that we're halving g, we can double f
			}
		}

		c := updateFactorsCompose(f, g)
		cD := c * 2
		fD, gD := updateFactorsDecompose(cD)

		if fD != 2*f || gD != 2*g {
			t.Error(i)
		}
	}
}

func TestUpdateFactorsNeg(t *testing.T) {
	var fMistake bool
	for i := 0; i < 1000; i++ {
		f, g := randomizeUpdateFactors()

		if f == 0x80000000 || g == 0x80000000 {
			// Update factors this large can only have been obtained after 31 iterations and will therefore never be negated
			// We don't have capacity to store -2³¹
			// Repeat this iteration
			i--
			continue
		}

		c := updateFactorsCompose(f, g)
		nc := -c
		nf, ng := updateFactorsDecompose(nc)
		fMistake = fMistake || nf != -f
		if nf != -f || ng != -g {
			t.Errorf("Mismatch iteration #%d:\n%d, %d ->\n %d -> %d ->\n %d, %d\n Inputs in hex: %X, %X",
				i, f, g, c, nc, nf, ng, f, g)
		}
	}
	if fMistake {
		t.Error("Mistake with f detected")
	} else {
		t.Log("All good with f")
	}
}

func TestUpdateFactorsNeg0(t *testing.T) {
	c := updateFactorsCompose(0, 0)
	t.Logf("c(0,0) = %X", c)
	cn := -c

	if c != cn {
		t.Error("Negation of zero update factors should yield the same result.")
	}
}

func TestUpdateFactorDecomposition(t *testing.T) {
	var negSeen bool

	for i := 0; i < 1000; i++ {

		f, g := randomizeUpdateFactors()

		if f <= -(1<<31) || f > 1<<31 {
			t.Fatal("f out of range")
		}

		negSeen = negSeen || f < 0

		c := updateFactorsCompose(f, g)

		fBack, gBack := updateFactorsDecompose(c)

		if f != fBack || g != gBack {
			t.Errorf("(%d, %d) -> %d -> (%d, %d)\n", f, g, c, fBack, gBack)
		}
	}

	if !negSeen {
		t.Fatal("No negative f factors")
	}
}

func TestUpdateFactorInitialValues(t *testing.T) {

	f0, g0 := updateFactorsDecompose(updateFactorIdentityMatrixRow0)
	f1, g1 := updateFactorsDecompose(updateFactorIdentityMatrixRow1)

	if f0 != 1 || g0 != 0 || f1 != 0 || g1 != 1 {
		t.Error("Update factor initial value constants are incorrect")
	}
}

func TestUpdateFactorsRandomization(t *testing.T) {
	var maxLen int

	//t.Log("|f| + |g| is not to exceed", 1 << 31)
	for i := 0; i < 1000; i++ {
		f, g := randomizeUpdateFactors()
		lf, lg := abs64T32(f), abs64T32(g)
		absSum := lf + lg
		if absSum >= 1<<31 {

			if absSum == 1<<31 {
				maxLen++
			} else {
				t.Error(i, "Sum of absolute values too large, f =", f, ",g =", g, ",|f| + |g| =", absSum)
			}
		}
	}

	if maxLen == 0 {
		t.Error("max len not observed")
	} else {
		t.Log(maxLen, "maxLens observed")
	}
}

func randomizeUpdateFactor(absLimit uint32) int64 {
	const maxSizeLikelihood = 10
	maxSize := mrand.Intn(maxSizeLikelihood) //#nosec G404 weak rng is fine here

	absLimit64 := int64(absLimit)
	var f int64
	switch maxSize {
	case 0:
		f = absLimit64
	case 1:
		f = -absLimit64
	default:
		f = int64(mrand.Uint64()%(2*uint64(absLimit64)+1)) - absLimit64 //#nosec G404 weak rng is fine here
	}

	if f > 1<<31 {
		return 1 << 31
	} else if f < -1<<31+1 {
		return -1<<31 + 1
	}

	return f
}

func abs64T32(f int64) uint32 {
	if f >= 1<<32 || f < -1<<32 {
		panic("f out of range")
	}

	if f < 0 {
		return uint32(-f)
	}
	return uint32(f)
}

func randomizeUpdateFactors() (int64, int64) {
	var f [2]int64
	b := mrand.Int() % 2 //#nosec G404 weak rng is fine here

	f[b] = randomizeUpdateFactor(1 << 31)

	//As per the paper, |f| + |g| \le 2³¹.
	f[1-b] = randomizeUpdateFactor(1<<31 - abs64T32(f[b]))

	//Patching another edge case
	if f[0]+f[1] == -1<<31 {
		b = mrand.Int() % 2 //#nosec G404 weak rng is fine here
		f[b]++
	}

	return f[0], f[1]
}

func testLinearComb(t *testing.T, x *{{.ElementName}}, xC int64, y *{{.ElementName}}, yC int64) {

	var p1 big.Int
	x.toBigInt(&p1)
	p1.Mul(&p1, big.NewInt(xC))

	var p2 big.Int
	y.toBigInt(&p2)
	p2.Mul(&p2, big.NewInt(yC))

	p1.Add(&p1, &p2)
	p1.Mod(&p1, Modulus())
	montReduce(&p1, &p1)

	var z {{.ElementName}}
	z.linearComb(x, xC, y, yC)
	z.assertMatchVeryBigInt(t, 0, &p1)
}

func testBigNumWMul(t *testing.T, a *{{.ElementName}}, c int64) {
	var aHi uint64
	var aTimes {{.ElementName}}
	aHi = aTimes.mulWNonModular(a, c)

	assertMulProduct(t, a, c, &aTimes, aHi)
}

func updateFactorsCompose(f int64, g int64) int64 {
	return f + g<<32
}

var rInv big.Int
func montReduce(res *big.Int, x *big.Int) {
	if rInv.BitLen() == 0 {	// initialization
		rInv.SetUint64(1)
		rInv.Lsh(&rInv, Limbs * 64)
		rInv.ModInverse(&rInv, Modulus())
	}
	res.Mul(x, &rInv)
	res.Mod(res, Modulus())
}

func (z *{{.ElementName}}) toVeryBigIntUnsigned(i *big.Int, xHi uint64) {
	z.toBigInt(i)
	var upperWord big.Int
	upperWord.SetUint64(xHi)
	upperWord.Lsh(&upperWord, Limbs*64)
	i.Add(&upperWord, i)
}

func (z *{{.ElementName}}) toVeryBigIntSigned(i *big.Int, xHi uint64) {
	z.toVeryBigIntUnsigned(i, xHi)
	if signBitSelector&xHi != 0 {
		twosCompModulus := big.NewInt(1)
		twosCompModulus.Lsh(twosCompModulus, (Limbs+1)*64)
		i.Sub(i, twosCompModulus)
	}
}

func assertMulProduct(t *testing.T, x *{{.ElementName}}, c int64, result *{{.ElementName}}, resultHi uint64) big.Int {
	var xInt big.Int
	x.toBigInt(&xInt)

	xInt.Mul(&xInt, big.NewInt(c))

	result.assertMatchVeryBigInt(t, resultHi, &xInt)
	return xInt
}

func approximateRef(x *{{.ElementName}}) uint64 {

	var asInt big.Int
	x.toBigInt(&asInt)
	n := x.BitLen()

	if n <= 64 {
		return asInt.Uint64()
	}

	modulus := big.NewInt(1 << 31)
	var lo big.Int
	lo.Mod(&asInt, modulus)

	modulus.Lsh(modulus, uint(n-64))
	var hi big.Int
	hi.Div(&asInt, modulus)
	hi.Lsh(&hi, 31)

	hi.Add(&hi, &lo)
	return hi.Uint64()
}
{{- end}}
`
