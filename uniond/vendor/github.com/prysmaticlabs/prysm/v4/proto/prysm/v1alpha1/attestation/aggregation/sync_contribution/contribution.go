package sync_contribution

import (
	"github.com/pkg/errors"
	v2 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation"
	"github.com/sirupsen/logrus"
)

const (
	// NaiveAggregation is an aggregation strategy without any optimizations.
	NaiveAggregation SyncContributionAggregationStrategy = "naive"

	// MaxCoverAggregation is a strategy based on Maximum Coverage greedy algorithm.
	MaxCoverAggregation SyncContributionAggregationStrategy = "max_cover"
)

// SyncContributionAggregationStrategy defines SyncContribution aggregation strategy.
type SyncContributionAggregationStrategy string

var _ = logrus.WithField("prefix", "aggregation.sync_contribution")

// Aggregate aggregates sync contributions. The minimal number of sync contributions is returned.
// Aggregation occurs in-place i.e. contents of input array will be modified. Should you need to
// preserve input sync contributions, clone them before aggregating.
func Aggregate(cs []*v2.SyncCommitteeContribution) ([]*v2.SyncCommitteeContribution, error) {
	strategy := NaiveAggregation
	switch strategy {
	case "", NaiveAggregation:
		return naiveSyncContributionAggregation(cs)
	case MaxCoverAggregation:
		// TODO: Implement max cover aggregation for sync contributions.
		return nil, errors.New("no implemented")
	default:
		return nil, errors.Wrapf(aggregation.ErrInvalidStrategy, "%q", strategy)
	}
}
