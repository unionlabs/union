// Package rangecheck implements range checking gadget
//
// This package chooses the most optimal path for performing range checks:
//   - if the backend supports native range checking and the frontend exports the variables in the proprietary format by implementing [frontend.Rangechecker], then use it directly;
//   - if the backend supports creating a commitment of variables by implementing [frontend.Committer], then we use the log-derivative variant [[Haböck22]] of the product argument as in [[BCG+18]] . [r1cs.NewBuilder] returns a builder which implements this interface;
//   - lacking these, we perform binary decomposition of variable into bits.
//
// [BCG+18]: https://eprint.iacr.org/2018/380
// [Haböck22]: https://eprint.iacr.org/2022/1530
package rangecheck

import (
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

// only for documentation purposes. If we import the package then godoc knows
// how to refer to package r1cs and we get nice links in godoc. We import the
// package anyway in test.
var _ = r1cs.NewBuilder

// New returns a new range checker depending on the frontend capabilities.
func New(api frontend.API) frontend.Rangechecker {
	if rc, ok := api.(frontend.Rangechecker); ok {
		return rc
	}
	if _, ok := api.(frontend.Committer); ok {
		return newCommitRangechecker(api)
	}
	return plainChecker{api: api}
}

// GetHints returns all hints used in this package
func GetHints() []solver.Hint {
	return []solver.Hint{DecomposeHint}
}
