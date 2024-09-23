package sw_bn254

import (
	"errors"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/std/math/emulated"
)

func init() {
	solver.RegisterHint(GetHints()...)
}

// GetHints returns all hint functions used in the package.
func GetHints() []solver.Hint {
	return []solver.Hint{
		millerLoopAndCheckFinalExpHint,
	}
}

func millerLoopAndCheckFinalExpHint(nativeMod *big.Int, nativeInputs, nativeOutputs []*big.Int) error {
	// This follows section 4.3.2 of https://eprint.iacr.org/2024/640.pdf
	return emulated.UnwrapHint(nativeInputs, nativeOutputs,
		func(mod *big.Int, inputs, outputs []*big.Int) error {
			var previous, tmp, x3, cubicNonResiduePower, x, millerLoop, residueWitness, residueWitnessInv, one, root27thOf1 bn254.E12
			var exp1, exp2, rInv, mInv big.Int
			var P bn254.G1Affine
			var Q bn254.G2Affine

			P.X.SetBigInt(inputs[0])
			P.Y.SetBigInt(inputs[1])
			Q.X.A0.SetBigInt(inputs[2])
			Q.X.A1.SetBigInt(inputs[3])
			Q.Y.A0.SetBigInt(inputs[4])
			Q.Y.A1.SetBigInt(inputs[5])

			previous.C0.B0.A0.SetBigInt(inputs[6])
			previous.C0.B0.A1.SetBigInt(inputs[7])
			previous.C0.B1.A0.SetBigInt(inputs[8])
			previous.C0.B1.A1.SetBigInt(inputs[9])
			previous.C0.B2.A0.SetBigInt(inputs[10])
			previous.C0.B2.A1.SetBigInt(inputs[11])
			previous.C1.B0.A0.SetBigInt(inputs[12])
			previous.C1.B0.A1.SetBigInt(inputs[13])
			previous.C1.B1.A0.SetBigInt(inputs[14])
			previous.C1.B1.A1.SetBigInt(inputs[15])
			previous.C1.B2.A0.SetBigInt(inputs[16])
			previous.C1.B2.A1.SetBigInt(inputs[17])

			if previous.IsZero() {
				return errors.New("previous Miller loop result is zero")
			}

			lines := bn254.PrecomputeLines(Q)
			millerLoop, err := bn254.MillerLoopFixedQ(
				[]bn254.G1Affine{P},
				[][2][len(bn254.LoopCounter)]bn254.LineEvaluationAff{lines},
			)
			if err != nil {
				return err
			}

			millerLoop.Mul(&millerLoop, &previous)

			// exp1 = (p^12-1)/3
			exp1.SetString("4030969696062745741797811005853058291874379204406359442560681893891674450106959530046539719647151210908190211459382793062006703141168852426020468083171325367934590379984666859998399967609544754664110191464072930598755441160008826659219834762354786403012110463250131961575955268597858015384895449311534622125256548620283853223733396368939858981844663598065852816056384933498610930035891058807598891752166582271931875150099691598048016175399382213304673796601585080509443902692818733420199004555566113537482054218823936116647313678747500267068559627206777530424029211671772692598157901876223857571299238046741502089890557442500582300718504160740314926185458079985126192563953772118929726791041828902047546977272656240744693339962973939047279285351052107950250121751682659529260304162131862468322644288196213423232132152125277136333208005221619443705106431645884840489295409272576227859206166894626854018093044908314720", 10)
			// root27thOf1 = (0, c010, c011, 0, 0, 0, 0, 0, 0, 0, 0, 0)
			// is a 27-th root of  unity which is necessarily a cubic non-residue
			// since h/r = (p^12-1)/r = 27·l and 3 does not divide l.
			// it was computed as w^((p^12-1)/27) = c2 * w^2 + c8 * w^8 where
			// Fp12 = Fp[w]/w^12-18w^6+82 which is isomorphic to our Fp12 tower
			// then c010 = (c2 + 9 * c8) % p and c011 = c8
			root27thOf1.C0.B1.A0.SetString("9483667112135124394372960210728142145589475128897916459350428495526310884707")
			root27thOf1.C0.B1.A1.SetString("4534159768373982659291990808346042891252278737770656686799127720849666919525")

			if one.Exp(millerLoop, &exp1).IsOne() {
				// residueWitness = millerLoop is a cubic residue
				cubicNonResiduePower.SetOne()
				residueWitness.Set(&millerLoop)
			} else if one.Exp(*millerLoop.Mul(&millerLoop, &root27thOf1), &exp1).IsOne() {
				// residueWitness = millerLoop * root27thOf1 is a cubic residue
				cubicNonResiduePower.Set(&root27thOf1)
				residueWitness.Set(&millerLoop)
			} else {
				// residueWitness = millerLoop * root27thOf1^2 is a cubic residue
				cubicNonResiduePower.Square(&root27thOf1)
				residueWitness.Mul(&millerLoop, &root27thOf1)
			}

			// 1. compute r-th root:
			// Exponentiate to rInv where
			// rInv = 1/r mod (p^12-1)/r
			rInv.SetString("495819184011867778744231927046742333492451180917315223017345540833046880485481720031136878341141903241966521818658471092566752321606779256340158678675679238405722886654128392203338228575623261160538734808887996935946888297414610216445334190959815200956855428635568184508263913274453942864817234480763055154719338281461936129150171789463489422401982681230261920147923652438266934726901346095892093443898852488218812468761027620988447655860644584419583586883569984588067403598284748297179498734419889699245081714359110559679136004228878808158639412436468707589339209058958785568729925402190575720856279605832146553573981587948304340677613460685405477047119496887534881410757668344088436651291444274840864486870663164657544390995506448087189408281061890434467956047582679858345583941396130713046072603335601764495918026585155498301896749919393", 10)
			residueWitness.Exp(residueWitness, &rInv)

			// 2. compute m-th root:
			// where m = (6x + 2 + q^3 - q^2 + q)/(3r)
			// Exponentiate to mInv where
			// mInv = 1/m mod p^12-1
			mInv.SetString("17840267520054779749190587238017784600702972825655245554504342129614427201836516118803396948809179149954197175783449826546445899524065131269177708416982407215963288737761615699967145070776364294542559324079147363363059480104341231360692143673915822421222230661528586799190306058519400019024762424366780736540525310403098758015600523609594113357130678138304964034267260758692953579514899054295817541844330584721967571697039986079722203518034173581264955381924826388858518077894154909963532054519350571947910625755075099598588672669612434444513251495355121627496067454526862754597351094345783576387352673894873931328099247263766690688395096280633426669535619271711975898132416216382905928886703963310231865346128293216316379527200971959980873989485521004596686352787540034457467115536116148612884807380187255514888720048664139404687086409399", 10)
			residueWitness.Exp(residueWitness, &mInv)

			// 3. compute cube root:
			// since gcd(3, (p^12-1)/r) ≠ 1 we use a modified Toneelli-Shanks algorithm
			// see Alg.4 of https://eprint.iacr.org/2024/640.pdf
			// Typo in the paper: p^k-1 = 3^n * s instead of p-1 = 3^r * s
			// where k=12 and n=3 here and exp2 = (s+1)/3
			residueWitnessInv.Inverse(&residueWitness)
			exp2.SetString("149295173928249842288807815031594751550902933496531831205951181255247201855813315927649619246190785589192230054051214557852100116339587126889646966043382421034614458517950624444385183985538694617189266350521219651805757080000326913304438324531658755667115202342597480058368713651772519088329461085612393412046538837788290860138273939590365147475728281409846400594680923462911515927255224400281440435265428973034513894448136725853630228718495637529802733207466114092942366766400693830377740909465411612499335341437923559875826432546203713595131838044695464089778859691547136762894737106526809539677749557286722299625576201574095640767352005953344997266128077036486155280146436004404804695964512181557316554713802082990544197776406442186936269827816744738898152657469728130713344598597476387715653492155415311971560450078713968012341037230430349766855793764662401499603533676762082513303932107208402000670112774382027", 10)
			x.Exp(residueWitness, &exp2)

			// 3^t is ord(x^3 / residueWitness)
			x3.Square(&x).Mul(&x3, &x).Mul(&x3, &residueWitnessInv)
			t := 0
			for !x3.IsOne() {
				t++
				tmp.Square(&x3)
				x3.Mul(&tmp, &x3)
			}

			for t != 0 {
				x.Mul(&x, tmp.Exp(root27thOf1, &exp2))

				// 3^t is ord(x^3 / residueWitness)
				x3.Square(&x).Mul(&x3, &x).Mul(&x3, &residueWitnessInv)
				t = 0
				for !x3.IsOne() {
					t++
					tmp.Square(&x3)
					x3.Mul(&tmp, &x3)
				}
			}

			// x is now the cube root of residueWitness
			residueWitness.Set(&x)
			residueWitnessInv.Inverse(&residueWitness)

			residueWitness.C0.B0.A0.BigInt(outputs[0])
			residueWitness.C0.B0.A1.BigInt(outputs[1])
			residueWitness.C0.B1.A0.BigInt(outputs[2])
			residueWitness.C0.B1.A1.BigInt(outputs[3])
			residueWitness.C0.B2.A0.BigInt(outputs[4])
			residueWitness.C0.B2.A1.BigInt(outputs[5])
			residueWitness.C1.B0.A0.BigInt(outputs[6])
			residueWitness.C1.B0.A1.BigInt(outputs[7])
			residueWitness.C1.B1.A0.BigInt(outputs[8])
			residueWitness.C1.B1.A1.BigInt(outputs[9])
			residueWitness.C1.B2.A0.BigInt(outputs[10])
			residueWitness.C1.B2.A1.BigInt(outputs[11])

			// we also need to return the cubic non-residue power
			cubicNonResiduePower.C0.B0.A0.BigInt(outputs[12])
			cubicNonResiduePower.C0.B0.A1.BigInt(outputs[13])
			cubicNonResiduePower.C0.B1.A0.BigInt(outputs[14])
			cubicNonResiduePower.C0.B1.A1.BigInt(outputs[15])
			cubicNonResiduePower.C0.B2.A0.BigInt(outputs[16])
			cubicNonResiduePower.C0.B2.A1.BigInt(outputs[17])
			cubicNonResiduePower.C1.B0.A0.BigInt(outputs[18])
			cubicNonResiduePower.C1.B0.A1.BigInt(outputs[19])
			cubicNonResiduePower.C1.B1.A0.BigInt(outputs[20])
			cubicNonResiduePower.C1.B1.A1.BigInt(outputs[21])
			cubicNonResiduePower.C1.B2.A0.BigInt(outputs[22])
			cubicNonResiduePower.C1.B2.A1.BigInt(outputs[23])

			return nil
		})
}
