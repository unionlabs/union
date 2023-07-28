package time

import (
	"sort"
	"time"
)

// Now returns the current time in UTC with no monotonic component.
func Now() time.Time {
	return Canonical(time.Now())
}

// Canonical returns UTC time with no monotonic component.
// Stripping the monotonic component is for time equality.
// See https://github.com/tendermint/tendermint/pull/2203#discussion_r215064334
func Canonical(t time.Time) time.Time {
	return t.Round(0).UTC()
}

// WeightedTime for computing a median.
type WeightedTime struct {
	Time   time.Time
	Weight int64
}

// NewWeightedTime with time and weight.
func NewWeightedTime(time time.Time, weight int64) *WeightedTime {
	return &WeightedTime{
		Time:   time,
		Weight: weight,
	}
}

// WeightedMedian computes weighted median time for a given array of WeightedTime and the total voting power.
func WeightedMedian(weightedTimes []*WeightedTime, totalVotingPower int64) (res time.Time) {
	median := totalVotingPower / 2

	sort.Slice(weightedTimes, func(i, j int) bool {
		if weightedTimes[i] == nil {
			return false
		}
		if weightedTimes[j] == nil {
			return true
		}
		return weightedTimes[i].Time.UnixNano() < weightedTimes[j].Time.UnixNano()
	})

	for _, weightedTime := range weightedTimes {
		if weightedTime != nil {
			if median <= weightedTime.Weight {
				res = weightedTime.Time
				break
			}
			median -= weightedTime.Weight
		}
	}
	return
}
