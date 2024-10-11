package teststat

import (
	"math"
	"math/rand"

	"github.com/go-kit/kit/metrics"
)

// PopulateNormalHistogram makes a series of normal random observations into the
// histogram. The number of observations is determined by Count. The randomness
// is determined by Mean, Stdev, and the seed parameter.
//
// This is a low-level function, exported only for metrics that don't perform
// dynamic quantile computation, like a Prometheus Histogram (c.f. Summary). In
// most cases, you don't need to use this function, and can use TestHistogram
// instead.
func PopulateNormalHistogram(h metrics.Histogram, seed int) {
	r := rand.New(rand.NewSource(int64(seed)))
	for i := 0; i < Count; i++ {
		sample := r.NormFloat64()*float64(Stdev) + float64(Mean)
		if sample < 0 {
			sample = 0
		}
		h.Observe(sample)
	}
}

func normalQuantiles() (p50, p90, p95, p99 float64) {
	return nvq(50), nvq(90), nvq(95), nvq(99)
}

func nvq(quantile int) float64 {
	// https://en.wikipedia.org/wiki/Normal_distribution#Quantile_function
	return float64(Mean) + float64(Stdev)*math.Sqrt2*erfinv(2*(float64(quantile)/100)-1)
}

func erfinv(y float64) float64 {
	// https://stackoverflow.com/questions/5971830/need-code-for-inverse-error-function
	if y < -1.0 || y > 1.0 {
		panic("invalid input")
	}

	var (
		a = [4]float64{0.886226899, -1.645349621, 0.914624893, -0.140543331}
		b = [4]float64{-2.118377725, 1.442710462, -0.329097515, 0.012229801}
		c = [4]float64{-1.970840454, -1.624906493, 3.429567803, 1.641345311}
		d = [2]float64{3.543889200, 1.637067800}
	)

	const y0 = 0.7
	var x, z float64

	if math.Abs(y) == 1.0 {
		x = -y * math.Log(0.0)
	} else if y < -y0 {
		z = math.Sqrt(-math.Log((1.0 + y) / 2.0))
		x = -(((c[3]*z+c[2])*z+c[1])*z + c[0]) / ((d[1]*z+d[0])*z + 1.0)
	} else {
		if y < y0 {
			z = y * y
			x = y * (((a[3]*z+a[2])*z+a[1])*z + a[0]) / ((((b[3]*z+b[3])*z+b[1])*z+b[0])*z + 1.0)
		} else {
			z = math.Sqrt(-math.Log((1.0 - y) / 2.0))
			x = (((c[3]*z+c[2])*z+c[1])*z + c[0]) / ((d[1]*z+d[0])*z + 1.0)
		}
		x -= (math.Erf(x) - y) / (2.0 / math.SqrtPi * math.Exp(-x*x))
		x -= (math.Erf(x) - y) / (2.0 / math.SqrtPi * math.Exp(-x*x))
	}

	return x
}
