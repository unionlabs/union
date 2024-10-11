package metrics

import "testing"

func BenchmarkHistogram(b *testing.B) {
	h := NewHistogram(NewUniformSample(100))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		h.Update(int64(i))
	}
}

func TestGetOrRegisterHistogram(t *testing.T) {
	r := NewRegistry()
	s := NewUniformSample(100)
	NewRegisteredHistogram("foo", r, s).Update(47)
	if h := GetOrRegisterHistogram("foo", r, s).Snapshot(); h.Count() != 1 {
		t.Fatal(h)
	}
}

func TestHistogram10000(t *testing.T) {
	h := NewHistogram(NewUniformSample(100000))
	for i := 1; i <= 10000; i++ {
		h.Update(int64(i))
	}
	testHistogram10000(t, h.Snapshot())
}

func TestHistogramEmpty(t *testing.T) {
	h := NewHistogram(NewUniformSample(100)).Snapshot()
	if count := h.Count(); count != 0 {
		t.Errorf("h.Count(): 0 != %v\n", count)
	}
	if min := h.Min(); min != 0 {
		t.Errorf("h.Min(): 0 != %v\n", min)
	}
	if max := h.Max(); max != 0 {
		t.Errorf("h.Max(): 0 != %v\n", max)
	}
	if mean := h.Mean(); mean != 0.0 {
		t.Errorf("h.Mean(): 0.0 != %v\n", mean)
	}
	if stdDev := h.StdDev(); stdDev != 0.0 {
		t.Errorf("h.StdDev(): 0.0 != %v\n", stdDev)
	}
	ps := h.Percentiles([]float64{0.5, 0.75, 0.99})
	if ps[0] != 0.0 {
		t.Errorf("median: 0.0 != %v\n", ps[0])
	}
	if ps[1] != 0.0 {
		t.Errorf("75th percentile: 0.0 != %v\n", ps[1])
	}
	if ps[2] != 0.0 {
		t.Errorf("99th percentile: 0.0 != %v\n", ps[2])
	}
}

func TestHistogramSnapshot(t *testing.T) {
	h := NewHistogram(NewUniformSample(100000))
	for i := 1; i <= 10000; i++ {
		h.Update(int64(i))
	}
	snapshot := h.Snapshot()
	h.Update(0)
	testHistogram10000(t, snapshot)
}

func testHistogram10000(t *testing.T, h HistogramSnapshot) {
	if count := h.Count(); count != 10000 {
		t.Errorf("h.Count(): 10000 != %v\n", count)
	}
	if min := h.Min(); min != 1 {
		t.Errorf("h.Min(): 1 != %v\n", min)
	}
	if max := h.Max(); max != 10000 {
		t.Errorf("h.Max(): 10000 != %v\n", max)
	}
	if mean := h.Mean(); mean != 5000.5 {
		t.Errorf("h.Mean(): 5000.5 != %v\n", mean)
	}
	if stdDev := h.StdDev(); stdDev != 2886.751331514372 {
		t.Errorf("h.StdDev(): 2886.751331514372 != %v\n", stdDev)
	}
	ps := h.Percentiles([]float64{0.5, 0.75, 0.99})
	if ps[0] != 5000.5 {
		t.Errorf("median: 5000.5 != %v\n", ps[0])
	}
	if ps[1] != 7500.75 {
		t.Errorf("75th percentile: 7500.75 != %v\n", ps[1])
	}
	if ps[2] != 9900.99 {
		t.Errorf("99th percentile: 9900.99 != %v\n", ps[2])
	}
}
