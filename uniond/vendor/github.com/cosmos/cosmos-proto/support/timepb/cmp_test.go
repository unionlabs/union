package timepb

import (
	"math"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	durpb "google.golang.org/protobuf/types/known/durationpb"
	tspb "google.golang.org/protobuf/types/known/timestamppb"
	"pgregory.net/rapid"
)

func new(s int64, n int32) *tspb.Timestamp {
	return &tspb.Timestamp{Seconds: s, Nanos: n}
}

func TestIsZero(t *testing.T) {
	tcs := []struct {
		t        *tspb.Timestamp
		expected bool
	}{
		{nil, true},

		{&tspb.Timestamp{}, false},
		{new(0, 0), false},
		{new(1, 0), false},
		{new(0, 1), false},
		{tspb.New(time.Time{}), false},
	}

	for i, tc := range tcs {
		require.Equal(t, tc.expected, IsZero(tc.t), "test_id %d", i)
	}
}

func TestCompare(t *testing.T) {
	tcs := []struct {
		t1       *tspb.Timestamp
		t2       *tspb.Timestamp
		expected int
	}{
		{&tspb.Timestamp{}, &tspb.Timestamp{}, 0},
		{new(1, 1), new(1, 1), 0},
		{new(-1, 1), new(-1, 1), 0},
		{new(231, -5), new(231, -5), 0},

		{new(1, -1), new(1, 0), -1},
		{new(1, -1), new(12, -1), -1},
		{new(-11, -1), new(-1, -1), -1},

		{new(1, -1), new(0, -1), 1},
		{new(1, -1), new(1, -2), 1},
	}
	for i, tc := range tcs {
		r := Compare(tc.t1, tc.t2)
		require.Equal(t, tc.expected, r, "test %d", i)
	}

	// test panics
	tcs2 := []struct {
		t1 *tspb.Timestamp
		t2 *tspb.Timestamp
	}{
		{nil, new(1, 1)},
		{new(1, 1), nil},
		{nil, nil},
	}
	for i, tc := range tcs2 {
		require.Panics(t, func() {
			Compare(tc.t1, tc.t2)
		}, "test-panics %d", i)
	}
}

func TestAddFuzzy(t *testing.T) {
	check := func(t require.TestingT, s, n int64, d time.Duration) {
		t_in := time.Unix(s, n)
		t_expected := tspb.New(t_in.Add(d))
		tb := tspb.New(t_in)
		tbPb := Add(tb, durpb.New(d))
		tbStd := AddStd(tb, d)
		require.Equal(t, *t_expected, *tbStd, "checking pb add")
		require.Equal(t, *t_expected, *tbPb, "checking stdlib add")
	}
	gen := rapid.Int64Range(0, 1<<62)
	genNano := rapid.Int64Range(0, 1e9-1)
	rInt := func(t *rapid.T, label string) int64 { return gen.Draw(t, label) }

	rapid.Check(t, func(t *rapid.T) {
		s, n, d := rInt(t, "sec"), genNano.Draw(t, "nanos"), time.Duration(rInt(t, "dur"))
		check(t, s, n, d)
	})

	check(t, 0, 0, 0)
	check(t, 1, 2, 0)
	check(t, -1, -1, 1)

	require.Nil(t, Add(nil, &durpb.Duration{Seconds: 1}), "Pb works with nil values")
	require.Nil(t, AddStd(nil, time.Second), "Std works with nil values")
}

func TestAddOverflow(t *testing.T) {
	require := require.New(t)
	tb := tspb.Timestamp{
		Seconds: math.MaxInt64,
		Nanos:   1000,
	}
	require.Panics(func() {
		AddStd(&tb, time.Second)
	}, "AddStd should panic on overflow")

	require.Panics(func() {
		Add(&tb, &durpb.Duration{Nanos: second - 1})
	}, "Add should panic on overflow")

	// should panic on underflow

	tb = tspb.Timestamp{
		Seconds: -math.MaxInt64 - 1,
		Nanos:   -1000,
	}
	require.True(tb.Seconds < 0, "sanity check")
	require.Panics(func() {
		tt := AddStd(&tb, -time.Second)
		t.Log(tt)
	}, "AddStd should panic on underflow")

	require.Panics(func() {
		tt := Add(&tb, &durpb.Duration{Nanos: -second + 1})
		t.Log(tt)
	}, "Add should panic on underflow")

}
