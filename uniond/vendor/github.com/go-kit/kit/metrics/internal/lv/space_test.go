package lv

import (
	"strings"
	"testing"
)

func TestSpaceWalkAbort(t *testing.T) {
	s := NewSpace()
	s.Observe("a", LabelValues{"a", "b"}, 1)
	s.Observe("a", LabelValues{"c", "d"}, 2)
	s.Observe("a", LabelValues{"e", "f"}, 4)
	s.Observe("a", LabelValues{"g", "h"}, 8)
	s.Observe("b", LabelValues{"a", "b"}, 16)
	s.Observe("b", LabelValues{"c", "d"}, 32)
	s.Observe("b", LabelValues{"e", "f"}, 64)
	s.Observe("b", LabelValues{"g", "h"}, 128)

	var count int
	s.Walk(func(name string, lvs LabelValues, obs []float64) bool {
		count++
		return false
	})
	if want, have := 1, count; want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestSpaceWalkSums(t *testing.T) {
	s := NewSpace()
	s.Observe("metric_one", LabelValues{}, 1)
	s.Observe("metric_one", LabelValues{}, 2)
	s.Observe("metric_one", LabelValues{"a", "1", "b", "2"}, 4)
	s.Observe("metric_one", LabelValues{"a", "1", "b", "2"}, 8)
	s.Observe("metric_one", LabelValues{}, 16)
	s.Observe("metric_one", LabelValues{"a", "1", "b", "3"}, 32)
	s.Observe("metric_two", LabelValues{}, 64)
	s.Observe("metric_two", LabelValues{}, 128)
	s.Observe("metric_two", LabelValues{"a", "1", "b", "2"}, 256)

	have := map[string]float64{}
	s.Walk(func(name string, lvs LabelValues, obs []float64) bool {
		have[name+" ["+strings.Join(lvs, "")+"]"] += sum(obs)
		return true
	})

	want := map[string]float64{
		"metric_one []":     1 + 2 + 16,
		"metric_one [a1b2]": 4 + 8,
		"metric_one [a1b3]": 32,
		"metric_two []":     64 + 128,
		"metric_two [a1b2]": 256,
	}
	for keystr, wantsum := range want {
		if havesum := have[keystr]; wantsum != havesum {
			t.Errorf("%q: want %.1f, have %.1f", keystr, wantsum, havesum)
		}
		delete(want, keystr)
		delete(have, keystr)
	}
	for keystr, havesum := range have {
		t.Errorf("%q: unexpected observations recorded: %.1f", keystr, havesum)
	}
}

func TestSpaceWalkSkipsEmptyDimensions(t *testing.T) {
	s := NewSpace()
	s.Observe("foo", LabelValues{"bar", "1", "baz", "2"}, 123)

	var count int
	s.Walk(func(name string, lvs LabelValues, obs []float64) bool {
		count++
		return true
	})
	if want, have := 1, count; want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func sum(a []float64) (v float64) {
	for _, f := range a {
		v += f
	}
	return
}
