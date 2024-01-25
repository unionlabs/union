package statsd

type bufferFullError string

func (e bufferFullError) Error() string { return string(e) }

const errBufferFull = bufferFullError("statsd buffer is full")

const metricOverhead = 512

// statsdBuffer is a buffer containing statsd messages
// this struct methods are NOT safe for concurent use
type statsdBuffer struct {
	buffer       []byte
	maxSize      int
	maxElements  int
	elementCount int
}

func newStatsdBuffer(maxSize, maxElements int) *statsdBuffer {
	return &statsdBuffer{
		buffer:      make([]byte, 0, maxSize+metricOverhead), // pre-allocate the needed size + metricOverhead to avoid having Go re-allocate on it's own if an element does not fit
		maxSize:     maxSize,
		maxElements: maxElements,
	}
}

func (b *statsdBuffer) writeGauge(namespace string, globalTags []string, name string, value float64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendGauge(b.buffer, namespace, globalTags, name, value, tags, rate)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeCount(namespace string, globalTags []string, name string, value int64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendCount(b.buffer, namespace, globalTags, name, value, tags, rate)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeHistogram(namespace string, globalTags []string, name string, value float64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendHistogram(b.buffer, namespace, globalTags, name, value, tags, rate)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeDistribution(namespace string, globalTags []string, name string, value float64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendDistribution(b.buffer, namespace, globalTags, name, value, tags, rate)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeSet(namespace string, globalTags []string, name string, value string, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendSet(b.buffer, namespace, globalTags, name, value, tags, rate)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeTiming(namespace string, globalTags []string, name string, value float64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendTiming(b.buffer, namespace, globalTags, name, value, tags, rate)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeEvent(event Event, globalTags []string) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendEvent(b.buffer, event, globalTags)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeServiceCheck(serviceCheck ServiceCheck, globalTags []string) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.writeSeparator()
	b.buffer = appendServiceCheck(b.buffer, serviceCheck, globalTags)
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) validateNewElement(originalBuffer []byte) error {
	if len(b.buffer) > b.maxSize {
		b.buffer = originalBuffer
		return errBufferFull
	}
	b.elementCount++
	return nil
}

func (b *statsdBuffer) writeSeparator() {
	if b.elementCount != 0 {
		b.buffer = appendSeparator(b.buffer)
	}
}

func (b *statsdBuffer) reset() {
	b.buffer = b.buffer[:0]
	b.elementCount = 0
}

func (b *statsdBuffer) bytes() []byte {
	return b.buffer
}
