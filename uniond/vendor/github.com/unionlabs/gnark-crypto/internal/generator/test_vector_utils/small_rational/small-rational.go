package small_rational

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"strconv"
	"strings"
)

const Bytes = 64

type SmallRational struct {
	text        string //For debugging purposes
	numerator   big.Int
	denominator big.Int // By convention, denominator == 0 also indicates zero
}

var smallPrimes = []*big.Int{
	big.NewInt(2), big.NewInt(3), big.NewInt(5),
	big.NewInt(7), big.NewInt(11), big.NewInt(13),
}

func bigDivides(p, a *big.Int) bool {
	var remainder big.Int
	remainder.Mod(a, p)
	return remainder.BitLen() == 0
}

func (z *SmallRational) UpdateText() {
	z.text = z.Text(10)
}

func (z *SmallRational) simplify() {

	if z.numerator.BitLen() == 0 || z.denominator.BitLen() == 0 {
		return
	}

	var num, den big.Int

	num.Set(&z.numerator)
	den.Set(&z.denominator)

	for _, p := range smallPrimes {
		for bigDivides(p, &num) && bigDivides(p, &den) {
			num.Div(&num, p)
			den.Div(&den, p)
		}
	}

	if bigDivides(&den, &num) {
		num.Div(&num, &den)
		den.SetInt64(1)
	}

	z.numerator = num
	z.denominator = den

}
func (z *SmallRational) Square(x *SmallRational) *SmallRational {
	var num, den big.Int
	num.Mul(&x.numerator, &x.numerator)
	den.Mul(&x.denominator, &x.denominator)

	z.numerator = num
	z.denominator = den

	z.UpdateText()

	return z
}

func (z *SmallRational) String() string {
	z.text = z.Text(10)
	return z.text
}

func (z *SmallRational) Add(x, y *SmallRational) *SmallRational {
	if x.denominator.BitLen() == 0 {
		*z = *y
	} else if y.denominator.BitLen() == 0 {
		*z = *x
	} else {
		//TODO: Exploit cases where one denom divides the other
		var numDen, denNum big.Int
		numDen.Mul(&x.numerator, &y.denominator)
		denNum.Mul(&x.denominator, &y.numerator)

		numDen.Add(&denNum, &numDen)
		z.numerator = numDen //to avoid shallow copy problems

		denNum.Mul(&x.denominator, &y.denominator)
		z.denominator = denNum
		z.simplify()
	}

	z.UpdateText()

	return z
}

func (z *SmallRational) IsZero() bool {
	return z.numerator.BitLen() == 0 || z.denominator.BitLen() == 0
}

func (z *SmallRational) Inverse(x *SmallRational) *SmallRational {
	if x.IsZero() {
		*z = *x
	} else {
		*z = SmallRational{numerator: x.denominator, denominator: x.numerator}
		z.UpdateText()
	}

	return z
}

func (z *SmallRational) Neg(x *SmallRational) *SmallRational {
	z.numerator.Neg(&x.numerator)
	z.denominator = x.denominator

	if x.text == "" {
		x.UpdateText()
	}

	if x.text[0] == '-' {
		z.text = x.text[1:]
	} else {
		z.text = "-" + x.text
	}

	return z
}

func (z *SmallRational) Double(x *SmallRational) *SmallRational {

	var y big.Int

	if x.denominator.Bit(0) == 0 {
		z.numerator = x.numerator
		y.Rsh(&x.denominator, 1)
		z.denominator = y
	} else {
		y.Lsh(&x.numerator, 1)
		z.numerator = y
		z.denominator = x.denominator
	}

	z.UpdateText()

	return z
}

func (z *SmallRational) Sign() int {
	return z.numerator.Sign() * z.denominator.Sign()
}

func (z *SmallRational) MarshalJSON() ([]byte, error) {
	return []byte(z.String()), nil
}

func (z *SmallRational) UnmarshalJson(data []byte) error {
	_, err := z.SetInterface(string(data))
	return err
}

func (z *SmallRational) Equal(x *SmallRational) bool {
	return z.Cmp(x) == 0
}

func (z *SmallRational) Sub(x, y *SmallRational) *SmallRational {
	var yNeg SmallRational
	yNeg.Neg(y)
	z.Add(x, &yNeg)

	z.UpdateText()
	return z
}

func (z *SmallRational) Cmp(x *SmallRational) int {
	zSign, xSign := z.Sign(), x.Sign()

	if zSign > xSign {
		return 1
	}
	if zSign < xSign {
		return -1
	}

	var Z, X big.Int
	Z.Mul(&z.numerator, &x.denominator)
	X.Mul(&x.numerator, &z.denominator)

	Z.Abs(&Z)
	X.Abs(&X)

	return Z.Cmp(&X) * zSign

}

func BatchInvert(a []SmallRational) []SmallRational {
	res := make([]SmallRational, len(a))
	for i := range a {
		res[i].Inverse(&a[i])
	}
	return res
}

func (z *SmallRational) Mul(x, y *SmallRational) *SmallRational {
	var num, den big.Int

	num.Mul(&x.numerator, &y.numerator)
	den.Mul(&x.denominator, &y.denominator)

	z.numerator = num
	z.denominator = den

	z.simplify()
	z.UpdateText()
	return z
}

