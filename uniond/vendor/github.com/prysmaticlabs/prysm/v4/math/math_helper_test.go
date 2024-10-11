package math_test

import (
	"fmt"
	stdmath "math"
	"math/big"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/math"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestIntegerSquareRoot(t *testing.T) {
	tt := []struct {
		number uint64
		root   uint64
	}{
		{
			number: 20,
			root:   4,
		},
		{
			number: 200,
			root:   14,
		},
		{
			number: 1987,
			root:   44,
		},
		{
			number: 34989843,
			root:   5915,
		},
		{
			number: 97282,
			root:   311,
		},
		{
			number: 1 << 32,
			root:   1 << 16,
		},
		{
			number: (1 << 32) + 1,
			root:   1 << 16,
		},
		{
			number: 1 << 33,
			root:   92681,
		},
		{
			number: 1 << 60,
			root:   1 << 30,
		},
		{
			number: 1 << 53,
			root:   94906265,
		},
		{
			number: 1 << 62,
			root:   1 << 31,
		},
		{
			number: 1024,
			root:   32,
		},
		{
			number: 4,
			root:   2,
		},
		{
			number: 16,
			root:   4,
		},
		{
			number: 5508423000000000,
			root:   74218750,
		},
		{
			number: 4503599761588224,
			root:   67108864,
		},
	}

	for _, testVals := range tt {
		require.Equal(t, testVals.root, math.IntegerSquareRoot(testVals.number))
		require.Equal(t, testVals.root, math.CachedSquareRoot(testVals.number))
	}
}

func TestMath_Div64(t *testing.T) {
	type args struct {
		a uint64
		b uint64
	}
	tests := []struct {
		args args
		res  uint64
		err  bool
	}{
		{args: args{0, 1}, res: 0, err: false},
		{args: args{0, 1}, res: 0},
		{args: args{1, 0}, res: 0, err: true},
		{args: args{1 << 32, 1 << 32}, res: 1},
		{args: args{429496729600, 1 << 32}, res: 100},
		{args: args{9223372036854775808, 1 << 32}, res: 1 << 31},
		{args: args{a: 1 << 32, b: 1 << 32}, res: 1},
		{args: args{9223372036854775808, 1 << 62}, res: 2},
		{args: args{9223372036854775808, 1 << 63}, res: 1},
	}
	for _, tt := range tests {
		got, err := math.Div64(tt.args.a, tt.args.b)
		if tt.err && err == nil {
			t.Errorf("Div64() Expected Error = %v, want error", tt.err)
			continue
		}
		if tt.res != got {
			t.Errorf("Div64() %v, want %v", got, tt.res)
		}
	}
}

func TestMath_Mod(t *testing.T) {
	type args struct {
		a uint64
		b uint64
	}
	tests := []struct {
		args args
		res  uint64
		err  bool
	}{
		{args: args{1, 0}, res: 0, err: true},
		{args: args{0, 1}, res: 0},
		{args: args{1 << 32, 1 << 32}, res: 0},
		{args: args{429496729600, 1 << 32}, res: 0},
		{args: args{9223372036854775808, 1 << 32}, res: 0},
		{args: args{1 << 32, 1 << 32}, res: 0},
		{args: args{9223372036854775808, 1 << 62}, res: 0},
		{args: args{9223372036854775808, 1 << 63}, res: 0},
		{args: args{1 << 32, 17}, res: 1},
		{args: args{1 << 32, 19}, res: (1 << 32) % 19},
		{args: args{stdmath.MaxUint64, stdmath.MaxUint64}, res: 0},
		{args: args{1 << 63, 2}, res: 0},
		{args: args{1<<63 + 1, 2}, res: 1},
	}
	for _, tt := range tests {
		got, err := math.Mod64(tt.args.a, tt.args.b)
		if tt.err && err == nil {
			t.Errorf("Mod64() Expected Error = %v, want error", tt.err)
			continue
		}
		if tt.res != got {
			t.Errorf("Mod64() %v, want %v", got, tt.res)
		}
	}
}

func BenchmarkIntegerSquareRootBelow52Bits(b *testing.B) {
	val := uint64(1 << 33)
	for i := 0; i < b.N; i++ {
		require.Equal(b, uint64(92681), math.IntegerSquareRoot(val))
	}
}

func BenchmarkIntegerSquareRootAbove52Bits(b *testing.B) {
	val := uint64(1 << 62)
	for i := 0; i < b.N; i++ {
		require.Equal(b, uint64(1<<31), math.IntegerSquareRoot(val))
	}
}

func BenchmarkSquareRootEffectiveBalance(b *testing.B) {
	val := uint64(1 << 62)
	for i := 0; i < b.N; i++ {
		require.Equal(b, uint64(1<<31), math.CachedSquareRoot(val))
	}
}

