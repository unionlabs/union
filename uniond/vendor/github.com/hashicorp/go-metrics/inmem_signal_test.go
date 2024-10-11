package metrics

import (
	"bytes"
	"os"
	"strings"
	"sync"
	"syscall"
	"testing"
	"time"
)

func TestInmemSignal(t *testing.T) {
	buf := newBuffer()
	inm := NewInmemSink(10*time.Millisecond, 50*time.Millisecond)
	sig := NewInmemSignal(inm, syscall.SIGUSR1, buf)
	defer sig.Stop()

	inm.SetGauge([]string{"foo"}, 42)
	inm.EmitKey([]string{"bar"}, 42)
	inm.IncrCounter([]string{"baz"}, 42)
	inm.AddSample([]string{"wow"}, 42)
	inm.SetGaugeWithLabels([]string{"asdf"}, 42, []Label{{"a", "b"}})
	inm.IncrCounterWithLabels([]string{"qwer"}, 42, []Label{{"a", "b"}})
	inm.AddSampleWithLabels([]string{"zxcv"}, 42, []Label{{"a", "b"}})

	// Wait for period to end
	time.Sleep(15 * time.Millisecond)

	// Send signal!
	syscall.Kill(os.Getpid(), syscall.SIGUSR1)

	// Wait for flush
	time.Sleep(10 * time.Millisecond)

	// Check the output
	out := buf.String()
	if !strings.Contains(out, "[G] 'foo': 42") {
		t.Fatalf("bad: %v", out)
	}
	if !strings.Contains(out, "[P] 'bar': 42") {
		t.Fatalf("bad: %v", out)
	}
	if !strings.Contains(out, "[C] 'baz': Count: 1 Sum: 42") {
		t.Fatalf("bad: %v", out)
	}
	if !strings.Contains(out, "[S] 'wow': Count: 1 Sum: 42") {
		t.Fatalf("bad: %v", out)
	}
	if !strings.Contains(out, "[G] 'asdf.b': 42") {
		t.Fatalf("bad: %v", out)
	}
	if !strings.Contains(out, "[C] 'qwer.b': Count: 1 Sum: 42") {
		t.Fatalf("bad: %v", out)
	}
	if !strings.Contains(out, "[S] 'zxcv.b': Count: 1 Sum: 42") {
		t.Fatalf("bad: %v", out)
	}
}

func newBuffer() *syncBuffer {
	return &syncBuffer{buf: bytes.NewBuffer(nil)}
}

type syncBuffer struct {
	buf  *bytes.Buffer
	lock sync.Mutex
}

func (s *syncBuffer) Write(p []byte) (int, error) {
	s.lock.Lock()
	defer s.lock.Unlock()

	return s.buf.Write(p)
}

func (s *syncBuffer) String() string {
	s.lock.Lock()
	defer s.lock.Unlock()

	return s.buf.String()
}
