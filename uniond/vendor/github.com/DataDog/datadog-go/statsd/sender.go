package statsd

import (
	"sync/atomic"
	"time"
)

// A statsdWriter offers a standard interface regardless of the underlying
// protocol. For now UDS and UPD writers are available.
// Attention: the underlying buffer of `data` is reused after a `statsdWriter.Write` call.
// `statsdWriter.Write` must be synchronous.
type statsdWriter interface {
	Write(data []byte) (n int, err error)
	SetWriteTimeout(time.Duration) error
	Close() error
}

// SenderMetrics contains metrics about the health of the sender
type SenderMetrics struct {
	TotalSentBytes                uint64
	TotalSentPayloads             uint64
	TotalDroppedPayloads          uint64
	TotalDroppedBytes             uint64
	TotalDroppedPayloadsQueueFull uint64
	TotalDroppedBytesQueueFull    uint64
	TotalDroppedPayloadsWriter    uint64
	TotalDroppedBytesWriter       uint64
}

type sender struct {
	transport   statsdWriter
	pool        *bufferPool
	queue       chan *statsdBuffer
	metrics     *SenderMetrics
	stop        chan struct{}
	flushSignal chan struct{}
}

func newSender(transport statsdWriter, queueSize int, pool *bufferPool) *sender {
	sender := &sender{
		transport:   transport,
		pool:        pool,
		queue:       make(chan *statsdBuffer, queueSize),
		metrics:     &SenderMetrics{},
		stop:        make(chan struct{}),
		flushSignal: make(chan struct{}),
	}

	go sender.sendLoop()
	return sender
}

func (s *sender) send(buffer *statsdBuffer) {
	select {
	case s.queue <- buffer:
	default:
		atomic.AddUint64(&s.metrics.TotalDroppedPayloads, 1)
		atomic.AddUint64(&s.metrics.TotalDroppedBytes, uint64(len(buffer.bytes())))
		atomic.AddUint64(&s.metrics.TotalDroppedPayloadsQueueFull, 1)
		atomic.AddUint64(&s.metrics.TotalDroppedBytesQueueFull, uint64(len(buffer.bytes())))
		s.pool.returnBuffer(buffer)
	}
}

func (s *sender) write(buffer *statsdBuffer) {
	_, err := s.transport.Write(buffer.bytes())
	if err != nil {
		atomic.AddUint64(&s.metrics.TotalDroppedPayloads, 1)
		atomic.AddUint64(&s.metrics.TotalDroppedBytes, uint64(len(buffer.bytes())))
		atomic.AddUint64(&s.metrics.TotalDroppedPayloadsWriter, 1)
		atomic.AddUint64(&s.metrics.TotalDroppedBytesWriter, uint64(len(buffer.bytes())))
	} else {
		atomic.AddUint64(&s.metrics.TotalSentPayloads, 1)
		atomic.AddUint64(&s.metrics.TotalSentBytes, uint64(len(buffer.bytes())))
	}
	s.pool.returnBuffer(buffer)
}

func (s *sender) flushTelemetryMetrics() SenderMetrics {
	return SenderMetrics{
		TotalSentBytes:                atomic.SwapUint64(&s.metrics.TotalSentBytes, 0),
		TotalSentPayloads:             atomic.SwapUint64(&s.metrics.TotalSentPayloads, 0),
		TotalDroppedPayloads:          atomic.SwapUint64(&s.metrics.TotalDroppedPayloads, 0),
		TotalDroppedBytes:             atomic.SwapUint64(&s.metrics.TotalDroppedBytes, 0),
		TotalDroppedPayloadsQueueFull: atomic.SwapUint64(&s.metrics.TotalDroppedPayloadsQueueFull, 0),
		TotalDroppedBytesQueueFull:    atomic.SwapUint64(&s.metrics.TotalDroppedBytesQueueFull, 0),
		TotalDroppedPayloadsWriter:    atomic.SwapUint64(&s.metrics.TotalDroppedPayloadsWriter, 0),
		TotalDroppedBytesWriter:       atomic.SwapUint64(&s.metrics.TotalDroppedBytesWriter, 0),
	}
}

func (s *sender) sendLoop() {
	defer close(s.stop)
	for {
		select {
		case buffer := <-s.queue:
			s.write(buffer)
		case <-s.stop:
			return
		case <-s.flushSignal:
			// At that point we know that the workers are paused (the statsd client
			// will pause them before calling sender.flush()).
			// So we can fully flush the input queue
			s.flushInputQueue()
			s.flushSignal <- struct{}{}
		}
	}
}

func (s *sender) flushInputQueue() {
	for {
		select {
		case buffer := <-s.queue:
			s.write(buffer)
		default:
			return
		}
	}
}
func (s *sender) flush() {
	s.flushSignal <- struct{}{}
	<-s.flushSignal
}

func (s *sender) close() error {
	s.stop <- struct{}{}
	<-s.stop
	s.flushInputQueue()
	return s.transport.Close()
}
