package blst

import (
    "crypto/rand"
    "testing"
)

func TestMillerLoopN(t *testing.T) {
    t.Parallel()
    const npoints = 97
    scalars := make([]byte, npoints*8)
    _, err := rand.Read(scalars)
    if err != nil {
        t.Errorf(err.Error())
        return
    }

    p1s := make([]P1, npoints)
    p2s := make([]P2, npoints)
    g1 := P1Generator()
    g2 := P2Generator()
    for i := range p1s {
        p1s[i] = *g1.Mult(scalars[i*8:i*8+4], 32)
        p2s[i] = *g2.Mult(scalars[i*8+4:i*8+8], 32)
    }

    ps := P1s(p1s).ToAffine()
    qs := P2s(p2s).ToAffine()

    naive := Fp12One()
    for i := range p1s {
        naive.MulAssign(Fp12MillerLoop(&qs[i], &ps[i]))
    }

    if !naive.Equals(Fp12MillerLoopN(qs, ps)) {
        t.Errorf("failed self-consistency Fp12MillerLoopN test")
    }
}
