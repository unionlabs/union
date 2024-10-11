package time

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestWeightedMedian(t *testing.T) {
	m := make([]*WeightedTime, 3)

	t1 := Now()
	t2 := t1.Add(5 * time.Second)
	t3 := t1.Add(10 * time.Second)

	m[2] = NewWeightedTime(t1, 33) // faulty processes
	m[0] = NewWeightedTime(t2, 40) // correct processes
	m[1] = NewWeightedTime(t3, 27) // correct processes
	totalVotingPower := int64(100)

	median := WeightedMedian(m, totalVotingPower)
	assert.Equal(t, t2, median)
	// median always returns value between values of correct processes
	assert.True(t, true, (median.After(t1) || median.Equal(t1)) &&
		(median.Before(t3) || median.Equal(t3)))

	m[1] = NewWeightedTime(t1, 40) // correct processes
	m[2] = NewWeightedTime(t2, 27) // correct processes
	m[0] = NewWeightedTime(t3, 33) // faulty processes
	totalVotingPower = int64(100)

	median = WeightedMedian(m, totalVotingPower)
	assert.Equal(t, t2, median)
	// median always returns value between values of correct processes
	assert.True(t, true, (median.After(t1) || median.Equal(t1)) &&
		(median.Before(t2) || median.Equal(t2)))

	m = make([]*WeightedTime, 8)
	t4 := t1.Add(15 * time.Second)
	t5 := t1.Add(60 * time.Second)

	m[3] = NewWeightedTime(t1, 10) // correct processes
	m[1] = NewWeightedTime(t2, 10) // correct processes
	m[5] = NewWeightedTime(t2, 10) // correct processes
	m[4] = NewWeightedTime(t3, 23) // faulty processes
	m[0] = NewWeightedTime(t4, 20) // correct processes
	m[7] = NewWeightedTime(t5, 10) // faulty processes
	totalVotingPower = int64(83)

	median = WeightedMedian(m, totalVotingPower)
	assert.Equal(t, t3, median)
	// median always returns value between values of correct processes
	assert.True(t, (median.After(t1) || median.Equal(t1)) &&
		(median.Before(t4) || median.Equal(t4)))
}
