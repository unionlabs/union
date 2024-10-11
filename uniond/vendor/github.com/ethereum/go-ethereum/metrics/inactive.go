// Copyright 2023 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package metrics

// compile-time checks that interfaces are implemented.
var (
	_ SampleSnapshot    = (*emptySnapshot)(nil)
	_ HistogramSnapshot = (*emptySnapshot)(nil)
	_ CounterSnapshot   = (*emptySnapshot)(nil)
	_ GaugeSnapshot     = (*emptySnapshot)(nil)
	_ MeterSnapshot     = (*emptySnapshot)(nil)
	_ EWMASnapshot      = (*emptySnapshot)(nil)
	_ TimerSnapshot     = (*emptySnapshot)(nil)
)

type emptySnapshot struct{}

func (*emptySnapshot) Count() int64                       { return 0 }
func (*emptySnapshot) Max() int64                         { return 0 }
func (*emptySnapshot) Mean() float64                      { return 0.0 }
func (*emptySnapshot) Min() int64                         { return 0 }
func (*emptySnapshot) Percentile(p float64) float64       { return 0.0 }
func (*emptySnapshot) Percentiles(ps []float64) []float64 { return make([]float64, len(ps)) }
func (*emptySnapshot) Size() int                          { return 0 }
func (*emptySnapshot) StdDev() float64                    { return 0.0 }
func (*emptySnapshot) Sum() int64                         { return 0 }
func (*emptySnapshot) Values() []int64                    { return []int64{} }
func (*emptySnapshot) Variance() float64                  { return 0.0 }
func (*emptySnapshot) Value() int64                       { return 0 }
func (*emptySnapshot) Rate() float64                      { return 0.0 }
func (*emptySnapshot) Rate1() float64                     { return 0.0 }
func (*emptySnapshot) Rate5() float64                     { return 0.0 }
func (*emptySnapshot) Rate15() float64                    { return 0.0 }
func (*emptySnapshot) RateMean() float64                  { return 0.0 }
