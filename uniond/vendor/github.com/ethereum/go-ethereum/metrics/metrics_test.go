package metrics

import (
	"fmt"
	"sync"
	"testing"
	"time"
)

const FANOUT = 128

func TestReadRuntimeValues(t *testing.T) {
	var v runtimeStats
	readRuntimeStats(&v)
	t.Logf("%+v", v)
}

func BenchmarkMetrics(b *testing.B) {
	r := NewRegistry()
	c := NewRegisteredCounter("counter", r)
	cf := NewRegisteredCounterFloat64("counterfloat64", r)
	g := NewRegisteredGauge("gauge", r)
	gf := NewRegisteredGaugeFloat64("gaugefloat64", r)
	h := NewRegisteredHistogram("histogram", r, NewUniformSample(100))
	m := NewRegisteredMeter("meter", r)
	t := NewRegisteredTimer("timer", r)
	RegisterDebugGCStats(r)
	b.ResetTimer()
	ch := make(chan bool)

	wgD := &sync.WaitGroup{}
	/*
		wgD.Add(1)
		go func() {
			defer wgD.Done()
			//log.Println("go CaptureDebugGCStats")
			for {
				select {
				case <-ch:
					//log.Println("done CaptureDebugGCStats")
					return
				default:
					CaptureDebugGCStatsOnce(r)
				}
			}
		}()
	//*/

	wgW := &sync.WaitGroup{}
	/*
		wgW.Add(1)
		go func() {
			defer wgW.Done()
			//log.Println("go Write")
			for {
				select {
				case <-ch:
					//log.Println("done Write")
					return
				default:
					WriteOnce(r, io.Discard)
				}
			}
		}()
	//*/

	wg := &sync.WaitGroup{}
	wg.Add(FANOUT)
	for i := 0; i < FANOUT; i++ {
		go func(i int) {
			defer wg.Done()
			//log.Println("go", i)
			for i := 0; i < b.N; i++ {
				c.Inc(1)
				cf.Inc(1.0)
				g.Update(int64(i))
				gf.Update(float64(i))
				h.Update(int64(i))
				m.Mark(1)
				t.Update(1)
			}
			//log.Println("done", i)
		}(i)
	}
	wg.Wait()
	close(ch)
	wgD.Wait()
	wgW.Wait()
}

func Example() {
	c := NewCounter()
	Register("money", c)
	c.Inc(17)

	// Threadsafe registration
	t := GetOrRegisterTimer("db.get.latency", nil)
	t.Time(func() { time.Sleep(10 * time.Millisecond) })
	t.Update(1)

	fmt.Println(c.Snapshot().Count())
	fmt.Println(t.Snapshot().Min())
	// Output: 17
	// 1
}
