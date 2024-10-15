// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package swu implements the Simplified Shallue-van de Woestijne-Ulas
// method.
package swu

import (
	"gitlab.com/yawning/secp256k1-voi/internal/field"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

//nolint:godot
var (
	// Z = -11
	feZ = func() *field.Element {
		negZ := field.NewElementFromUint64(11)
		return negZ.Negate(negZ)
	}()

	// A' = 0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533
	feA = field.NewElementFromCanonicalHex("0x3f8731abdd661adca08a5558f0f5d272e953d363cb6f0e5d405447c01a444533")

	// B' = 1771
	feB = field.NewElementFromUint64(1771)

	// k_(1,0) = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7
	feK10 = field.NewElementFromCanonicalHex("0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa8c7")

	// k_(1,1) = 0x7d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581
	feK11 = field.NewElementFromCanonicalHex("0x7d3d4c80bc321d5b9f315cea7fd44c5d595d2fc0bf63b92dfff1044f17c6581")

	// k_(1,2) = 0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262
	feK12 = field.NewElementFromCanonicalHex("0x534c328d23f234e6e2a413deca25caece4506144037c40314ecbd0b53d9dd262")

	// k_(1,3) = 0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c
	feK13 = field.NewElementFromCanonicalHex("0x8e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38e38daaaaa88c")

	// k_(2,0) = 0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b
	feK20 = field.NewElementFromCanonicalHex("0xd35771193d94918a9ca34ccbb7b640dd86cd409542f8487d9fe6b745781eb49b")

	// k_(2,1) = 0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14
	feK21 = field.NewElementFromCanonicalHex("0xedadc6f64383dc1df7c4b2d51b54225406d36b641f5e41bbc52a56612a8c6d14")

	// k_(3,0) = 0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c
	feK30 = field.NewElementFromCanonicalHex("0x4bda12f684bda12f684bda12f684bda12f684bda12f684bda12f684b8e38e23c")

	// k_(3,1) = 0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3
	feK31 = field.NewElementFromCanonicalHex("0xc75e0c32d5cb7c0fa9d0a54b12a0a6d5647ab046d686da6fdffc90fc201d71a3")

	// k_(3,2) = 0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931
	feK32 = field.NewElementFromCanonicalHex("0x29a6194691f91a73715209ef6512e576722830a201be2018a765e85a9ecee931")

	// k_(3,3) = 0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84
	feK33 = field.NewElementFromCanonicalHex("0x2f684bda12f684bda12f684bda12f684bda12f684bda12f684bda12f38e38d84")

	// k_(4,0) = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b
	feK40 = field.NewElementFromCanonicalHex("0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffff93b")

	// k_(4,1) = 0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573
	feK41 = field.NewElementFromCanonicalHex("0x7a06534bb8bdb49fd5e9e6632722c2989467c1bfc8e8d978dfb425d2685c2573")

	// k_(4,2) = 0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f
	feK42 = field.NewElementFromCanonicalHex("0x6484aa716545ca2cf3a70c3fa8fe337e0a3d21162f0d6299a7bf8192bfd2a76f")

	feOne = field.NewElement().One()
)

func MapToCurveSimpleSWU(u *field.Element) (*field.Element, *field.Element) {
	// F.2. Simplified SWU method

	// 1. tv1 = u^2
	tv1 := field.NewElement().Square(u)

	// 2. tv1 = Z * tv1
	tv1.Multiply(feZ, tv1)

	// 3. tv2 = tv1^2
	tv2 := field.NewElement().Square(tv1)

	// 4. tv2 = tv2 + tv1
	tv2.Add(tv2, tv1)

	// 5. tv3 = tv2 + 1
	tv3 := field.NewElement().Add(tv2, feOne)

	// 6. tv3 = B * tv3
	tv3.Multiply(feB, tv3)

	// 7. tv4 = CMOV(Z, -tv2, tv2 != 0)
	sel := tv2.IsZero()
	tv2.Negate(tv2) // Ok, tv2 just overwritten in step 9.
	tv4 := field.NewElement().ConditionalSelect(tv2, feZ, sel)

	// 8. tv4 = A * tv4
	tv4.Multiply(feA, tv4)

	// 9. tv2 = tv3^2
	tv2.Square(tv3)

	// 10. tv6 = tv4^2
	tv6 := field.NewElement().Square(tv4)

	// 11. tv5 = A * tv6
	tv5 := field.NewElement().Multiply(feA, tv6)

	// 12. tv2 = tv2 + tv5
	tv2.Add(tv2, tv5)

	// 13. tv2 = tv2 * tv3
	tv2.Multiply(tv2, tv3)

	// 14. tv6 = tv6 * tv4
	tv6.Multiply(tv6, tv4)

	// 15. tv5 = B * tv6
	tv5.Multiply(feB, tv6)

	// 16. tv2 = tv2 + tv5
	tv2.Add(tv2, tv5)

	// 17. x = tv1 * tv3
	x := field.NewElement().Multiply(tv1, tv3)

	// 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)
	y1, isGx1Square := field.NewElement().SqrtRatio(tv2, tv6)

	// 19. y = tv1 * u
	y := field.NewElement().Multiply(tv1, u)

	// 20. y = y * y1
	y.Multiply(y, y1)

	// 21. x = CMOV(x, tv3, is_gx1_square)
	x.ConditionalSelect(x, tv3, isGx1Square)

	// 22. y = CMOV(y, y1, is_gx1_square)
	y.ConditionalSelect(y, y1, isGx1Square)

	// 23. e1 = sgn0(u) == sgn0(y)
	e1 := helpers.Uint64Equal(sgn0(u), sgn0(y))

	// 24. y = CMOV(-y, y, e1)
	y.ConditionalNegate(y, helpers.Uint64IsZero(e1))

	// 25. x = x / tv4
	tv4.Invert(tv4)
	x.Multiply(x, tv4)

	// 26. return (x, y)
	return x, y
}

func IsoMap(X, Y *field.Element) (*field.Element, *field.Element, uint64) { //nolint:gocritic
	XX := field.NewElement().Square(X)
	XXX := field.NewElement().Multiply(XX, X)

	// x = x_num / x_den, where
	//
	// - x_num = k_(1,3) * x'^3 + k_(1,2) * x'^2 + k_(1,1) * x' + k_(1,0)
	// - x_den = x'^2 + k_(2,1) * x' + k_(2,0)

	xNum := field.NewElement().Multiply(feK13, XXX)
	xNum.Add(xNum, field.NewElement().Multiply(feK12, XX))
	xNum.Add(xNum, field.NewElement().Multiply(feK11, X))
	xNum.Add(xNum, feK10)

	xDen := field.NewElement().Multiply(feK21, X)
	xDen.Add(xDen, XX)
	xDen.Add(xDen, feK20)
	xDenIsZero := xDen.IsZero()

	xDen.Invert(xDen)
	x := field.NewElement().Multiply(xNum, xDen)

	// y = y' * y_num / y_den, where
	//
	// - y_num = k_(3,3) * x'^3 + k_(3,2) * x'^2 + k_(3,1) * x' + k_(3,0)
	// - y_den = x'^3 + k_(4,2) * x'^2 + k_(4,1) * x' + k_(4,0)

	yNum := field.NewElement().Multiply(feK33, XXX)
	yNum.Add(yNum, field.NewElement().Multiply(feK32, XX))
	yNum.Add(yNum, field.NewElement().Multiply(feK31, X))
	yNum.Add(yNum, feK30)

	yDen := field.NewElement().Multiply(feK42, XX)
	yDen.Add(yDen, field.NewElement().Multiply(feK41, X))
	yDen.Add(yDen, XXX)
	yDen.Add(yDen, feK40)
	yDenIsZero := yDen.IsZero()

	yDen.Invert(yDen)

	y := field.NewElement().Multiply(yNum, yDen)
	y.Multiply(Y, y)

	return x, y, helpers.Uint64IsZero(xDenIsZero | yDenIsZero)
}

func sgn0(x *field.Element) uint64 {
	// When m == 1, sgn0 can be significantly simplified:
	// 1. return x mod 2
	return x.IsOdd()
}
