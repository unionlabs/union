package types

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestInfiniteGasMeter(t *testing.T) {
	t.Parallel()
	meter := NewInfiniteGasMeter()
	require.Equal(t, uint64(math.MaxUint64), meter.Limit())
	require.Equal(t, uint64(math.MaxUint64), meter.GasRemaining())
	require.Equal(t, uint64(0), meter.GasConsumed())
	require.Equal(t, uint64(0), meter.GasConsumedToLimit())
	meter.ConsumeGas(10, "consume 10")
	require.Equal(t, uint64(math.MaxUint64), meter.GasRemaining())
	require.Equal(t, uint64(10), meter.GasConsumed())
	require.Equal(t, uint64(10), meter.GasConsumedToLimit())
	meter.RefundGas(1, "refund 1")
	require.Equal(t, uint64(math.MaxUint64), meter.GasRemaining())
	require.Equal(t, uint64(9), meter.GasConsumed())
	require.False(t, meter.IsPastLimit())
	require.False(t, meter.IsOutOfGas())
	meter.ConsumeGas(Gas(math.MaxUint64/2), "consume half max uint64")
	require.Panics(t, func() { meter.ConsumeGas(Gas(math.MaxUint64/2)+2, "panic") })
	require.Panics(t, func() { meter.RefundGas(meter.GasConsumed()+1, "refund greater than consumed") })
}

func TestGasMeter(t *testing.T) {
	t.Parallel()
	cases := []struct {
		limit Gas
		usage []Gas
	}{
		{10, []Gas{1, 2, 3, 4}},
		{1000, []Gas{40, 30, 20, 10, 900}},
		{100000, []Gas{99999, 1}},
		{100000000, []Gas{50000000, 40000000, 10000000}},
		{65535, []Gas{32768, 32767}},
		{65536, []Gas{32768, 32767, 1}},
	}

	for tcnum, tc := range cases {
		meter := NewGasMeter(tc.limit)
		used := uint64(0)

		for unum, usage := range tc.usage {
			usage := usage
			used += usage
			require.NotPanics(t, func() { meter.ConsumeGas(usage, "") }, "Not exceeded limit but panicked. tc #%d, usage #%d", tcnum, unum)
			require.Equal(t, used, meter.GasConsumed(), "Gas consumption not match. tc #%d, usage #%d", tcnum, unum)
			require.Equal(t, tc.limit-used, meter.GasRemaining(), "Gas left not match. tc #%d, usage #%d", tcnum, unum)
			require.Equal(t, used, meter.GasConsumedToLimit(), "Gas consumption (to limit) not match. tc #%d, usage #%d", tcnum, unum)
			require.False(t, meter.IsPastLimit(), "Not exceeded limit but got IsPastLimit() true")
			if unum < len(tc.usage)-1 {
				require.False(t, meter.IsOutOfGas(), "Not yet at limit but got IsOutOfGas() true")
			} else {
				require.True(t, meter.IsOutOfGas(), "At limit but got IsOutOfGas() false")
			}
		}

		require.Panics(t, func() { meter.ConsumeGas(1, "") }, "Exceeded but not panicked. tc #%d", tcnum)
		require.Equal(t, meter.GasConsumedToLimit(), meter.Limit(), "Gas consumption (to limit) not match limit")
		require.Equal(t, meter.GasConsumed(), meter.Limit()+1, "Gas consumption not match limit+1")
		require.Equal(t, uint64(0), meter.GasRemaining())

		require.NotPanics(t, func() { meter.RefundGas(1, "refund 1") })
		require.Equal(t, meter.GasConsumed(), meter.Limit(), "Gas consumption not match with limit")
		require.Equal(t, uint64(0), meter.GasRemaining())
		require.Panics(t, func() { meter.RefundGas(meter.GasConsumed()+1, "refund greater than consumed") })

		require.NotPanics(t, func() { meter.RefundGas(meter.GasConsumed(), "refund consumed gas") })
		require.Equal(t, meter.Limit(), meter.GasRemaining())

		meter2 := NewGasMeter(math.MaxUint64)
		require.Equal(t, uint64(math.MaxUint64), meter2.GasRemaining())
		meter2.ConsumeGas(Gas(math.MaxUint64/2), "consume half max uint64")
		require.Equal(t, Gas(math.MaxUint64-(math.MaxUint64/2)), meter2.GasRemaining())
		require.Panics(t, func() { meter2.ConsumeGas(Gas(math.MaxUint64/2)+2, "panic") })
	}
}

func TestAddUint64Overflow(t *testing.T) {
	t.Parallel()
	testCases := []struct {
		a, b     uint64
		result   uint64
		overflow bool
	}{
		{0, 0, 0, false},
		{100, 100, 200, false},
		{math.MaxUint64 / 2, math.MaxUint64/2 + 1, math.MaxUint64, false},
		{math.MaxUint64 / 2, math.MaxUint64/2 + 2, 0, true},
	}

	for i, tc := range testCases {
		res, overflow := addUint64Overflow(tc.a, tc.b)
		require.Equal(
			t, tc.overflow, overflow,
			"invalid overflow result; tc: #%d, a: %d, b: %d", i, tc.a, tc.b,
		)
		require.Equal(
			t, tc.result, res,
			"invalid uint64 result; tc: #%d, a: %d, b: %d", i, tc.a, tc.b,
		)
	}
}

func TestTransientGasConfig(t *testing.T) {
	t.Parallel()
	config := TransientGasConfig()
	require.Equal(t, config, GasConfig{
		HasCost:          100,
		DeleteCost:       100,
		ReadCostFlat:     100,
		ReadCostPerByte:  0,
		WriteCostFlat:    200,
		WriteCostPerByte: 3,
		IterNextCostFlat: 3,
	})
}
