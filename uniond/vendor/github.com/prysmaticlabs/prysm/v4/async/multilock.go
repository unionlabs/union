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
	"runtime"
	"sort"
)

var locks = struct {
	lock chan byte
	list map[string]chan byte
}{
	lock: make(chan byte, 1),
	list: make(map[string]chan byte),
}

type Lock struct {
	keys   []string
	chans  []chan byte
	lock   chan byte
	unlock chan byte
}

func (lk *Lock) Lock() {
	lk.lock <- 1

	// get the channels and attempt to acquire them
	lk.chans = make([]chan byte, 0, len(lk.keys))
	for i := 0; i < len(lk.keys); {
		ch := getChan(lk.keys[i])
		_, ok := <-ch
		if ok {
			lk.chans = append(lk.chans, ch)
			i++
		}
	}

	lk.unlock <- 1
}

// Unlock unlocks this lock. Must be called after Lock.
// Can only be invoked if there is a previous call to Lock.
func (lk *Lock) Unlock() {
	<-lk.unlock

	if lk.chans != nil {
		for _, ch := range lk.chans {
			ch <- 1
		}
		lk.chans = nil
	}
	// Clean unused channels after the unlock.
	Clean()
	<-lk.lock
}

// Yield temporarily unlocks, gives up the cpu time to other goroutine, and attempts to lock again.
func (lk *Lock) Yield() {
	lk.Unlock()
	runtime.Gosched()
	lk.Lock()
}

// NewMultilock creates a new multilock for the specified keys
func NewMultilock(locks ...string) *Lock {
	if len(locks) == 0 {
		return nil
	}

	locks = unique(locks)
	sort.Strings(locks)
	return &Lock{
		keys:   locks,
		lock:   make(chan byte, 1),
		unlock: make(chan byte, 1),
	}
}

// Clean cleans old unused locks. Returns removed keys.
func Clean() []string {
	locks.lock <- 1
	defer func() { <-locks.lock }()

	toDelete := make([]string, 0, len(locks.list))
	for key, ch := range locks.list {
		select {
		case <-ch:
			close(ch)
			toDelete = append(toDelete, key)
		default:
		}
	}

	for _, del := range toDelete {
		delete(locks.list, del)
	}

	return toDelete
}

// Create and get the channel for the specified key.
func getChan(key string) chan byte {
	locks.lock <- 1
	defer func() { <-locks.lock }()

	if locks.list[key] == nil {
		locks.list[key] = make(chan byte, 1)
		locks.list[key] <- 1
	}
	return locks.list[key]
}

// Return a new string with unique elements.
func unique(arr []string) []string {
	if arr == nil || len(arr) <= 1 {
		return arr
	}

	found := map[string]bool{}
	result := make([]string, 0, len(arr))
	for _, v := range arr {
		if !found[v] {
			found[v] = true
			result = append(result, v)
		}
	}
	return result
}
