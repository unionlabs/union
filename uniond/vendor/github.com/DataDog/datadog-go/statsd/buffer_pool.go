package statsd

type bufferPool struct {
	pool              chan *statsdBuffer
	bufferMaxSize     int
	bufferMaxElements int
}

func newBufferPool(poolSize, bufferMaxSize, bufferMaxElements int) *bufferPool {
	p := &bufferPool{
		pool:              make(chan *statsdBuffer, poolSize),
		bufferMaxSize:     bufferMaxSize,
		bufferMaxElements: bufferMaxElements,
	}
	for i := 0; i < poolSize; i++ {
		p.addNewBuffer()
	}
	return p
}

func (p *bufferPool) addNewBuffer() {
	p.pool <- newStatsdBuffer(p.bufferMaxSize, p.bufferMaxElements)
}

func (p *bufferPool) borrowBuffer() *statsdBuffer {
	select {
	case b := <-p.pool:
		return b
	default:
		return newStatsdBuffer(p.bufferMaxSize, p.bufferMaxElements)
	}
}

func (p *bufferPool) returnBuffer(buffer *statsdBuffer) {
	buffer.reset()
	select {
	case p.pool <- buffer:
	default:
	}
}
