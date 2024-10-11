package aggregation

import (
	"sort"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
)

// ErrInvalidMaxCoverProblem is returned when Maximum Coverage problem was initialized incorrectly.
var ErrInvalidMaxCoverProblem = errors.New("invalid max_cover problem")

// MaxCoverProblem defines Maximum Coverage problem.
//
// Problem is defined as MaxCover(U, S, k): S', where:
// U is a finite set of objects, where |U| = n. Furthermore, let S = {S_1, ..., S_m} be all
// subsets of U, that's their union is equal to U. Then, Maximum Coverage is the problem of
// finding such a collection S' of subsets from S, where |S'| <= k, and union of all subsets in S'
// covering U with maximum cardinality.
//
// The current implementation captures the original MaxCover problem, and the variant where
// additional invariant is enforced: all elements of S' must be disjoint. This comes handy when
// we need to aggregate bitsets, and overlaps are not allowed.
//
// For more details, see:
// "Analysis of the Greedy Approach in Problems of Maximum k-Coverage" by Hochbaum and Pathria.
// https://hochbaum.ieor.berkeley.edu/html/pub/HPathria-max-k-coverage-greedy.pdf
type MaxCoverProblem struct {
	Candidates MaxCoverCandidates
}

// MaxCoverCandidate represents a candidate set to be used in aggregation.
type MaxCoverCandidate struct {
	key       int
	bits      *bitfield.Bitlist
	score     uint64
	processed bool
}

// MaxCoverCandidates is defined to allow group operations (filtering, sorting) on all candidates.
type MaxCoverCandidates []*MaxCoverCandidate

// NewMaxCoverCandidate returns initialized candidate.
func NewMaxCoverCandidate(key int, bits *bitfield.Bitlist) *MaxCoverCandidate {
	return &MaxCoverCandidate{
		key:  key,
		bits: bits,
	}
}

// Cover calculates solution to Maximum k-Cover problem in O(knm), where
// n is number of candidates and m is a length of bitlist in each candidate.
func (mc *MaxCoverProblem) Cover(k int, allowOverlaps bool) (*Aggregation, error) {
	if len(mc.Candidates) == 0 {
		return nil, errors.Wrap(ErrInvalidMaxCoverProblem, "cannot calculate set coverage")
	}
	if len(mc.Candidates) < k {
		k = len(mc.Candidates)
	}

	solution := &Aggregation{
		Coverage: bitfield.NewBitlist(mc.Candidates[0].bits.Len()),
		Keys:     make([]int, 0, k),
	}
	remainingBits, err := mc.Candidates.union()
	if err != nil {
		return nil, err
	}
	if remainingBits == nil {
		return nil, errors.Wrap(ErrInvalidMaxCoverProblem, "empty bitlists")
	}

	for len(solution.Keys) < k && len(mc.Candidates) > 0 {
		// Score candidates against remaining bits.
		// Filter out processed and overlapping (when disallowed).
		// Sort by score in a descending order.
		s, err := mc.Candidates.score(remainingBits)
		if err != nil {
			return nil, err
		}
		s, err = s.filter(solution.Coverage, allowOverlaps)
		if err != nil {
			return nil, err
		}
		s.sort()

		for _, candidate := range mc.Candidates {
			if len(solution.Keys) >= k {
				break
			}
			if !candidate.processed {
				var err error
				solution.Coverage, err = solution.Coverage.Or(*candidate.bits)
				if err != nil {
					return nil, err
				}
				solution.Keys = append(solution.Keys, candidate.key)
				remainingBits, err = remainingBits.And(candidate.bits.Not())
				if err != nil {
					return nil, err
				}
				candidate.processed = true
				break
			}
		}
	}
	return solution, nil
}

