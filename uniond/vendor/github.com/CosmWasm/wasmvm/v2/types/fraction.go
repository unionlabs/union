package types

type Fraction struct {
	Numerator   int64
	Denominator int64
}

func (f *Fraction) Mul(m int64) Fraction {
	return Fraction{f.Numerator * m, f.Denominator}
}

func (f Fraction) Floor() int64 {
	return f.Numerator / f.Denominator
}

type UFraction struct {
	Numerator   uint64
	Denominator uint64
}

func (f *UFraction) Mul(m uint64) UFraction {
	return UFraction{f.Numerator * m, f.Denominator}
}

func (f UFraction) Floor() uint64 {
	return f.Numerator / f.Denominator
}
