package deadlock

import (
	"testing"
)

func BenchmarkCheckDeadlock(b *testing.B) {
	ch := make(chan struct{})
	close(ch)
	for i := 0; i < b.N; i++ {
		checkDeadlock(nil, nil, 0, ch)
	}
}
