// +build go1.18

package deadlock

import (
	"math/rand"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

func TestTryLockNoDeadlocks(t *testing.T) {
	defer restore()()
	Opts.DeadlockTimeout = time.Millisecond * 5000
	var a RWMutex
	var b Mutex
	var c RWMutex
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for k := 0; k < 5; k++ {
				func() {
					if a.TryLock() {
						defer a.Unlock()
						func() {
							if b.TryLock() {
								defer b.Unlock()
								func() {
									if c.TryRLock() {
										defer c.RUnlock()
										time.Sleep(time.Duration((1000 + rand.Intn(1000))) * time.Millisecond / 200)
									}
								}()
							}
						}()
					}
				}()
			}
		}()
		wg.Add(1)
		go func() {
			defer wg.Done()
			for k := 0; k < 5; k++ {
				func() {
					if a.TryRLock() {
						defer a.RUnlock()
						func() {
							if b.TryLock() {
								defer b.Unlock()
								func() {
									if c.TryLock() {
										defer c.Unlock()
										time.Sleep(time.Duration((1000 + rand.Intn(1000))) * time.Millisecond / 200)
									}
								}()
							}
						}()
					}
				}()
			}
		}()
	}
	wg.Wait()
}

func TestTryLockOrder(t *testing.T) {
	defer restore()()
	Opts.DeadlockTimeout = 0
	var deadlocks uint32
	Opts.OnPotentialDeadlock = func() {
		atomic.AddUint32(&deadlocks, 1)
	}
	var a RWMutex
	var b Mutex
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		if a.TryLock() {
			if b.TryLock() {
				b.Unlock()
			}
			a.Unlock()
		}
	}()
	wg.Wait()
	wg.Add(1)
	go func() {
		defer wg.Done()
		b.Lock()
		a.RLock()
		a.RUnlock()
		b.Unlock()
	}()
	wg.Wait()
	if atomic.LoadUint32(&deadlocks) != 1 {
		t.Fatalf("expected 1 deadlock, detected %d", deadlocks)
	}
}

func TestHardDeadlockTryLock(t *testing.T) {
	defer restore()()
	Opts.DisableLockOrderDetection = true
	Opts.DeadlockTimeout = time.Millisecond * 20
	var deadlocks uint32
	Opts.OnPotentialDeadlock = func() {
		atomic.AddUint32(&deadlocks, 1)
	}
	var mu Mutex
	if !mu.TryLock() {
		t.Fatal("expected TryLock to succeed, it failed")
	}
	ch := make(chan struct{})
	go func() {
		defer close(ch)
		mu.Lock()
		defer mu.Unlock()
	}()
	select {
	case <-ch:
	case <-time.After(time.Millisecond * 100):
	}
	if atomic.LoadUint32(&deadlocks) != 1 {
		t.Fatalf("expected 1 deadlock, detected %d", deadlocks)
	}
	mu.Unlock()
	<-ch
}

func TestMutexTryLock(t *testing.T) {
	defer restore()()
	Opts.DisableLockOrderDetection = true
	var mu Mutex
	if !mu.TryLock() {
		t.Fatal("expected TryLock to succeed, it failed")
	}
	if mu.TryLock() {
		t.Fatal("expected TryLock to fail, it succeeded")
	}
	mu.Unlock()
}

func TestRWMutexTryLock(t *testing.T) {
	defer restore()()
	Opts.DeadlockTimeout = time.Millisecond * 20
	var deadlocks uint32
	Opts.OnPotentialDeadlock = func() {
		atomic.AddUint32(&deadlocks, 1)
	}
	var a RWMutex
	if !a.TryRLock() {
		t.Fatal("expected TryRLock to succeed, it failed")
	}
	go func() {
		// We detect a potential deadlock here.
		a.Lock()
		defer a.Unlock()
	}()
	time.Sleep(time.Millisecond * 100) // We want the Lock call to happen.
	ch := make(chan struct{})
	go func() {
		// We detect a potential deadlock here.
		defer close(ch)
		a.RLock()
		defer a.RUnlock()
	}()
	select {
	case <-ch:
		t.Fatal("expected a timeout")
	case <-time.After(time.Millisecond * 50):
	}
	a.RUnlock()
	if atomic.LoadUint32(&deadlocks) != 2 {
		t.Fatalf("expected 2 deadlocks, detected %d", deadlocks)
	}
	<-ch
}

func TestTryLockDuplicate(t *testing.T) {
	defer restore()()
	Opts.DeadlockTimeout = 0
	var deadlocks uint32
	Opts.OnPotentialDeadlock = func() {
		atomic.AddUint32(&deadlocks, 1)
	}
	var a RWMutex
	var b Mutex
	go func() {
		if !a.TryRLock() {
			t.Fatal("expected TryRLock to succeed, it failed")
		}
		a.Lock()
		a.RUnlock()
		a.Unlock()
	}()
	go func() {
		if !b.TryLock() {
			t.Fatal("expected TryLock to succeed, it failed")
		}
		b.Lock()
		b.Unlock()
		b.Unlock()
	}()
	time.Sleep(time.Second * 1)
	if atomic.LoadUint32(&deadlocks) != 2 {
		t.Fatalf("expected 2 deadlocks, detected %d", deadlocks)
	}
}
