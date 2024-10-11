package statsd

import (
	"math/rand"
	"sync"
)

func shouldSample(rate float64, r *rand.Rand, lock *sync.Mutex) bool {
	if rate >= 1 {
		return true
	}
	// sources created by rand.NewSource() (ie. w.random) are not thread safe.
	// TODO: use defer once the lowest Go version we support is 1.14 (defer
	// has an overhead before that).
	lock.Lock()
	if r.Float64() > rate {
		lock.Unlock()
		return false
	}
	lock.Unlock()
	return true

}
