package fields_bn254

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/math/emulated"
)

type curveF = emulated.Field[emulated.BN254Fp]
type baseEl = emulated.Element[emulated.BN254Fp]

type E2 struct {
	A0, A1 baseEl
}

type Ext2 struct {
	api         frontend.API
	fp          *curveF
	nonResidues map[int]map[int]*E2
}

func NewExt2(api frontend.API) *Ext2 {
	fp, err := emulated.NewField[emulated.BN254Fp](api)
	if err != nil {
		// TODO: we start returning errors when generifying
		panic(err)
	}
	pwrs := map[int]map[int]struct {
		A0 string
		A1 string
	}{
		1: {
			1: {"8376118865763821496583973867626364092589906065868298776909617916018768340080", "16469823323077808223889137241176536799009286646108169935659301613961712198316"},
			2: {"21575463638280843010398324269430826099269044274347216827212613867836435027261", "10307601595873709700152284273816112264069230130616436755625194854815875713954"},
			3: {"2821565182194536844548159561693502659359617185244120367078079554186484126554", "3505843767911556378687030309984248845540243509899259641013678093033130930403"},
			4: {"2581911344467009335267311115468803099551665605076196740867805258568234346338", "19937756971775647987995932169929341994314640652964949448313374472400716661030"},
			5: {"685108087231508774477564247770172212460312782337200605669322048753928464687", "8447204650696766136447902020341177575205426561248465145919723016860428151883"},
		},
		3: {
			1: {"11697423496358154304825782922584725312912383441159505038794027105778954184319", "303847389135065887422783454877609941456349188919719272345083954437860409601"},
			2: {"3772000881919853776433695186713858239009073593817195771773381919316419345261", "2236595495967245188281701248203181795121068902605861227855261137820944008926"},
			3: {"19066677689644738377698246183563772429336693972053703295610958340458742082029", "18382399103927718843559375435273026243156067647398564021675359801612095278180"},
			4: {"5324479202449903542726783395506214481928257762400643279780343368557297135718", "16208900380737693084919495127334387981393726419856888799917914180988844123039"},
			5: {"8941241848238582420466759817324047081148088512956452953208002715982955420483", "10338197737521362862238855242243140895517409139741313354160881284257516364953"},
		},
	}
	nonResidues := make(map[int]map[int]*E2)
	for pwr, v := range pwrs {
		for coeff, v := range v {
			el := E2{emulated.ValueOf[emulated.BN254Fp](v.A0), emulated.ValueOf[emulated.BN254Fp](v.A1)}
			if nonResidues[pwr] == nil {
				nonResidues[pwr] = make(map[int]*E2)
			}
			nonResidues[pwr][coeff] = &el
		}
	}
	return &Ext2{api: api, fp: fp, nonResidues: nonResidues}
}

func (e Ext2) MulByElement(x *E2, y *baseEl) *E2 {
	z0 := e.fp.MulMod(&x.A0, y)
	z1 := e.fp.MulMod(&x.A1, y)
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) MulByConstElement(x *E2, y *big.Int) *E2 {
	z0 := e.fp.MulConst(&x.A0, y)
	z1 := e.fp.MulConst(&x.A1, y)
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) Conjugate(x *E2) *E2 {
	z0 := x.A0
	z1 := e.fp.Neg(&x.A1)
	return &E2{
		A0: z0,
		A1: *z1,
	}
}

func (e Ext2) MulByNonResidueGeneric(x *E2, power, coef int) *E2 {
	y := e.nonResidues[power][coef]
	z := e.Mul(x, y)
	return z
}

// MulByNonResidue return x*(9+u)
func (e Ext2) MulByNonResidue(x *E2) *E2 {
	nine := big.NewInt(9)
	a := e.fp.MulConst(&x.A0, nine)
	a = e.fp.Sub(a, &x.A1)
	b := e.fp.MulConst(&x.A1, nine)
	b = e.fp.Add(b, &x.A0)
	return &E2{
		A0: *a,
		A1: *b,
	}
}

// MulByNonResidue1Power1 returns x*(9+u)^(1*(p^1-1)/6)
func (e Ext2) MulByNonResidue1Power1(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 1, 1)
}

