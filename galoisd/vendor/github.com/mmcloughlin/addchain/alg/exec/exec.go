// Package exec implements addition chain algorithm execution.
package exec

import (
	"errors"
	"io/ioutil"
	"log"
	"math/big"
	"runtime"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/internal/bigint"
)

// Result from applying an algorithm to a target.
type Result struct {
	Target    *big.Int
	Algorithm alg.ChainAlgorithm
	Err       error
	Chain     addchain.Chain
	Program   addchain.Program
}

// Execute the algorithm on the target number n.
func Execute(n *big.Int, a alg.ChainAlgorithm) Result {
	r := Result{
		Target:    n,
		Algorithm: a,
	}

	r.Chain, r.Err = a.FindChain(n)
	if r.Err != nil {
		return r
	}

	// Note this also performs validation.
	r.Program, r.Err = r.Chain.Program()
	if r.Err != nil {
		return r
	}

	// Still, verify that it produced what we wanted.
	if !bigint.Equal(r.Chain.End(), n) {
		r.Err = errors.New("did not produce the required value")
	}

	return r
}

// Parallel executes multiple algorithms in parallel.
type Parallel struct {
	limit  int
	logger *log.Logger
}

// NewParallel builds a new parallel executor.
func NewParallel() *Parallel {
	return &Parallel{
		limit:  runtime.NumCPU(),
		logger: log.New(ioutil.Discard, "", 0),
	}
}

// SetConcurrency sets the number of algorithms that may be run in parallel.
func (p *Parallel) SetConcurrency(limit int) {
	p.limit = limit
}

// SetLogger sets logging output.
func (p *Parallel) SetLogger(l *log.Logger) {
	p.logger = l
}

// Execute all algorithms against the provided target.
func (p Parallel) Execute(n *big.Int, as []alg.ChainAlgorithm) []Result {
	rs := make([]Result, len(as))

	// Use buffered channel to limit concurrency.
	type token struct{}
	sem := make(chan token, p.limit)

	for i, a := range as {
		sem <- token{}
		go func(i int, a alg.ChainAlgorithm) {
			p.logger.Printf("start: %s", a)
			rs[i] = Execute(n, a)
			p.logger.Printf("done: %s", a)
			<-sem
		}(i, a)
	}

	// Wait for completion.
	for i := 0; i < p.limit; i++ {
		sem <- token{}
	}

	return rs
}