// MaxCover finds the k-cover of Maximum Coverage problem.
func MaxCover(candidates []*bitfield.Bitlist64, k int, allowOverlaps bool) (selected, coverage *bitfield.Bitlist64, err error) {
	if len(candidates) == 0 {
		return nil, nil, errors.Wrap(ErrInvalidMaxCoverProblem, "cannot calculate set coverage")
	}
	if len(candidates) < k {
		k = len(candidates)
	}

	// Track usable candidates, and candidates selected for coverage as two bitlists.
	selectedCandidates := bitfield.NewBitlist64(uint64(len(candidates)))
	usableCandidates := bitfield.NewBitlist64(uint64(len(candidates))).Not()

	// Track bits covered so far as a bitlist.
	coveredBits := bitfield.NewBitlist64(candidates[0].Len())
	remainingBits, err := union(candidates)
	if err != nil {
		return nil, nil, err
	}
	if remainingBits == nil {
		return nil, nil, errors.Wrap(ErrInvalidMaxCoverProblem, "empty bitlists")
	}

	attempts := 0
	tmpBitlist := bitfield.NewBitlist64(candidates[0].Len()) // Used as return param for NoAlloc*() methods.
	indices := make([]int, usableCandidates.Count())
	for selectedCandidates.Count() < uint64(k) && usableCandidates.Count() > 0 {
		// Safe-guard, each iteration should come with at least one candidate selected.
		if attempts > k {
			break
		}
		attempts++

		// Greedy select the next best candidate (from usable ones) to cover the remaining bits maximally.
		maxScore := uint64(0)
		bestIdx := uint64(0)
		indices = indices[0:usableCandidates.Count()]
		usableCandidates.NoAllocBitIndices(indices)
		for _, idx := range indices {
			// Score is calculated by taking into account uncovered bits only.
			score := uint64(0)
			if candidates[idx].Len() == remainingBits.Len() {
				var err error
				score, err = candidates[idx].AndCount(remainingBits)
				if err != nil {
					return nil, nil, err
				}
			}

			// Filter out zero-score candidates.
			if score == 0 {
				usableCandidates.SetBitAt(uint64(idx), false)
				continue
			}

			// Filter out overlapping candidates (if overlapping is not allowed).
			wrongLen := coveredBits.Len() != candidates[idx].Len()
			overlaps := func(idx int) (bool, error) {
				o, err := coveredBits.Overlaps(candidates[idx])
				return !allowOverlaps && o, err
			}
			if wrongLen { // Shortcut for wrong length check
				usableCandidates.SetBitAt(uint64(idx), false)
				continue
			} else if o, err := overlaps(idx); err != nil {
				return nil, nil, err
			} else if o {
				usableCandidates.SetBitAt(uint64(idx), false)
				continue
			}

			// Track the candidate with the best score.
			if score > maxScore {
				maxScore = score
				bestIdx = uint64(idx)
			}
		}
		// Process greedy selected candidate.
		if maxScore > 0 {
			if err := coveredBits.NoAllocOr(candidates[bestIdx], coveredBits); err != nil {
				return nil, nil, err
			}
			selectedCandidates.SetBitAt(bestIdx, true)
			candidates[bestIdx].NoAllocNot(tmpBitlist)
			if err := remainingBits.NoAllocAnd(tmpBitlist, remainingBits); err != nil {
				return nil, nil, err
			}
			usableCandidates.SetBitAt(bestIdx, false)
		}
	}
	return selectedCandidates, coveredBits, nil
}

// score updates scores of candidates, taking into account the uncovered elements only.
func (cl *MaxCoverCandidates) score(uncovered bitfield.Bitlist) (*MaxCoverCandidates, error) {
	for i := 0; i < len(*cl); i++ {
		if (*cl)[i].bits.Len() == uncovered.Len() {
			a, err := (*cl)[i].bits.And(uncovered)
			if err != nil {
				return nil, err
			}
			(*cl)[i].score = a.Count()
		}
	}
	return cl, nil
}

// filter removes processed, overlapping and zero-score candidates.
func (cl *MaxCoverCandidates) filter(covered bitfield.Bitlist, allowOverlaps bool) (*MaxCoverCandidates, error) {
	overlaps := func(e bitfield.Bitlist) (bool, error) {
		if !allowOverlaps && covered.Len() == e.Len() {
			return covered.Overlaps(e)
		}
		return false, nil
	}
	cur, end := 0, len(*cl)
	for cur < end {
		e := *(*cl)[cur]
		if e.processed || e.score == 0 {
			(*cl)[cur] = (*cl)[end-1]
			end--
			continue
		} else if o, err := overlaps(*e.bits); err == nil && o {
			(*cl)[cur] = (*cl)[end-1]
			end--
			continue
		} else if err != nil {
			return nil, err
		}

		cur++
	}
	*cl = (*cl)[:end]
	return cl, nil
}

// sort orders candidates by their score, starting from the candidate with the highest score.
func (cl *MaxCoverCandidates) sort() *MaxCoverCandidates {
	sort.Slice(*cl, func(i, j int) bool {
		if (*cl)[i].score == (*cl)[j].score {
			return (*cl)[i].key < (*cl)[j].key
		}
		return (*cl)[i].score > (*cl)[j].score
	})
	return cl
}

// union merges all candidate bitlists using logical OR operator.
func (cl *MaxCoverCandidates) union() (bitfield.Bitlist, error) {
	if len(*cl) == 0 {
		return nil, nil
	}
	if (*cl)[0].bits == nil || (*cl)[0].bits.Len() == 0 {
		return nil, nil
	}
	ret := bitfield.NewBitlist((*cl)[0].bits.Len())
	var err error
	for i := 0; i < len(*cl); i++ {
		if *(*cl)[i].bits != nil && ret.Len() == (*cl)[i].bits.Len() {
			ret, err = ret.Or(*(*cl)[i].bits)
			if err != nil {
				return nil, err
			}
		}
	}
	return ret, nil
}

func union(candidates []*bitfield.Bitlist64) (*bitfield.Bitlist64, error) {
	if len(candidates) == 0 || candidates[0].Len() == 0 {
		return nil, nil
	}
	ret := bitfield.NewBitlist64(candidates[0].Len())
	for _, bl := range candidates {
		if ret.Len() == bl.Len() {
			if err := ret.NoAllocOr(bl, ret); err != nil {
				return nil, err
			}
		}
	}
	return ret, nil
}