// MulByNonResidue1Power2 returns x*(9+u)^(2*(p^1-1)/6)
func (e Ext2) MulByNonResidue1Power2(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 1, 2)
}

// MulByNonResidue1Power3 returns x*(9+u)^(3*(p^1-1)/6)
func (e Ext2) MulByNonResidue1Power3(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 1, 3)
}

// MulByNonResidue1Power4 returns x*(9+u)^(4*(p^1-1)/6)
func (e Ext2) MulByNonResidue1Power4(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 1, 4)
}

// MulByNonResidue1Power5 returns x*(9+u)^(5*(p^1-1)/6)
func (e Ext2) MulByNonResidue1Power5(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 1, 5)
}

// MulByNonResidue2Power1 returns x*(9+u)^(1*(p^2-1)/6)
func (e Ext2) MulByNonResidue2Power1(x *E2) *E2 {
	element := emulated.ValueOf[emulated.BN254Fp]("21888242871839275220042445260109153167277707414472061641714758635765020556617")
	return &E2{
		A0: *e.fp.MulMod(&x.A0, &element),
		A1: *e.fp.MulMod(&x.A1, &element),
	}
}

// MulByNonResidue2Power2 returns x*(9+u)^(2*(p^2-1)/6)
func (e Ext2) MulByNonResidue2Power2(x *E2) *E2 {
	element := emulated.ValueOf[emulated.BN254Fp]("21888242871839275220042445260109153167277707414472061641714758635765020556616")
	return &E2{
		A0: *e.fp.MulMod(&x.A0, &element),
		A1: *e.fp.MulMod(&x.A1, &element),
	}
}

// MulByNonResidue2Power3 returns x*(9+u)^(3*(p^2-1)/6)
func (e Ext2) MulByNonResidue2Power3(x *E2) *E2 {
	element := emulated.ValueOf[emulated.BN254Fp]("21888242871839275222246405745257275088696311157297823662689037894645226208582")
	return &E2{
		A0: *e.fp.MulMod(&x.A0, &element),
		A1: *e.fp.MulMod(&x.A1, &element),
	}
}

// MulByNonResidue2Power4 returns x*(9+u)^(4*(p^2-1)/6)
func (e Ext2) MulByNonResidue2Power4(x *E2) *E2 {
	element := emulated.ValueOf[emulated.BN254Fp]("2203960485148121921418603742825762020974279258880205651966")
	return &E2{
		A0: *e.fp.MulMod(&x.A0, &element),
		A1: *e.fp.MulMod(&x.A1, &element),
	}
}

// MulByNonResidue2Power5 returns x*(9+u)^(5*(p^2-1)/6)
func (e Ext2) MulByNonResidue2Power5(x *E2) *E2 {
	element := emulated.ValueOf[emulated.BN254Fp]("2203960485148121921418603742825762020974279258880205651967")
	return &E2{
		A0: *e.fp.MulMod(&x.A0, &element),
		A1: *e.fp.MulMod(&x.A1, &element),
	}
}

// MulByNonResidue3Power1 returns x*(9+u)^(1*(p^3-1)/6)
func (e Ext2) MulByNonResidue3Power1(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 3, 1)
}

// MulByNonResidue3Power2 returns x*(9+u)^(2*(p^3-1)/6)
func (e Ext2) MulByNonResidue3Power2(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 3, 2)
}

// MulByNonResidue3Power3 returns x*(9+u)^(3*(p^3-1)/6)
func (e Ext2) MulByNonResidue3Power3(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 3, 3)
}

// MulByNonResidue3Power4 returns x*(9+u)^(4*(p^3-1)/6)
func (e Ext2) MulByNonResidue3Power4(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 3, 4)
}

// MulByNonResidue3Power5 returns x*(9+u)^(5*(p^3-1)/6)
func (e Ext2) MulByNonResidue3Power5(x *E2) *E2 {
	return e.MulByNonResidueGeneric(x, 3, 5)
}

