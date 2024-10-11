package metrics

import (
	"fmt"
	"sync"
	"testing"
)

func BenchmarkRegistry(b *testing.B) {
	r := NewRegistry()
	r.Register("foo", NewCounter())
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		r.Each(func(string, interface{}) {})
	}
}

func BenchmarkHugeRegistry(b *testing.B) {
	r := NewRegistry()
	for i := 0; i < 10000; i++ {
		r.Register(fmt.Sprintf("foo%07d", i), NewCounter())
	}
	v := make([]string, 10000)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		v := v[:0]
		r.Each(func(k string, _ interface{}) {
			v = append(v, k)
		})
	}
}

func BenchmarkRegistryParallel(b *testing.B) {
	r := NewRegistry()
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			r.GetOrRegister("foo", NewCounter())
		}
	})
}

func TestRegistry(t *testing.T) {
	r := NewRegistry()
	r.Register("foo", NewCounter())
	i := 0
	r.Each(func(name string, iface interface{}) {
		i++
		if "foo" != name {
			t.Fatal(name)
		}
		if _, ok := iface.(Counter); !ok {
			t.Fatal(iface)
		}
	})
	if 1 != i {
		t.Fatal(i)
	}
	r.Unregister("foo")
	i = 0
	r.Each(func(string, interface{}) { i++ })
	if 0 != i {
		t.Fatal(i)
	}
}

func TestRegistryDuplicate(t *testing.T) {
	r := NewRegistry()
	if err := r.Register("foo", NewCounter()); nil != err {
		t.Fatal(err)
	}
	if err := r.Register("foo", NewGauge()); nil == err {
		t.Fatal(err)
	}
	i := 0
	r.Each(func(name string, iface interface{}) {
		i++
		if _, ok := iface.(Counter); !ok {
			t.Fatal(iface)
		}
	})
	if 1 != i {
		t.Fatal(i)
	}
}

func TestRegistryGet(t *testing.T) {
	r := NewRegistry()
	r.Register("foo", NewCounter())
	if count := r.Get("foo").(Counter).Count(); 0 != count {
		t.Fatal(count)
	}
	r.Get("foo").(Counter).Inc(1)
	if count := r.Get("foo").(Counter).Count(); 1 != count {
		t.Fatal(count)
	}
}

