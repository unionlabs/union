package sw_bn254

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
)

// lineEvaluation represents a sparse Fp12 Elmt (result of the line evaluation)
// line: 1 + R0(x/y) + R1(1/y) = 0 instead of R0'*y + R1'*x + R2' = 0 This
// makes the multiplication by lines (MulBy014)
type lineEvaluation struct {
	R0, R1 fields_bn254.E2
}
type lineEvaluations [2][len(bn254.LoopCounter)]*lineEvaluation

func precomputeLines(Q bn254.G2Affine) lineEvaluations {
	var cLines lineEvaluations
	nLines := bn254.PrecomputeLines(Q)
	for j := range cLines[0] {
		cLines[0][j] = &lineEvaluation{
			R0: fields_bn254.FromE2(&nLines[0][j].R0),
			R1: fields_bn254.FromE2(&nLines[0][j].R1),
		}
		cLines[1][j] = &lineEvaluation{
			R0: fields_bn254.FromE2(&nLines[1][j].R0),
			R1: fields_bn254.FromE2(&nLines[1][j].R1),
		}
	}
	return cLines
}

func (p *Pairing) computeLines(Q *g2AffP) lineEvaluations {

	var cLines lineEvaluations
	Qacc := Q
	QNeg := &g2AffP{
		X: Q.X,
		Y: *p.Ext2.Neg(&Q.Y),
	}
	n := len(bn254.LoopCounter)
	Qacc, cLines[0][n-2] = p.doubleStep(Qacc)
	cLines[1][n-3] = p.lineCompute(Qacc, QNeg)
	Qacc, cLines[0][n-3] = p.addStep(Qacc, Q)
	for i := n - 4; i >= 0; i-- {
		switch loopCounter[i] {
		case 0:
			Qacc, cLines[0][i] = p.doubleStep(Qacc)
		case 1:
			Qacc, cLines[0][i], cLines[1][i] = p.doubleAndAddStep(Qacc, Q)
		case -1:
			Qacc, cLines[0][i], cLines[1][i] = p.doubleAndAddStep(Qacc, QNeg)
		default:
			return lineEvaluations{}
		}
	}

	Q1X := p.Ext2.Conjugate(&Q.X)
	Q1X = p.Ext2.MulByNonResidue1Power2(Q1X)
	Q1Y := p.Ext2.Conjugate(&Q.Y)
	Q1Y = p.Ext2.MulByNonResidue1Power3(Q1Y)
	Q1 := &g2AffP{
		X: *Q1X,
		Y: *Q1Y,
	}

	Q2Y := p.Ext2.MulByNonResidue2Power3(&Q.Y)
	Q2Y = p.Ext2.Neg(Q2Y)
	Q2 := &g2AffP{
		X: *p.Ext2.MulByNonResidue2Power2(&Q.X),
		Y: *Q2Y,
	}

	Qacc, cLines[0][n-1] = p.addStep(Qacc, Q1)
	cLines[1][n-1] = p.lineCompute(Qacc, Q2)

	return cLines
}
