package multi

import (
	"fmt"
	"testing"

	"github.com/go-kit/kit/metrics"
)

func TestMultiCounter(t *testing.T) {
	c1 := &mockCounter{}
	c2 := &mockCounter{}
	c3 := &mockCounter{}
	mc := NewCounter(c1, c2, c3)

	mc.Add(123)
	mc.Add(456)

	want := "[123 456]"
	for i, m := range []fmt.Stringer{c1, c2, c3} {
		if have := m.String(); want != have {
			t.Errorf("c%d: want %q, have %q", i+1, want, have)
		}
	}
}

func TestMultiGauge(t *testing.T) {
	g1 := &mockGauge{}
	g2 := &mockGauge{}
	g3 := &mockGauge{}
	mg := NewGauge(g1, g2, g3)

	mg.Set(9)
	mg.Set(8)
	mg.Set(7)
	mg.Add(3)

	want := "[9 8 7 10]"
	for i, m := range []fmt.Stringer{g1, g2, g3} {
		if have := m.String(); want != have {
			t.Errorf("g%d: want %q, have %q", i+1, want, have)
		}
	}
}

func TestMultiHistogram(t *testing.T) {
	h1 := &mockHistogram{}
	h2 := &mockHistogram{}
	h3 := &mockHistogram{}
	mh := NewHistogram(h1, h2, h3)

	mh.Observe(1)
	mh.Observe(2)
	mh.Observe(4)
	mh.Observe(8)

	want := "[1 2 4 8]"
	for i, m := range []fmt.Stringer{h1, h2, h3} {
		if have := m.String(); want != have {
			t.Errorf("g%d: want %q, have %q", i+1, want, have)
		}
	}
}

type mockCounter struct {
	obs []float64
}

func (c *mockCounter) Add(delta float64)              { c.obs = append(c.obs, delta) }
func (c *mockCounter) With(...string) metrics.Counter { return c }
func (c *mockCounter) String() string                 { return fmt.Sprintf("%v", c.obs) }

type mockGauge struct {
	obs []float64
}

func (g *mockGauge) Set(value float64)            { g.obs = append(g.obs, value) }
func (g *mockGauge) With(...string) metrics.Gauge { return g }
func (g *mockGauge) String() string               { return fmt.Sprintf("%v", g.obs) }
func (g *mockGauge) Add(delta float64) {
	var value float64
	if len(g.obs) > 0 {
		value = g.obs[len(g.obs)-1] + delta
	} else {
		value = delta
	}
	g.obs = append(g.obs, value)
}

type mockHistogram struct {
	obs []float64
}

func (h *mockHistogram) Observe(value float64)            { h.obs = append(h.obs, value) }
func (h *mockHistogram) With(...string) metrics.Histogram { return h }
func (h *mockHistogram) String() string                   { return fmt.Sprintf("%v", h.obs) }
