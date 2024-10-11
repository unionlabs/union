package metrics

import (
	"sync"
	"testing"
)

func BenchmarkGaugeFloat64(b *testing.B) {
	g := NewGaugeFloat64()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		g.Update(float64(i))
	}
}

func BenchmarkGaugeFloat64Parallel(b *testing.B) {
	c := NewGaugeFloat64()
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			for i := 0; i < b.N; i++ {
				c.Update(float64(i))
			}
			wg.Done()
		}()
	}
	wg.Wait()
	if have, want := c.Snapshot().Value(), float64(b.N-1); have != want {
		b.Fatalf("have %f want %f", have, want)
	}
}

func TestGaugeFloat64Snapshot(t *testing.T) {
	g := NewGaugeFloat64()
	g.Update(47.0)
	snapshot := g.Snapshot()
	g.Update(float64(0))
	if v := snapshot.Value(); 47.0 != v {
		t.Errorf("g.Value(): 47.0 != %v\n", v)
	}
}

func TestGetOrRegisterGaugeFloat64(t *testing.T) {
	r := NewRegistry()
	NewRegisteredGaugeFloat64("foo", r).Update(47.0)
	t.Logf("registry: %v", r)
	if g := GetOrRegisterGaugeFloat64("foo", r).Snapshot(); 47.0 != g.Value() {
		t.Fatal(g)
	}
}
