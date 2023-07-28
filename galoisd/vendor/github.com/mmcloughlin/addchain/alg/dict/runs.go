package dict

import (
	"errors"
	"fmt"
	"math/big"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/bigints"
)

// RunsAlgorithm is a custom variant of the dictionary approach that decomposes
// a target into runs of ones. It leverages the observation that building a
// dictionary consisting of runs of 1s of lengths l₁, l₂, ..., l_k can itself
// be reduced to first finding an addition chain for the run lengths. Then from
// this chain we can build a chain for the runs themselves.
type RunsAlgorithm struct {
	seqalg alg.SequenceAlgorithm
}

// NewRunsAlgorithm constructs a RunsAlgorithm using the given sequence
// algorithm to generate addition sequences for run lengths. Note that since run
// lengths are far smaller than the integers themselves, this sequence algorithm
// does not need to be able to handle large integers.
func NewRunsAlgorithm(a alg.SequenceAlgorithm) *RunsAlgorithm {
	return &RunsAlgorithm{
		seqalg: a,
	}
}

func (a RunsAlgorithm) String() string {
	return fmt.Sprintf("runs(%s)", a.seqalg)
}

// FindChain uses the run lengths method to find a chain for n.
func (a RunsAlgorithm) FindChain(n *big.Int) (addchain.Chain, error) {
	// Find the runs in n.
	d := RunLength{T: 0}
	sum := d.Decompose(n)
	runs := sum.Dictionary()

	// Treat the run lengths themselves as a sequence to be solved.
	lengths := []*big.Int{}
	for _, run := range runs {
		length := int64(run.BitLen())
		lengths = append(lengths, big.NewInt(length))
	}

	// Delegate to the sequence algorithm for a solution.
	lc, err := a.seqalg.FindSequence(lengths)
	if err != nil {
		return nil, err
	}

	// Build a dictionary chain from this.
	c, err := RunsChain(lc)
	if err != nil {
		return nil, err
	}

	// Reduce.
	sum, c, err = primitive(sum, c)
	if err != nil {
		return nil, err
	}

	// Build chain for n out of the dictionary.
	dc := dictsumchain(sum)
	c = append(c, dc...)
	bigints.Sort(c)
	c = addchain.Chain(bigints.Unique(c))

	return c, nil
}

// RunsChain takes a chain for the run lengths and generates a chain for the
// runs themselves. That is, if the provided chain is l₁, l₂, ..., l_k then
// the result will contain r(l₁), r(l₂), ..., r(l_k) where r(n) = 2ⁿ - 1.
func RunsChain(lc addchain.Chain) (addchain.Chain, error) {
	p, err := lc.Program()
	if err != nil {
		return nil, err
	}

	c := addchain.New()
	s := map[uint]uint{} // current largest shift of each run length
	for _, op := range p {
		a, b := bigint.MinMax(lc[op.I], lc[op.J])
		if !a.IsUint64() || !b.IsUint64() {
			return nil, errors.New("values in lengths chain are far too large")
		}

		la := uint(a.Uint64())
		lb := uint(b.Uint64())

		rb := bigint.Ones(lb)
		for ; s[lb] < la; s[lb]++ {
			shift := new(big.Int).Lsh(rb, s[lb]+1)
			c = append(c, shift)
		}

		c = append(c, bigint.Ones(la+lb))
	}

	return c, nil
}