func BenchmarkSquareRootBabylonian(b *testing.B) {
	//Start with 700K validators' effective balance
	val := uint64(22400000000000000)
	for i := 0; i < b.N; i++ {
		sqr := math.CachedSquareRoot(val)
		require.Equal(b, true, sqr^2 <= val)
		require.Equal(b, true, (sqr+1)*(sqr+1) > val)
		val += 10_000_000_000
	}
}

func BenchmarkSquareRootOldWay(b *testing.B) {
	//Start with 700K validators' effective balance
	val := uint64(22400000000000000)
	for i := 0; i < b.N; i++ {
		sqr := math.IntegerSquareRoot(val)
		require.Equal(b, true, sqr^2 <= val)
		require.Equal(b, true, (sqr+1)*(sqr+1) > val)
		val += 10_000_000_000
	}
}

func BenchmarkIntegerSquareRoot_WithDatatable(b *testing.B) {
	val := uint64(1024)
	for i := 0; i < b.N; i++ {
		require.Equal(b, uint64(32), math.IntegerSquareRoot(val))
	}
}

func TestCeilDiv8(t *testing.T) {
	tests := []struct {
		number int
		div8   int
	}{
		{
			number: 20,
			div8:   3,
		},
		{
			number: 200,
			div8:   25,
		},
		{
			number: 1987,
			div8:   249,
		},
		{
			number: 1,
			div8:   1,
		},
		{
			number: 97282,
			div8:   12161,
		},
	}

	for _, tt := range tests {
		require.Equal(t, tt.div8, math.CeilDiv8(tt.number))
	}
}

func TestIsPowerOf2(t *testing.T) {
	tests := []struct {
		a uint64
		b bool
	}{
		{
			a: 2,
			b: true,
		},
		{
			a: 64,
			b: true,
		},
		{
			a: 100,
			b: false,
		},
		{
			a: 1024,
			b: true,
		},
		{
			a: 0,
			b: false,
		},
	}
	for _, tt := range tests {
		require.Equal(t, tt.b, math.IsPowerOf2(tt.a))
	}
}

func TestPowerOf2(t *testing.T) {
	tests := []struct {
		a uint64
		b uint64
	}{
		{
			a: 3,
			b: 8,
		},
		{
			a: 20,
			b: 1048576,
		},
		{
			a: 11,
			b: 2048,
		},
		{
			a: 8,
			b: 256,
		},
	}
	for _, tt := range tests {
		require.Equal(t, tt.b, math.PowerOf2(tt.a))
	}
}

func TestMaxValue(t *testing.T) {
	tests := []struct {
		a      uint64
		b      uint64
		result uint64
	}{
		{
			a:      10,
			b:      8,
			result: 10,
		},
		{
			a:      300,
			b:      256,
			result: 300,
		},
		{
			a:      1200,
			b:      1024,
			result: 1200,
		},
		{
			a:      4500,
			b:      4096,
			result: 4500,
		},
		{
			a:      9999,
			b:      9999,
			result: 9999,
		},
	}
	for _, tt := range tests {
		require.Equal(t, tt.result, math.Max(tt.a, tt.b))
	}
}

func TestMinValue(t *testing.T) {
	tests := []struct {
		a      uint64
		b      uint64
		result uint64
	}{
		{
			a:      10,
			b:      8,
			result: 8,
		},
		{
			a:      300,
			b:      256,
			result: 256,
		},
		{
			a:      1200,
			b:      1024,
			result: 1024,
		},
		{
			a:      4500,
			b:      4096,
			result: 4096,
		},
		{
			a:      9999,
			b:      9999,
			result: 9999,
		},
	}
	for _, tt := range tests {
		require.Equal(t, tt.result, math.Min(tt.a, tt.b))
	}
}

func TestMul64(t *testing.T) {
	type args struct {
		a uint64
		b uint64
	}
	tests := []struct {
		args args
		res  uint64
		err  bool
	}{
		{args: args{0, 1}, res: 0, err: false},
		{args: args{1 << 32, 1}, res: 1 << 32, err: false},
		{args: args{1 << 32, 100}, res: 429496729600, err: false},
		{args: args{1 << 32, 1 << 31}, res: 9223372036854775808, err: false},
		{args: args{1 << 32, 1 << 32}, res: 0, err: true},
		{args: args{1 << 62, 2}, res: 9223372036854775808, err: false},
		{args: args{1 << 62, 4}, res: 0, err: true},
		{args: args{1 << 63, 1}, res: 9223372036854775808, err: false},
		{args: args{1 << 63, 2}, res: 0, err: true},
	}
	for _, tt := range tests {
		got, err := math.Mul64(tt.args.a, tt.args.b)
		if tt.err && err == nil {
			t.Errorf("Mul64() Expected Error = %v, want error", tt.err)
			continue
		}
		if tt.res != got {
			t.Errorf("Mul64() %v, want %v", got, tt.res)
		}
	}
}

