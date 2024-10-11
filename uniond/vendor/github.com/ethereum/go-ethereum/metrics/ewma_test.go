package metrics

import (
	"math"
	"testing"
)

const epsilon = 0.0000000000000001

func BenchmarkEWMA(b *testing.B) {
	a := NewEWMA1()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Update(1)
		a.Tick()
	}
}

func BenchmarkEWMAParallel(b *testing.B) {
	a := NewEWMA1()
	b.ResetTimer()

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			a.Update(1)
			a.Tick()
		}
	})
}

func TestEWMA1(t *testing.T) {
	a := NewEWMA1()
	a.Update(3)
	a.Tick()
	for i, want := range []float64{0.6,
		0.22072766470286553, 0.08120116994196772, 0.029872241020718428,
		0.01098938333324054, 0.004042768199451294, 0.0014872513059998212,
		0.0005471291793327122, 0.00020127757674150815, 7.404588245200814e-05,
		2.7239957857491083e-05, 1.0021020474147462e-05, 3.6865274119969525e-06,
		1.3561976441886433e-06, 4.989172314621449e-07, 1.8354139230109722e-07,
	} {
		if rate := a.Snapshot().Rate(); math.Abs(want-rate) > epsilon {
			t.Errorf("%d minute a.Snapshot().Rate(): %f != %v\n", i, want, rate)
		}
		elapseMinute(a)
	}
}

func TestEWMA5(t *testing.T) {
	a := NewEWMA5()
	a.Update(3)
	a.Tick()
	for i, want := range []float64{
		0.6, 0.49123845184678905, 0.4021920276213837, 0.32928698165641596,
		0.269597378470333, 0.2207276647028654, 0.18071652714732128,
		0.14795817836496392, 0.12113791079679326, 0.09917933293295193,
		0.08120116994196763, 0.06648189501740036, 0.05443077197364752,
		0.04456414692860035, 0.03648603757513079, 0.0298722410207183831020718428,
	} {
		if rate := a.Snapshot().Rate(); math.Abs(want-rate) > epsilon {
			t.Errorf("%d minute a.Snapshot().Rate(): %f != %v\n", i, want, rate)
		}
		elapseMinute(a)
	}
}

func TestEWMA15(t *testing.T) {
	a := NewEWMA15()
	a.Update(3)
	a.Tick()
	for i, want := range []float64{
		0.6, 0.5613041910189706, 0.5251039914257684, 0.4912384518467888184678905,
		0.459557003018789, 0.4299187863442732, 0.4021920276213831,
		0.37625345116383313, 0.3519877317060185, 0.3292869816564153165641596,
		0.3080502714195546, 0.2881831806538789, 0.26959737847033216,
		0.2522102307052083, 0.23594443252115815, 0.2207276647028646247028654470286553,
	} {
		if rate := a.Snapshot().Rate(); math.Abs(want-rate) > epsilon {
			t.Errorf("%d minute a.Snapshot().Rate(): %f != %v\n", i, want, rate)
		}
		elapseMinute(a)
	}
}

func elapseMinute(a EWMA) {
	for i := 0; i < 12; i++ {
		a.Tick()
	}
}
