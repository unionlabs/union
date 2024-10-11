//go:build (!amd64 && !arm64) || generic
// +build !amd64,!arm64 generic

package bn256

func gfpCarry(a *gfP, head uint64) {
	b := &gfP{}

	var carry uint64
	for i, pi := range p2 {
		ai := a[i]
		bi := ai - pi - carry
		b[i] = bi
		carry = (pi&^ai | (pi|^ai)&bi) >> 63
	}
	carry = carry &^ head

	// If b is negative, then return a.
	// Else return b.
	carry = -carry
	ncarry := ^carry
	for i := 0; i < 4; i++ {
		a[i] = (a[i] & carry) | (b[i] & ncarry)
	}
}

func gfpNeg(c, a *gfP) {
	var carry uint64
	for i, pi := range p2 {
		ai := a[i]
		ci := pi - ai - carry
		c[i] = ci
		carry = (ai&^pi | (ai|^pi)&ci) >> 63
	}
	gfpCarry(c, 0)
}

func gfpAdd(c, a, b *gfP) {
	var carry uint64
	for i, ai := range a {
		bi := b[i]
		ci := ai + bi + carry
		c[i] = ci
		carry = (ai&bi | (ai|bi)&^ci) >> 63
	}
	gfpCarry(c, carry)
}

func gfpSub(c, a, b *gfP) {
	t := &gfP{}

	var carry uint64
	for i, pi := range p2 {
		bi := b[i]
		ti := pi - bi - carry
		t[i] = ti
		carry = (bi&^pi | (bi|^pi)&ti) >> 63
	}

	carry = 0
	for i, ai := range a {
		ti := t[i]
		ci := ai + ti + carry
		c[i] = ci
		carry = (ai&ti | (ai|ti)&^ci) >> 63
	}
	gfpCarry(c, carry)
}

func mul(a, b [4]uint64) [8]uint64 {
	const (
		mask16 uint64 = 0x0000ffff
		mask32 uint64 = 0xffffffff
	)

	var buff [32]uint64
	for i, ai := range a {
		a0, a1, a2, a3 := ai&mask16, (ai>>16)&mask16, (ai>>32)&mask16, ai>>48

		for j, bj := range b {
			b0, b2 := bj&mask32, bj>>32

			off := 4 * (i + j)
			buff[off+0] += a0 * b0
			buff[off+1] += a1 * b0
			buff[off+2] += a2*b0 + a0*b2
			buff[off+3] += a3*b0 + a1*b2
			buff[off+4] += a2 * b2
			buff[off+5] += a3 * b2
		}
	}

	for i := uint(1); i < 4; i++ {
		shift := 16 * i

		var head, carry uint64
		for j := uint(0); j < 8; j++ {
			block := 4 * j

			xi := buff[block]
			yi := (buff[block+i] << shift) + head
			zi := xi + yi + carry
			buff[block] = zi
			carry = (xi&yi | (xi|yi)&^zi) >> 63

			head = buff[block+i] >> (64 - shift)
		}
	}

	return [8]uint64{buff[0], buff[4], buff[8], buff[12], buff[16], buff[20], buff[24], buff[28]}
}

func halfMul(a, b [4]uint64) [4]uint64 {
	const (
		mask16 uint64 = 0x0000ffff
		mask32 uint64 = 0xffffffff
	)

	var buff [18]uint64
	for i, ai := range a {
		a0, a1, a2, a3 := ai&mask16, (ai>>16)&mask16, (ai>>32)&mask16, ai>>48

		for j, bj := range b {
			if i+j > 3 {
				break
			}
			b0, b2 := bj&mask32, bj>>32

			off := 4 * (i + j)
			buff[off+0] += a0 * b0
			buff[off+1] += a1 * b0
			buff[off+2] += a2*b0 + a0*b2
			buff[off+3] += a3*b0 + a1*b2
			buff[off+4] += a2 * b2
			buff[off+5] += a3 * b2
		}
	}

	for i := uint(1); i < 4; i++ {
		shift := 16 * i

		var head, carry uint64
		for j := uint(0); j < 4; j++ {
			block := 4 * j

			xi := buff[block]
			yi := (buff[block+i] << shift) + head
			zi := xi + yi + carry
			buff[block] = zi
			carry = (xi&yi | (xi|yi)&^zi) >> 63

			head = buff[block+i] >> (64 - shift)
		}
	}

	return [4]uint64{buff[0], buff[4], buff[8], buff[12]}
}

func gfpMul(c, a, b *gfP) {
	T := mul(*a, *b)
	m := halfMul([4]uint64{T[0], T[1], T[2], T[3]}, np)
	t := mul([4]uint64{m[0], m[1], m[2], m[3]}, p2)

	var carry uint64
	for i, Ti := range T {
		ti := t[i]
		zi := Ti + ti + carry
		T[i] = zi
		carry = (Ti&ti | (Ti|ti)&^zi) >> 63
	}

	*c = gfP{T[4], T[5], T[6], T[7]}
	gfpCarry(c, carry)
}