func (z *SmallRational) SetOne() *SmallRational {
	return z.SetInt64(1)
}

func (z *SmallRational) SetZero() *SmallRational {
	return z.SetInt64(0)
}

func (z *SmallRational) SetInt64(i int64) *SmallRational {
	z.numerator = *big.NewInt(i)
	z.denominator = *big.NewInt(1)
	z.text = strconv.FormatInt(i, 10)
	return z
}

func (z *SmallRational) SetRandom() (*SmallRational, error) {

	bytes := make([]byte, 1)
	n, err := rand.Read(bytes)
	if err != nil {
		return nil, err
	}
	if n != len(bytes) {
		return nil, fmt.Errorf("%d bytes read instead of %d", n, len(bytes))
	}

	z.numerator = *big.NewInt(int64(bytes[0]%16) - 8)
	z.denominator = *big.NewInt(int64((bytes[0]) / 16))

	z.simplify()
	z.UpdateText()

	return z, nil
}

func (z *SmallRational) SetUint64(i uint64) {
	var num big.Int
	num.SetUint64(i)
	z.numerator = num
	z.denominator = *big.NewInt(1)
	z.text = strconv.FormatUint(i, 10)
}

func (z *SmallRational) IsOne() bool {
	return z.numerator.Cmp(&z.denominator) == 0 && z.denominator.BitLen() != 0
}

func (z *SmallRational) Text(base int) string {

	if z.denominator.BitLen() == 0 {
		return "0"
	}

	if z.denominator.Sign() < 0 {
		var num, den big.Int
		num.Neg(&z.numerator)
		den.Neg(&z.denominator)
		z.numerator = num
		z.denominator = den
	}

	if bigDivides(&z.denominator, &z.numerator) {
		var num big.Int
		num.Div(&z.numerator, &z.denominator)
		z.numerator = num
		z.denominator = *big.NewInt(1)
	}

	numerator := z.numerator.Text(base)

	if z.denominator.IsInt64() && z.denominator.Int64() == 1 {
		return numerator
	}

	return numerator + "/" + z.denominator.Text(base)
}

func (z *SmallRational) Set(x *SmallRational) *SmallRational {
	*z = *x // shallow copy is safe because ops are never in place
	return z
}

func (z *SmallRational) SetInterface(x interface{}) (*SmallRational, error) {

	switch v := x.(type) {
	case *SmallRational:
		*z = *v
	case SmallRational:
		*z = v
	case int64:
		z.SetInt64(v)
	case int:
		z.SetInt64(int64(v))
	case float64:
		asInt := int64(v)
		if float64(asInt) != v {
			return nil, fmt.Errorf("cannot currently parse float")
		}
		z.SetInt64(asInt)
	case string:
		z.text = v
		sep := strings.Split(v, "/")
		switch len(sep) {
		case 1:
			if asInt, err := strconv.Atoi(sep[0]); err == nil {
				z.SetInt64(int64(asInt))
			} else {
				return nil, err
			}
		case 2:
			var err error
			var num, denom int
			num, err = strconv.Atoi(sep[0])
			if err != nil {
				return nil, err
			}
			denom, err = strconv.Atoi(sep[1])
			if err != nil {
				return nil, err
			}
			z.numerator = *big.NewInt(int64(num))
			z.denominator = *big.NewInt(int64(denom))
		default:
			return nil, fmt.Errorf("cannot parse \"%s\"", v)
		}
	default:
		return nil, fmt.Errorf("cannot parse %T", x)
	}

	return z, nil
}

func bigIntToBytesSigned(dst []byte, src big.Int) {
	src.FillBytes(dst[1:])
	dst[0] = 0
	if src.Sign() < 0 {
		dst[0] = 255
	}
}

func (z *SmallRational) Bytes() [Bytes]byte {
	var res [Bytes]byte
	bigIntToBytesSigned(res[:Bytes/2], z.numerator)
	bigIntToBytesSigned(res[Bytes/2:], z.denominator)
	return res
}

func bytesToBigIntSigned(src []byte) big.Int {
	var res big.Int
	res.SetBytes(src[1:])
	if src[0] != 0 {
		res.Neg(&res)
	}
	return res
}

// BigInt returns sets dst to the value of z if it is an integer.
// if z is not an integer, nil is returned.
// if the given dst is nil, the address of the numerator is returned.
// if the given dst is non-nil, it is returned.
func (z *SmallRational) BigInt(dst *big.Int) *big.Int {
	if z.denominator.Cmp(big.NewInt(1)) != 0 {
		return nil
	}
	if dst == nil {
		return &z.numerator
	}
	dst.Set(&z.numerator)
	return dst
}

func (z *SmallRational) SetBytes(b []byte) {
	if len(b) > Bytes/2 {
		z.numerator = bytesToBigIntSigned(b[:Bytes/2])
		z.denominator = bytesToBigIntSigned(b[Bytes/2:])
	} else {
		z.numerator.SetBytes(b)
		z.denominator.SetInt64(1)
	}
	z.simplify()
	z.UpdateText()
}

func Modulus() *big.Int {
	res := big.NewInt(1)
	res.Lsh(res, 64)
	return res
}
