package statsd

import (
	"strconv"
)

type bufferFullError string

func (e bufferFullError) Error() string { return string(e) }

const errBufferFull = bufferFullError("statsd buffer is full")

type partialWriteError string

func (e partialWriteError) Error() string { return string(e) }

const errPartialWrite = partialWriteError("value partially written")

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
	b.buffer = appendGauge(b.buffer, namespace, globalTags, name, value, tags, rate)
	b.writeSeparator()
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeCount(namespace string, globalTags []string, name string, value int64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.buffer = appendCount(b.buffer, namespace, globalTags, name, value, tags, rate)
	b.writeSeparator()
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeHistogram(namespace string, globalTags []string, name string, value float64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.buffer = appendHistogram(b.buffer, namespace, globalTags, name, value, tags, rate)
	b.writeSeparator()
	return b.validateNewElement(originalBuffer)
}

// writeAggregated serialized as many values as possible in the current buffer and return the position in values where it stopped.
func (b *statsdBuffer) writeAggregated(metricSymbol []byte, namespace string, globalTags []string, name string, values []float64, tags string, tagSize int, precision int) (int, error) {
	if b.elementCount >= b.maxElements {
		return 0, errBufferFull
	}

	originalBuffer := b.buffer
	b.buffer = appendHeader(b.buffer, namespace, name)

	// buffer already full
	if len(b.buffer)+tagSize > b.maxSize {
		b.buffer = originalBuffer
		return 0, errBufferFull
	}

	// We add as many value as possible
	var position int
	for idx, v := range values {
		previousBuffer := b.buffer
		if idx != 0 {
			b.buffer = append(b.buffer, ':')
		}

		b.buffer = strconv.AppendFloat(b.buffer, v, 'f', precision, 64)

		// Should we stop serializing and switch to another buffer
		if len(b.buffer)+tagSize > b.maxSize {
			b.buffer = previousBuffer
			break
		}
		position = idx + 1
	}

	// we could not add a single value
	if position == 0 {
		b.buffer = originalBuffer
		return 0, errBufferFull
	}

	b.buffer = append(b.buffer, '|')
	b.buffer = append(b.buffer, metricSymbol...)
	b.buffer = appendTagsAggregated(b.buffer, globalTags, tags)
	b.writeSeparator()
	b.elementCount++

	if position != len(values) {
		return position, errPartialWrite
	}
	return position, nil

}

func (b *statsdBuffer) writeDistribution(namespace string, globalTags []string, name string, value float64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.buffer = appendDistribution(b.buffer, namespace, globalTags, name, value, tags, rate)
	b.writeSeparator()
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeSet(namespace string, globalTags []string, name string, value string, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.buffer = appendSet(b.buffer, namespace, globalTags, name, value, tags, rate)
	b.writeSeparator()
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeTiming(namespace string, globalTags []string, name string, value float64, tags []string, rate float64) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.buffer = appendTiming(b.buffer, namespace, globalTags, name, value, tags, rate)
	b.writeSeparator()
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeEvent(event Event, globalTags []string) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.buffer = appendEvent(b.buffer, event, globalTags)
	b.writeSeparator()
	return b.validateNewElement(originalBuffer)
}

func (b *statsdBuffer) writeServiceCheck(serviceCheck ServiceCheck, globalTags []string) error {
	if b.elementCount >= b.maxElements {
		return errBufferFull
	}
	originalBuffer := b.buffer
	b.buffer = appendServiceCheck(b.buffer, serviceCheck, globalTags)
	b.writeSeparator()
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
	b.buffer = append(b.buffer, '\n')
}

func (b *statsdBuffer) reset() {
	b.buffer = b.buffer[:0]
	b.elementCount = 0
}

func (b *statsdBuffer) bytes() []byte {
	return b.buffer
}
