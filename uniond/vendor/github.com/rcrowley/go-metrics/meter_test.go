package metrics

import (
	"math/rand"
	"sync"
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

func BenchmarkMeterParallel(b *testing.B) {
	m := NewMeter()
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			m.Mark(1)
		}
	})
}

// exercise race detector
func TestMeterConcurrency(t *testing.T) {
	rand.Seed(time.Now().Unix())
	ma := meterArbiter{
		ticker: time.NewTicker(time.Millisecond),
		meters: make(map[*StandardMeter]struct{}),
	}
	m := newStandardMeter()
	ma.meters[m] = struct{}{}
	go ma.tick()
	wg := &sync.WaitGroup{}
	reps := 100
	for i := 0; i < reps; i++ {
		wg.Add(1)
		go func(m Meter, wg *sync.WaitGroup) {
			m.Mark(1)
			wg.Done()
		}(m, wg)
		wg.Add(1)
		go func(m Meter, wg *sync.WaitGroup) {
			m.Stop()
			wg.Done()
		}(m, wg)
	}
	wg.Wait()
}

func TestGetOrRegisterMeter(t *testing.T) {
	r := NewRegistry()
	NewRegisteredMeter("foo", r).Mark(47)
	if m := GetOrRegisterMeter("foo", r); 47 != m.Count() {
		t.Fatal(m)
	}
}

func TestMeterDecay(t *testing.T) {
	ma := meterArbiter{
		ticker: time.NewTicker(time.Millisecond),
		meters: make(map[*StandardMeter]struct{}),
	}
	m := newStandardMeter()
	ma.meters[m] = struct{}{}
	go ma.tick()
	m.Mark(1)
	rateMean := m.RateMean()
	time.Sleep(100 * time.Millisecond)
	if m.RateMean() >= rateMean {
		t.Error("m.RateMean() didn't decrease")
	}
}

func TestMeterNonzero(t *testing.T) {
	m := NewMeter()
	m.Mark(3)
	if count := m.Count(); 3 != count {
		t.Errorf("m.Count(): 3 != %v\n", count)
	}
}

func TestMeterStop(t *testing.T) {
	l := len(arbiter.meters)
	m := NewMeter()
	if len(arbiter.meters) != l+1 {
		t.Errorf("arbiter.meters: %d != %d\n", l+1, len(arbiter.meters))
	}
	m.Stop()
	if len(arbiter.meters) != l {
		t.Errorf("arbiter.meters: %d != %d\n", l, len(arbiter.meters))
	}
}

func TestMeterSnapshot(t *testing.T) {
	m := NewMeter()
	m.Mark(1)
	if snapshot := m.Snapshot(); m.RateMean() != snapshot.RateMean() {
		t.Fatal(snapshot)
	}
}

func TestMeterZero(t *testing.T) {
	m := NewMeter()
	if count := m.Count(); 0 != count {
		t.Errorf("m.Count(): 0 != %v\n", count)
	}
}
