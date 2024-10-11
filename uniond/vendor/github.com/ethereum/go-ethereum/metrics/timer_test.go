package metrics

import (
	"fmt"
	"math"
	"testing"
	"time"
)

func BenchmarkTimer(b *testing.B) {
	tm := NewTimer()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		tm.Update(1)
	}
}

func TestGetOrRegisterTimer(t *testing.T) {
	r := NewRegistry()
	NewRegisteredTimer("foo", r).Update(47)
	if tm := GetOrRegisterTimer("foo", r).Snapshot(); tm.Count() != 1 {
		t.Fatal(tm)
	}
}

func TestTimerExtremes(t *testing.T) {
	tm := NewTimer()
	tm.Update(math.MaxInt64)
	tm.Update(0)
	if stdDev := tm.Snapshot().StdDev(); stdDev != 4.611686018427388e+18 {
		t.Errorf("tm.StdDev(): 4.611686018427388e+18 != %v\n", stdDev)
	}
}

func TestTimerStop(t *testing.T) {
	l := len(arbiter.meters)
	tm := NewTimer()
	if l+1 != len(arbiter.meters) {
		t.Errorf("arbiter.meters: %d != %d\n", l+1, len(arbiter.meters))
	}
	tm.Stop()
	if l != len(arbiter.meters) {
		t.Errorf("arbiter.meters: %d != %d\n", l, len(arbiter.meters))
	}
}

func TestTimerFunc(t *testing.T) {
	var (
		tm         = NewTimer()
		testStart  = time.Now()
		actualTime time.Duration
	)
	tm.Time(func() {
		time.Sleep(50 * time.Millisecond)
		actualTime = time.Since(testStart)
	})
	var (
		drift    = time.Millisecond * 2
		measured = time.Duration(tm.Snapshot().Max())
		ceil     = actualTime + drift
		floor    = actualTime - drift
	)
	if measured > ceil || measured < floor {
		t.Errorf("tm.Max(): %v > %v || %v > %v\n", measured, ceil, measured, floor)
	}
}

func TestTimerZero(t *testing.T) {
	tm := NewTimer().Snapshot()
	if count := tm.Count(); count != 0 {
		t.Errorf("tm.Count(): 0 != %v\n", count)
	}
	if min := tm.Min(); min != 0 {
		t.Errorf("tm.Min(): 0 != %v\n", min)
	}
	if max := tm.Max(); max != 0 {
		t.Errorf("tm.Max(): 0 != %v\n", max)
	}
	if mean := tm.Mean(); mean != 0.0 {
		t.Errorf("tm.Mean(): 0.0 != %v\n", mean)
	}
	if stdDev := tm.StdDev(); stdDev != 0.0 {
		t.Errorf("tm.StdDev(): 0.0 != %v\n", stdDev)
	}
	ps := tm.Percentiles([]float64{0.5, 0.75, 0.99})
	if ps[0] != 0.0 {
		t.Errorf("median: 0.0 != %v\n", ps[0])
	}
	if ps[1] != 0.0 {
		t.Errorf("75th percentile: 0.0 != %v\n", ps[1])
	}
	if ps[2] != 0.0 {
		t.Errorf("99th percentile: 0.0 != %v\n", ps[2])
	}
	if rate1 := tm.Rate1(); rate1 != 0.0 {
		t.Errorf("tm.Rate1(): 0.0 != %v\n", rate1)
	}
	if rate5 := tm.Rate5(); rate5 != 0.0 {
		t.Errorf("tm.Rate5(): 0.0 != %v\n", rate5)
	}
	if rate15 := tm.Rate15(); rate15 != 0.0 {
		t.Errorf("tm.Rate15(): 0.0 != %v\n", rate15)
	}
	if rateMean := tm.RateMean(); rateMean != 0.0 {
		t.Errorf("tm.RateMean(): 0.0 != %v\n", rateMean)
	}
}

func ExampleGetOrRegisterTimer() {
	m := "account.create.latency"
	t := GetOrRegisterTimer(m, nil)
	t.Update(47)
	fmt.Println(t.Snapshot().Max()) // Output: 47
}
