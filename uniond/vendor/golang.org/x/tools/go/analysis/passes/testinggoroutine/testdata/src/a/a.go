// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package a

import (
	"log"
	"sync"
	"testing"
)

func TestBadFatalf(t *testing.T) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < 2; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			t.Fatalf("TestFailed: id = %v\n", id) // want "call to .+T.+Fatalf from a non-test goroutine"
		}(i)
	}
}

func TestOKErrorf(t *testing.T) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < 2; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			t.Errorf("TestFailed: id = %v\n", id)
		}(i)
	}
}

func TestBadFatal(t *testing.T) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < 2; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			t.Fatal("TestFailed") // want "call to .+T.+Fatal from a non-test goroutine"
		}(i)
	}
}

func f(t *testing.T, _ string) {
	t.Fatal("TestFailed")
}

func g() {}

func TestBadFatalIssue47470(t *testing.T) {
	go f(t, "failed test 1") // want "call to .+T.+Fatal from a non-test goroutine"

	g := func(t *testing.T, _ string) {
		t.Fatal("TestFailed")
	}
	go g(t, "failed test 2") // want "call to .+T.+Fatal from a non-test goroutine"
}

func BenchmarkBadFatalf(b *testing.B) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < b.N; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			b.Fatalf("TestFailed: id = %v\n", id) // want "call to .+B.+Fatalf from a non-test goroutine"
		}(i)
	}
}

func BenchmarkBadFatal(b *testing.B) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < b.N; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			b.Fatal("TestFailed") // want "call to .+B.+Fatal from a non-test goroutine"
		}(i)
	}
}

func BenchmarkOKErrorf(b *testing.B) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < b.N; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			b.Errorf("TestFailed: %d", i)
		}(i)
	}
}

func BenchmarkBadFatalGoGo(b *testing.B) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < b.N; i++ {
		wg.Add(1)
		go func(id int) {
			go func() {
				defer wg.Done()
				b.Fatal("TestFailed") // want "call to .+B.+Fatal from a non-test goroutine"
			}()
		}(i)
	}

	if false {
		defer b.Fatal("here")
	}

	if true {
		go func() {
			b.Fatal("in here") // want "call to .+B.+Fatal from a non-test goroutine"
		}()
	}

	func() {
		func() {
			func() {
				func() {
					go func() {
						b.Fatal("Here") // want "call to .+B.+Fatal from a non-test goroutine"
					}()
				}()
			}()
		}()
	}()

	_ = 10 * 10
	_ = func() bool {
		go b.Fatal("Failed") // want "call to .+B.+Fatal from a non-test goroutine"
		return true
	}

	defer func() {
		go b.Fatal("Here") // want "call to .+B.+Fatal from a non-test goroutine"
	}()
}

func BenchmarkBadSkip(b *testing.B) {
	for i := 0; i < b.N; i++ {
		if i == 100 {
			go b.Skip("Skipping") // want "call to .+B.+Skip from a non-test goroutine"
		}
		if i == 22 {
			go func() {
				go func() {
					b.Skip("Skipping now") // want "call to .+B.+Skip from a non-test goroutine"
				}()
			}()
		}
	}
}

func TestBadSkip(t *testing.T) {
	for i := 0; i < 1000; i++ {
		if i == 100 {
			go t.Skip("Skipping") // want "call to .+T.+Skip from a non-test goroutine"
		}
		if i == 22 {
			go func() {
				go func() {
					t.Skip("Skipping now") // want "call to .+T.+Skip from a non-test goroutine"
				}()
			}()
		}
	}
}

func BenchmarkBadFailNow(b *testing.B) {
	for i := 0; i < b.N; i++ {
		if i == 100 {
			go b.FailNow() // want "call to .+B.+FailNow from a non-test goroutine"
		}
		if i == 22 {
			go func() {
				go func() {
					b.FailNow() // want "call to .+B.+FailNow from a non-test goroutine"
				}()
			}()
		}
	}
}

func TestBadFailNow(t *testing.T) {
	for i := 0; i < 1000; i++ {
		if i == 100 {
			go t.FailNow() // want "call to .+T.+FailNow from a non-test goroutine"
		}
		if i == 22 {
			go func() {
				go func() {
					t.FailNow() // want "call to .+T.+FailNow from a non-test goroutine"
				}()
			}()
		}
	}
}

func TestBadWithLoopCond(ty *testing.T) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(id int) {
			defer ty.Fatalf("Why") // want "call to .+T.+Fatalf from a non-test goroutine"
			go func() {
				for j := 0; j < 2; ty.FailNow() { // want "call to .+T.+FailNow from"
					j++
					ty.Errorf("Done here")
				}
			}()
		}(i)
	}
}

type customType int

func (ct *customType) Fatalf(fmtSpec string, args ...interface{}) {
	if fmtSpec == "" {
		panic("empty format specifier")
	}
}

func (ct *customType) FailNow() {}
func (ct *customType) Skip()    {}

func TestWithLogFatalf(t *testing.T) {
	var wg sync.WaitGroup
	defer wg.Wait()

	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(id int) {
			go func() {
				for j := 0; j < 2; j++ {
					log.Fatal("Done here")
				}
			}()
		}(i)
	}
}

func TestWithCustomType(t *testing.T) {
	var wg sync.WaitGroup
	defer wg.Wait()

	ct := new(customType)
	defer ct.FailNow()
	defer ct.Skip()

	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(id int) {
			go func() {
				for j := 0; j < 2; j++ {
					ct.Fatalf("Done here: %d", i)
				}
			}()
		}(i)
	}
}