func TestAdd64(t *testing.T) {
	type args struct {
		a uint64
		b uint64
	}
	tests := []struct {
		args args
		res  uint64
		err  bool
	}{
		{args: args{0, 1}, res: 1, err: false},
		{args: args{1 << 32, 1}, res: 4294967297, err: false},
		{args: args{1 << 32, 100}, res: 4294967396, err: false},
		{args: args{1 << 31, 1 << 31}, res: 4294967296, err: false},
		{args: args{1 << 63, 1 << 63}, res: 0, err: true},
		{args: args{1 << 63, 1}, res: 9223372036854775809, err: false},
		{args: args{stdmath.MaxUint64, 1}, res: 0, err: true},
		{args: args{stdmath.MaxUint64, 0}, res: stdmath.MaxUint64, err: false},
		{args: args{1 << 63, 2}, res: 9223372036854775810, err: false},
	}
	for _, tt := range tests {
		got, err := math.Add64(tt.args.a, tt.args.b)
		if tt.err && err == nil {
			t.Errorf("Add64() Expected Error = %v, want error", tt.err)
			continue
		}
		if tt.res != got {
			t.Errorf("Add64() %v, want %v", got, tt.res)
		}
	}
}

func TestMath_Sub64(t *testing.T) {
	type args struct {
		a uint64
		b uint64
	}
	tests := []struct {
		args args
		res  uint64
		err  bool
	}{
		{args: args{1, 0}, res: 1},
		{args: args{0, 1}, res: 0, err: true},
		{args: args{1 << 32, 1}, res: 4294967295},
		{args: args{1 << 32, 100}, res: 4294967196},
		{args: args{1 << 31, 1 << 31}, res: 0},
		{args: args{1 << 63, 1 << 63}, res: 0},
		{args: args{1 << 63, 1}, res: 9223372036854775807},
		{args: args{stdmath.MaxUint64, stdmath.MaxUint64}, res: 0},
		{args: args{stdmath.MaxUint64 - 1, stdmath.MaxUint64}, res: 0, err: true},
		{args: args{stdmath.MaxUint64, 0}, res: stdmath.MaxUint64},
		{args: args{1 << 63, 2}, res: 9223372036854775806},
	}
	for _, tt := range tests {
		got, err := math.Sub64(tt.args.a, tt.args.b)
		if tt.err && err == nil {
			t.Errorf("Sub64() Expected Error = %v, want error", tt.err)
			continue
		}
		if tt.res != got {
			t.Errorf("Sub64() %v, want %v", got, tt.res)
		}
	}
}

func TestInt(t *testing.T) {
	tests := []struct {
		arg     uint64
		want    int
		wantErr bool
	}{
		{
			arg:  0,
			want: 0,
		},
		{
			arg:  10000000,
			want: 10000000,
		},
		{
			arg:  stdmath.MaxInt64,
			want: stdmath.MaxInt64,
		},
		{
			arg:     stdmath.MaxInt64 + 1,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(fmt.Sprint(tt.arg), func(t *testing.T) {
			got, err := math.Int(tt.arg)
			if (err != nil) != tt.wantErr {
				t.Errorf("Int() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("Int() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAddInt(t *testing.T) {
	tests := []struct {
		name    string
		args    []int
		want    int
		wantErr bool
	}{
		{
			name: "no overflow",
			args: []int{1, 2, 3, 4, 5},
			want: 15,
		},
		{
			name:    "overflow",
			args:    []int{1, stdmath.MaxInt},
			wantErr: true,
		},
		{
			name:    "underflow",
			args:    []int{-1, stdmath.MinInt},
			wantErr: true,
		},
		{
			name: "max int",
			args: []int{1, stdmath.MaxInt - 1},
			want: stdmath.MaxInt,
		},
		{
			name: "min int",
			args: []int{-1, stdmath.MinInt + 1},
			want: stdmath.MinInt,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := math.AddInt(tt.args...)
			if (err != nil) != tt.wantErr {
				t.Errorf("AddInt() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("AddInt() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestWeiToGwei(t *testing.T) {
	tests := []struct {
		v    *big.Int
		want math.Gwei
	}{
		{big.NewInt(1e9 - 1), 0},
		{big.NewInt(1e9), 1},
		{big.NewInt(1e10), 10},
		{big.NewInt(239489233849348394), 239489233},
	}
	for _, tt := range tests {
		if got := math.WeiToGwei(tt.v); got != tt.want {
			t.Errorf("WeiToGwei() = %v, want %v", got, tt.want)
		}
	}
}

func TestWeiToGwei_CopyOk(t *testing.T) {
	v := big.NewInt(1e9)
	got := math.WeiToGwei(v)

	require.Equal(t, math.Gwei(1), got)
	require.Equal(t, big.NewInt(1e9).Uint64(), v.Uint64())
}
