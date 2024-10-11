/*
Copyright 2017 Albert Tedja
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

	http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
package async

import (
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestUnique(t *testing.T) {
	var arr []string
	a := assert.New(t)

	arr = []string{"a", "b", "c"}
	a.Equal(arr, unique(arr))

	arr = []string{"a", "a", "a"}
	a.Equal([]string{"a"}, unique(arr))

	arr = []string{"a", "a", "b"}
	a.Equal([]string{"a", "b"}, unique(arr))

	arr = []string{"a", "b", "a"}
	a.Equal([]string{"a", "b"}, unique(arr))

	arr = []string{"a", "b", "c", "b", "d"}
	a.Equal([]string{"a", "b", "c", "d"}, unique(arr))
}

func TestGetChan(t *testing.T) {
	ch1 := getChan("a")
	ch2 := getChan("aa")
	ch3 := getChan("a")

	a := assert.New(t)
	a.NotEqual(ch1, ch2)
	a.Equal(ch1, ch3)
}

func TestLockUnlock(_ *testing.T) {
	var wg sync.WaitGroup

	wg.Add(5)

	go func() {
		lock := NewMultilock("dog", "cat", "owl")
		lock.Lock()
		defer lock.Unlock()

		<-time.After(100 * time.Millisecond)
		wg.Done()
	}()

	go func() {
		lock := NewMultilock("cat", "dog", "bird")
		lock.Lock()
		defer lock.Unlock()

		<-time.After(100 * time.Millisecond)
		wg.Done()
	}()

	go func() {
		lock := NewMultilock("cat", "bird", "owl")
		lock.Lock()
		defer lock.Unlock()

		<-time.After(100 * time.Millisecond)
		wg.Done()
	}()

	go func() {
		lock := NewMultilock("bird", "owl", "snake")
		lock.Lock()
		defer lock.Unlock()

		<-time.After(100 * time.Millisecond)
		wg.Done()
	}()

	go func() {
		lock := NewMultilock("owl", "snake")
		lock.Lock()
		defer lock.Unlock()

		<-time.After(1 * time.Second)
		wg.Done()
	}()

	wg.Wait()
}

func TestLockUnlock_CleansUnused(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		lock := NewMultilock("dog", "cat", "owl")
		lock.Lock()
		assert.Equal(t, 3, len(locks.list))
		lock.Unlock()

		wg.Done()
	}()
	wg.Wait()
	// We expect that unlocking completely cleared the locks list
	// given all 3 lock keys were unused at time of unlock.
	assert.Equal(t, 0, len(locks.list))
}

func TestLockUnlock_DoesNotCleanIfHeldElsewhere(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(2)
	go func() {
		lock := NewMultilock("cat")
		lock.Lock()
		// We take 200 milliseconds to release the lock on "cat"
		<-time.After(200 * time.Millisecond)
		lock.Unlock()
		// Assert that at the end of this goroutine, all locks are cleared.
		assert.Equal(t, 0, len(locks.list))
		wg.Done()
	}()
	go func() {
		lock := NewMultilock("dog", "cat", "owl")
		lock.Lock()
		// We release the locks after 100 milliseconds, and check that "cat" is not
		// cleared as a lock for it is still held by the previous goroutine.
		<-time.After(100 * time.Millisecond)
		lock.Unlock()
		assert.Equal(t, 1, len(locks.list))
		_, ok := locks.list["cat"]
		assert.Equal(t, true, ok)
		wg.Done()
	}()
	wg.Wait()
	// We expect that at the end of this test, all locks are cleared.
	assert.Equal(t, 0, len(locks.list))
}

func TestYield(t *testing.T) {
	var wg sync.WaitGroup

	wg.Add(2)
	var resources = map[string]int{}

	go func() {
		lock := NewMultilock("A", "C")
		lock.Lock()
		defer lock.Unlock()

		for resources["ac"] == 0 {
			lock.Yield()
		}
		resources["dc"] = 10

		wg.Done()
	}()

	go func() {
		lock := NewMultilock("D", "C")
		lock.Lock()
		defer lock.Unlock()

		resources["ac"] = 5
		for resources["dc"] == 0 {
			lock.Yield()
		}

		wg.Done()
	}()

	wg.Wait()

	assert.Equal(t, 5, resources["ac"])
	assert.Equal(t, 10, resources["dc"])
}

func TestClean(t *testing.T) {
	var wg sync.WaitGroup

	wg.Add(3)

	// some goroutine that holds multiple locks
	go1done := make(chan bool, 1)
	go func() {
	Loop:
		for {
			select {
			case <-go1done:
				break Loop
			default:
				lock := NewMultilock("A", "B", "C", "E", "Z")
				lock.Lock()
				<-time.After(30 * time.Millisecond)
				lock.Unlock()
			}
		}
		wg.Done()
	}()

	// another goroutine
	go2done := make(chan bool, 1)
	go func() {
	Loop:
		for {
			select {
			case <-go2done:
				break Loop
			default:
				lock := NewMultilock("B", "C", "K", "L", "Z")
				lock.Lock()
				<-time.After(200 * time.Millisecond)
				lock.Unlock()
			}
		}
		wg.Done()
	}()

	// this one cleans up the locks every 100 ms
	done := make(chan bool, 1)
	go func() {
		c := time.Tick(100 * time.Millisecond)
	Loop:
		for {
			select {
			case <-done:
				break Loop
			case <-c:
				Clean()
			}
		}
		wg.Done()
	}()

	<-time.After(2 * time.Second)
	go1done <- true
	go2done <- true
	<-time.After(1 * time.Second)
	done <- true
	wg.Wait()
	assert.Equal(t, []string{}, Clean())
}

func TestBankAccountProblem(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(3)

	joe := 50.0
	susan := 100.0

	// withdraw $80 from joe, only if balance is sufficient
	go func() {
		lock := NewMultilock("joe")
		lock.Lock()
		defer lock.Unlock()

		for joe < 80.0 {
			lock.Yield()
		}
		joe -= 80.0

		wg.Done()
	}()

	// transfer $200 from susan to joe, only if balance is sufficient
	go func() {
		lock := NewMultilock("joe", "susan")
		lock.Lock()
		defer lock.Unlock()

		for susan < 200.0 {
			lock.Yield()
		}

		susan -= 200.0
		joe += 200.0

		wg.Done()
	}()

	// susan deposit $300 to cover balance
	go func() {
		lock := NewMultilock("susan")
		lock.Lock()
		defer lock.Unlock()

		susan += 300.0

		wg.Done()
	}()

	wg.Wait()
	assert.Equal(t, 170.0, joe)
	assert.Equal(t, 200.0, susan)
}

func TestSyncCondCompatibility(t *testing.T) {
	var wg sync.WaitGroup
	wg.Add(2)
	cond := sync.NewCond(NewMultilock("A", "C"))
	var testValues = [3]string{"foo", "bar", "fizz!"}
	sharedRsc := testValues[0]

	go func() {
		cond.L.Lock()
		for sharedRsc == testValues[0] {
			cond.Wait()
		}
		sharedRsc = testValues[2]
		cond.Broadcast()
		cond.L.Unlock()
		wg.Done()
	}()

	go func() {
		cond.L.Lock()
		sharedRsc = testValues[1]
		cond.Broadcast()
		for sharedRsc == testValues[1] {
			cond.Wait()
		}
		cond.L.Unlock()
		wg.Done()
	}()

	wg.Wait()
	assert.Equal(t, testValues[2], sharedRsc)
}