func (e Ext2) Mul(x, y *E2) *E2 {
	a := e.fp.Add(&x.A0, &x.A1)
	b := e.fp.Add(&y.A0, &y.A1)
	a = e.fp.MulMod(a, b)
	b = e.fp.MulMod(&x.A0, &y.A0)
	c := e.fp.MulMod(&x.A1, &y.A1)
	z1 := e.fp.Sub(a, b)
	z1 = e.fp.Sub(z1, c)
	z0 := e.fp.Sub(b, c)
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) Add(x, y *E2) *E2 {
	z0 := e.fp.Add(&x.A0, &y.A0)
	z1 := e.fp.Add(&x.A1, &y.A1)
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) Sub(x, y *E2) *E2 {
	z0 := e.fp.Sub(&x.A0, &y.A0)
	z1 := e.fp.Sub(&x.A1, &y.A1)
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) Neg(x *E2) *E2 {
	z0 := e.fp.Neg(&x.A0)
	z1 := e.fp.Neg(&x.A1)
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) One() *E2 {
	z0 := e.fp.One()
	z1 := e.fp.Zero()
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) Zero() *E2 {
	z0 := e.fp.Zero()
	z1 := e.fp.Zero()
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) IsZero(z *E2) frontend.Variable {
	a0 := e.fp.IsZero(&z.A0)
	a1 := e.fp.IsZero(&z.A1)
	return e.api.And(a0, a1)
}

func (e Ext2) Square(x *E2) *E2 {
	a := e.fp.Add(&x.A0, &x.A1)
	b := e.fp.Sub(&x.A0, &x.A1)
	a = e.fp.MulMod(a, b)
	b = e.fp.MulMod(&x.A0, &x.A1)
	b = e.fp.MulConst(b, big.NewInt(2))
	return &E2{
		A0: *a,
		A1: *b,
	}
}

func (e Ext2) Double(x *E2) *E2 {
	two := big.NewInt(2)
	z0 := e.fp.MulConst(&x.A0, two)
	z1 := e.fp.MulConst(&x.A1, two)
	return &E2{
		A0: *z0,
		A1: *z1,
	}
}

func (e Ext2) AssertIsEqual(x, y *E2) {
	e.fp.AssertIsEqual(&x.A0, &y.A0)
	e.fp.AssertIsEqual(&x.A1, &y.A1)
}

func FromE2(y *bn254.E2) E2 {
	return E2{
		A0: emulated.ValueOf[emulated.BN254Fp](y.A0),
		A1: emulated.ValueOf[emulated.BN254Fp](y.A1),
	}
}

func (e Ext2) Inverse(x *E2) *E2 {
	res, err := e.fp.NewHint(inverseE2Hint, 2, &x.A0, &x.A1)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	inv := E2{
		A0: *res[0],
		A1: *res[1],
	}
	one := e.One()

	// 1 == inv * x
	_one := e.Mul(&inv, x)
	e.AssertIsEqual(one, _one)

	return &inv

}

func (e Ext2) DivUnchecked(x, y *E2) *E2 {
	res, err := e.fp.NewHint(divE2Hint, 2, &x.A0, &x.A1, &y.A0, &y.A1)
	if err != nil {
		// err is non-nil only for invalid number of inputs
		panic(err)
	}

	div := E2{
		A0: *res[0],
		A1: *res[1],
	}

	// x == div * y
	_x := e.Mul(&div, y)
	e.AssertIsEqual(x, _x)

	return &div
}

func (e Ext2) Select(selector frontend.Variable, z1, z0 *E2) *E2 {
	a0 := e.fp.Select(selector, &z1.A0, &z0.A0)
	a1 := e.fp.Select(selector, &z1.A1, &z0.A1)
	return &E2{A0: *a0, A1: *a1}
}

func (e Ext2) Lookup2(s1, s2 frontend.Variable, a, b, c, d *E2) *E2 {
	a0 := e.fp.Lookup2(s1, s2, &a.A0, &b.A0, &c.A0, &d.A0)
	a1 := e.fp.Lookup2(s1, s2, &a.A1, &b.A1, &c.A1, &d.A1)
	return &E2{A0: *a0, A1: *a1}
}