func helpTB(tb testing.TB) {
	tb.FailNow()
}

func TestTB(t *testing.T) {
	go helpTB(t) // want "call to .+TB.+FailNow from a non-test goroutine"
}

func TestIssue48124(t *testing.T) {
	go helper(t) // want "call to .+T.+Skip from a non-test goroutine"
}

func TestEachCall(t *testing.T) {
	go helper(t) // want "call to .+T.+Skip from a non-test goroutine"
	go helper(t) // want "call to .+T.+Skip from a non-test goroutine"
}

func TestWithSubtest(t *testing.T) {
	t.Run("name", func(t2 *testing.T) {
		t.FailNow() // want "call to .+T.+FailNow on t defined outside of the subtest"
		t2.Fatal()
	})

	f := func(t3 *testing.T) {
		t.FailNow()
		t3.Fatal()
	}
	t.Run("name", f) // want "call to .+T.+FailNow on t defined outside of the subtest"

	g := func(t4 *testing.T) {
		t.FailNow()
		t4.Fatal()
	}
	g(t)

	t.Run("name", helper)

	go t.Run("name", func(t2 *testing.T) {
		t.FailNow() // want "call to .+T.+FailNow on t defined outside of the subtest"
		t2.Fatal()
	})
}

func TestMultipleVariables(t *testing.T) {
	{ // short decl
		f, g := func(t1 *testing.T) {
			t1.Fatal()
		}, func(t2 *testing.T) {
			t2.Error()
		}

		go f(t) // want "call to .+T.+Fatal from a non-test goroutine"
		go g(t)

		t.Run("name", f)
		t.Run("name", g)
	}

	{ // var decl
		var f, g = func(t1 *testing.T) {
			t1.Fatal()
		}, func(t2 *testing.T) {
			t2.Error()
		}

		go f(t) // want "call to .+T.+Fatal from a non-test goroutine"
		go g(t)

		t.Run("name", f)
		t.Run("name", g)
	}
}

func BadIgnoresMultipleAssignments(t *testing.T) {
	{
		f := func(t1 *testing.T) {
			t1.Fatal()
		}
		go f(t) // want "call to .+T.+Fatal from a non-test goroutine"

		f = func(t2 *testing.T) {
			t2.Error()
		}
		go f(t) // want "call to .+T.+Fatal from a non-test goroutine"
	}
	{
		f := func(t1 *testing.T) {
			t1.Error()
		}
		go f(t)

		f = func(t2 *testing.T) {
			t2.FailNow()
		}
		go f(t) // false negative
	}
}

func TestGoDoesNotDescendIntoSubtest(t *testing.T) {
	f := func(t2 *testing.T) {
		g := func(t3 *testing.T) {
			t3.Fatal() // fine
		}
		t2.Run("name", g)
		t2.FailNow() // bad
	}
	go f(t) // want "call to .+T.+FailNow from a non-test goroutine"
}

func TestFreeVariableAssignedWithinEnclosing(t *testing.T) {
	f := func(t2 *testing.T) {
		inner := t
		inner.FailNow()
	}

	go f(nil) // want "call to .+T.+FailNow from a non-test goroutine"

	t.Run("name", func(t3 *testing.T) {
		go f(nil) // want "call to .+T.+FailNow from a non-test goroutine"
	})

	// Without pointer analysis we cannot tell if inner is t or t2.
	// So we accept a false negatives on the following examples.
	t.Run("name", f)

	go func(_ *testing.T) {
		t.Run("name", f)
	}(nil)

	go t.Run("name", f)
}

func TestWithUnusedSelection(t *testing.T) {
	go func() {
		_ = t.FailNow
	}()
	t.Run("name", func(t2 *testing.T) {
		_ = t.FailNow
	})
}

func TestMethodExprsAreIgnored(t *testing.T) {
	go func() {
		(*testing.T).FailNow(t)
	}()
}

func TestRecursive(t *testing.T) {
	t.SkipNow()

	go TestRecursive(t) // want "call to .+T.+SkipNow from a non-test goroutine"

	t.Run("name", TestRecursive)
}

func TestMethodSelection(t *testing.T) {
	var h helperType

	go h.help(t) // want "call to .+T.+SkipNow from a non-test goroutine"
	t.Run("name", h.help)
}

type helperType struct{}

func (h *helperType) help(t *testing.T) { t.SkipNow() }

func TestIssue63799a(t *testing.T) {
	done := make(chan struct{})
	go func() {
		defer close(done)
		t.Run("", func(t *testing.T) {
			t.Fatal() // No warning. This is in a subtest.
		})
	}()
	<-done
}

func TestIssue63799b(t *testing.T) {
	// Simplified from go.dev/cl/538698

	// nondet is some unspecified boolean placeholder.
	var nondet func() bool

	t.Run("nohup", func(t *testing.T) {
		if nondet() {
			t.Skip("ignored")
		}

		go t.Run("nohup-i", func(t *testing.T) {
			t.Parallel()
			if nondet() {
				if nondet() {
					t.Skip("go.dev/cl/538698 wanted to have skip here")
				}

				t.Error("ignored")
			} else {
				t.Log("ignored")
			}
		})
	})
}

func TestIssue63849(t *testing.T) {
	go func() {
		helper(t) // False negative. We do not do an actual interprodecural reachability analysis.
	}()
	go helper(t) // want "call to .+T.+Skip from a non-test goroutine"
}