func TestRegistryGetOrRegister(t *testing.T) {
	r := NewRegistry()

	// First metric wins with GetOrRegister
	_ = r.GetOrRegister("foo", NewCounter())
	m := r.GetOrRegister("foo", NewGauge())
	if _, ok := m.(Counter); !ok {
		t.Fatal(m)
	}

	i := 0
	r.Each(func(name string, iface interface{}) {
		i++
		if name != "foo" {
			t.Fatal(name)
		}
		if _, ok := iface.(Counter); !ok {
			t.Fatal(iface)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}
}

func TestRegistryGetOrRegisterWithLazyInstantiation(t *testing.T) {
	r := NewRegistry()

	// First metric wins with GetOrRegister
	_ = r.GetOrRegister("foo", NewCounter)
	m := r.GetOrRegister("foo", NewGauge)
	if _, ok := m.(Counter); !ok {
		t.Fatal(m)
	}

	i := 0
	r.Each(func(name string, iface interface{}) {
		i++
		if name != "foo" {
			t.Fatal(name)
		}
		if _, ok := iface.(Counter); !ok {
			t.Fatal(iface)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}
}

func TestRegistryUnregister(t *testing.T) {
	l := len(arbiter.meters)
	r := NewRegistry()
	r.Register("foo", NewCounter())
	r.Register("bar", NewMeter())
	r.Register("baz", NewTimer())
	if len(arbiter.meters) != l+2 {
		t.Errorf("arbiter.meters: %d != %d\n", l+2, len(arbiter.meters))
	}
	r.Unregister("foo")
	r.Unregister("bar")
	r.Unregister("baz")
	if len(arbiter.meters) != l {
		t.Errorf("arbiter.meters: %d != %d\n", l+2, len(arbiter.meters))
	}
}

func TestPrefixedChildRegistryGetOrRegister(t *testing.T) {
	r := NewRegistry()
	pr := NewPrefixedChildRegistry(r, "prefix.")

	_ = pr.GetOrRegister("foo", NewCounter())

	i := 0
	r.Each(func(name string, m interface{}) {
		i++
		if name != "prefix.foo" {
			t.Fatal(name)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}
}

func TestPrefixedRegistryGetOrRegister(t *testing.T) {
	r := NewPrefixedRegistry("prefix.")

	_ = r.GetOrRegister("foo", NewCounter())

	i := 0
	r.Each(func(name string, m interface{}) {
		i++
		if name != "prefix.foo" {
			t.Fatal(name)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}
}

func TestPrefixedRegistryRegister(t *testing.T) {
	r := NewPrefixedRegistry("prefix.")
	err := r.Register("foo", NewCounter())
	c := NewCounter()
	Register("bar", c)
	if err != nil {
		t.Fatal(err.Error())
	}

	i := 0
	r.Each(func(name string, m interface{}) {
		i++
		if name != "prefix.foo" {
			t.Fatal(name)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}
}

func TestPrefixedRegistryUnregister(t *testing.T) {
	r := NewPrefixedRegistry("prefix.")

	_ = r.Register("foo", NewCounter())

	i := 0
	r.Each(func(name string, m interface{}) {
		i++
		if name != "prefix.foo" {
			t.Fatal(name)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}

	r.Unregister("foo")

	i = 0
	r.Each(func(name string, m interface{}) {
		i++
	})

	if i != 0 {
		t.Fatal(i)
	}
}

func TestPrefixedRegistryGet(t *testing.T) {
	pr := NewPrefixedRegistry("prefix.")
	name := "foo"
	pr.Register(name, NewCounter())

	fooCounter := pr.Get(name)
	if fooCounter == nil {
		t.Fatal(name)
	}
}

func TestPrefixedChildRegistryGet(t *testing.T) {
	r := NewRegistry()
	pr := NewPrefixedChildRegistry(r, "prefix.")
	name := "foo"
	pr.Register(name, NewCounter())
	fooCounter := pr.Get(name)
	if fooCounter == nil {
		t.Fatal(name)
	}
}

func TestChildPrefixedRegistryRegister(t *testing.T) {
	r := NewPrefixedChildRegistry(DefaultRegistry, "prefix.")
	err := r.Register("foo", NewCounter())
	c := NewCounter()
	Register("bar", c)
	if err != nil {
		t.Fatal(err.Error())
	}

	i := 0
	r.Each(func(name string, m interface{}) {
		i++
		if name != "prefix.foo" {
			t.Fatal(name)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}
}

func TestChildPrefixedRegistryOfChildRegister(t *testing.T) {
	r := NewPrefixedChildRegistry(NewRegistry(), "prefix.")
	r2 := NewPrefixedChildRegistry(r, "prefix2.")
	err := r.Register("foo2", NewCounter())
	if err != nil {
		t.Fatal(err.Error())
	}
	err = r2.Register("baz", NewCounter())
	c := NewCounter()
	Register("bars", c)

	i := 0
	r2.Each(func(name string, m interface{}) {
		i++
		if name != "prefix.prefix2.baz" {
			//t.Fatal(name)
		}
	})
	if i != 1 {
		t.Fatal(i)
	}
}

func TestWalkRegistries(t *testing.T) {
	r := NewPrefixedChildRegistry(NewRegistry(), "prefix.")
	r2 := NewPrefixedChildRegistry(r, "prefix2.")
	err := r.Register("foo2", NewCounter())
	if err != nil {
		t.Fatal(err.Error())
	}
	err = r2.Register("baz", NewCounter())
	c := NewCounter()
	Register("bars", c)

	_, prefix := findPrefix(r2, "")
	if "prefix.prefix2." != prefix {
		t.Fatal(prefix)
	}
}

func TestConcurrentRegistryAccess(t *testing.T) {
	r := NewRegistry()

	counter := NewCounter()

	signalChan := make(chan struct{})

	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(dowork chan struct{}) {
			defer wg.Done()
			iface := r.GetOrRegister("foo", counter)
			retCounter, ok := iface.(Counter)
			if !ok {
				t.Fatal("Expected a Counter type")
			}
			if retCounter != counter {
				t.Fatal("Counter references don't match")
			}
		}(signalChan)
	}

	close(signalChan) // Closing will cause all go routines to execute at the same time
	wg.Wait()         // Wait for all go routines to do their work

	// At the end of the test we should still only have a single "foo" Counter
	i := 0
	r.Each(func(name string, iface interface{}) {
		i++
		if "foo" != name {
			t.Fatal(name)
		}
		if _, ok := iface.(Counter); !ok {
			t.Fatal(iface)
		}
	})
	if 1 != i {
		t.Fatal(i)
	}
	r.Unregister("foo")
	i = 0
	r.Each(func(string, interface{}) { i++ })
	if 0 != i {
		t.Fatal(i)
	}
}

// exercise race detector
func TestRegisterAndRegisteredConcurrency(t *testing.T) {
	r := NewRegistry()
	wg := &sync.WaitGroup{}
	wg.Add(1)
	go func(r Registry, wg *sync.WaitGroup) {
		defer wg.Done()
		r.Each(func(name string, iface interface{}) {
		})
	}(r, wg)
	r.Register("foo", NewCounter())
	wg.Wait()
}
