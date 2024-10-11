package metrics

import (
	"testing"
	"time"
)

func BenchmarkMeter(b *testing.B) {
	m := NewMeter()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		m.Mark(1)
	}
}
func TestMeter(t *testing.T) {
	m := NewMeter()
	m.Mark(47)
	if v := m.Snapshot().Count(); v != 47 {
		t.Fatalf("have %d want %d", v, 47)
	}
}
func TestGetOrRegisterMeter(t *testing.T) {
	r := NewRegistry()
	NewRegisteredMeter("foo", r).Mark(47)
	if m := GetOrRegisterMeter("foo", r).Snapshot(); m.Count() != 47 {
		t.Fatal(m.Count())
	}
}

func TestMeterDecay(t *testing.T) {
	ma := meterArbiter{
		ticker: time.NewTicker(time.Millisecond),
		meters: make(map[*StandardMeter]struct{}),
	}
	defer ma.ticker.Stop()
	m := newStandardMeter()
	ma.meters[m] = struct{}{}
	m.Mark(1)
	ma.tickMeters()
	rateMean := m.Snapshot().RateMean()
	time.Sleep(100 * time.Millisecond)
	ma.tickMeters()
	if m.Snapshot().RateMean() >= rateMean {
		t.Error("m.RateMean() didn't decrease")
	}
}

func TestMeterNonzero(t *testing.T) {
	m := NewMeter()
	m.Mark(3)
	if count := m.Snapshot().Count(); count != 3 {
		t.Errorf("m.Count(): 3 != %v\n", count)
	}
}

func TestMeterStop(t *testing.T) {
	l := len(arbiter.meters)
	m := NewMeter()
	if l+1 != len(arbiter.meters) {
		t.Errorf("arbiter.meters: %d != %d\n", l+1, len(arbiter.meters))
	}
	m.Stop()
	if l != len(arbiter.meters) {
		t.Errorf("arbiter.meters: %d != %d\n", l, len(arbiter.meters))
	}
}

func TestMeterZero(t *testing.T) {
	m := NewMeter().Snapshot()
	if count := m.Count(); count != 0 {
		t.Errorf("m.Count(): 0 != %v\n", count)
	}
}

func TestMeterRepeat(t *testing.T) {
	m := NewMeter()
	for i := 0; i < 101; i++ {
		m.Mark(int64(i))
	}
	if count := m.Snapshot().Count(); count != 5050 {
		t.Errorf("m.Count(): 5050 != %v\n", count)
	}
	for i := 0; i < 101; i++ {
		m.Mark(int64(i))
	}
	if count := m.Snapshot().Count(); count != 10100 {
		t.Errorf("m.Count(): 10100 != %v\n", count)
	}
}
