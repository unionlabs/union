// THE ANALYZER DOES NOT CATCH RLOCKS WHEN THE MUTEX VARIABLE IS OUTSIDE OF SCOPE LIKE THIS

package testdata

import (
	"sync"
)

var mutex *sync.RWMutex

func RLockFuncs() {
	regularRLock()
	nestedRLock1Level()
	nestedRLock2Levels()
	deferredRLock()
	recursiveRLock(true)
}

func regularRLock() {
	mutex.RLock()
	mutex.RUnlock()
}

func nestedRLock1Level() {
	mutex.RLock()
	regularRLock() // this is a nested RLock, but the analyzer will not pick it up
	mutex.RUnlock()
}

func nestedRLock2Levels() {
	mutex.RLock()
	callRegularRLock()
	mutex.RUnlock()
}

func callRegularRLock() {
	regularRLock()
}

func deferredRLock() {
	mutex.RLock()
	defer mutex.RUnlock()
	regularRLock()
}

func recursiveRLock(run bool) {
	mutex.RLock()
	defer mutex.RUnlock()
	if run {
		recursiveRLock(!run)
	}
}
