package metrics

import (
	"sync"
	"testing"
)

func BenchmarkCounterFloat64(b *testing.B) {
	c := NewCounterFloat64()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		c.Inc(1.0)
	}
}

func BenchmarkCounterFloat64Parallel(b *testing.B) {
	c := NewCounterFloat64()
	b.ResetTimer()
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			for i := 0; i < b.N; i++ {
				c.Inc(1.0)
			}
			wg.Done()
		}()
	}
	wg.Wait()
	if have, want := c.Snapshot().Count(), 10.0*float64(b.N); have != want {
		b.Fatalf("have %f want %f", have, want)
	}
}

func TestCounterFloat64Clear(t *testing.T) {
	c := NewCounterFloat64()
	c.Inc(1.0)
	c.Clear()
	if count := c.Snapshot().Count(); count != 0 {
		t.Errorf("c.Count(): 0 != %v\n", count)
	}
}

func TestCounterFloat64Dec1(t *testing.T) {
	c := NewCounterFloat64()
	c.Dec(1.0)
	if count := c.Snapshot().Count(); count != -1.0 {
		t.Errorf("c.Count(): -1.0 != %v\n", count)
	}
}

func TestCounterFloat64Dec2(t *testing.T) {
	c := NewCounterFloat64()
	c.Dec(2.0)
	if count := c.Snapshot().Count(); count != -2.0 {
		t.Errorf("c.Count(): -2.0 != %v\n", count)
	}
}

func TestCounterFloat64Inc1(t *testing.T) {
	c := NewCounterFloat64()
	c.Inc(1.0)
	if count := c.Snapshot().Count(); count != 1.0 {
		t.Errorf("c.Count(): 1.0 != %v\n", count)
	}
}

func TestCounterFloat64Inc2(t *testing.T) {
	c := NewCounterFloat64()
	c.Inc(2.0)
	if count := c.Snapshot().Count(); count != 2.0 {
		t.Errorf("c.Count(): 2.0 != %v\n", count)
	}
}

func TestCounterFloat64Snapshot(t *testing.T) {
	c := NewCounterFloat64()
	c.Inc(1.0)
	snapshot := c.Snapshot()
	c.Inc(1.0)
	if count := snapshot.Count(); count != 1.0 {
		t.Errorf("c.Count(): 1.0 != %v\n", count)
	}
}

func TestCounterFloat64Zero(t *testing.T) {
	c := NewCounterFloat64()
	if count := c.Snapshot().Count(); count != 0 {
		t.Errorf("c.Count(): 0 != %v\n", count)
	}
}

func TestGetOrRegisterCounterFloat64(t *testing.T) {
	r := NewRegistry()
	NewRegisteredCounterFloat64("foo", r).Inc(47.0)
	if c := GetOrRegisterCounterFloat64("foo", r).Snapshot(); c.Count() != 47.0 {
		t.Fatal(c)
	}
}
