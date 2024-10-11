package validator

import (
	"bytes"

	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

type proposerSyncContributions []*eth.SyncCommitteeContribution

// filterByBlockRoot separates sync aggregate list into a valid group.
// The valid group contains the input block root.
func (cs proposerSyncContributions) filterByBlockRoot(r [32]byte) proposerSyncContributions {
	matchedSyncContributions := make([]*eth.SyncCommitteeContribution, 0, len(cs))
	for _, c := range cs {
		if bytes.Equal(c.BlockRoot, r[:]) {
			matchedSyncContributions = append(matchedSyncContributions, c)
		}
	}
	return matchedSyncContributions
}

// filterBySubIndex separates sync aggregate list into a valid group.
// The valid group contains the matching sub committee index.
func (cs proposerSyncContributions) filterBySubIndex(i uint64) proposerSyncContributions {
	matchedSyncContributions := make([]*eth.SyncCommitteeContribution, 0, len(cs))
	for _, c := range cs {
		if c.SubcommitteeIndex == i {
			matchedSyncContributions = append(matchedSyncContributions, c)
		}
	}
	return matchedSyncContributions
}

// dedup removes duplicate sync contributions (ones with the same bits set on).
// Important: not only exact duplicates are removed, but proper subsets are removed too
// (their known bits are redundant and are already contained in their supersets).
func (cs proposerSyncContributions) dedup() (proposerSyncContributions, error) {
	if len(cs) < 2 {
		return cs, nil
	}
	contributionsBySubIdx := make(map[uint64][]*eth.SyncCommitteeContribution, len(cs))
	for _, c := range cs {
		contributionsBySubIdx[c.SubcommitteeIndex] = append(contributionsBySubIdx[c.SubcommitteeIndex], c)
	}

	uniqContributions := make([]*eth.SyncCommitteeContribution, 0, len(cs))
	for _, cs := range contributionsBySubIdx {
		for i := 0; i < len(cs); i++ {
			a := cs[i]
			for j := i + 1; j < len(cs); j++ {
				b := cs[j]
				if aContainsB, err := a.AggregationBits.Contains(b.AggregationBits); err != nil {
					return nil, err
				} else if aContainsB {
					// a contains b, b is redundant.
					cs[j] = cs[len(cs)-1]
					cs[len(cs)-1] = nil
					cs = cs[:len(cs)-1]
					j--
				} else if bContainsA, err := b.AggregationBits.Contains(a.GetAggregationBits()); err != nil {
					return nil, err
				} else if bContainsA {
					// b contains a, a is redundant.
					cs[i] = cs[len(cs)-1]
					cs[len(cs)-1] = nil
					cs = cs[:len(cs)-1]
					i--
					break
				}
			}
		}
		uniqContributions = append(uniqContributions, cs...)
	}
	return uniqContributions, nil
}

// mostProfitable returns the most profitable sync contribution, the one with the most
// votes (ie. aggregation bits count)
func (cs proposerSyncContributions) mostProfitable() *eth.SyncCommitteeContribution {
	if len(cs) == 0 {
		return nil
	}
	mostProfitable := cs[0]
	if len(cs) < 2 {
		return mostProfitable
	}
	for _, c := range cs[1:] {
		if c.AggregationBits.Count() > mostProfitable.AggregationBits.Count() {
			mostProfitable = c
		}
	}
	return mostProfitable
}
