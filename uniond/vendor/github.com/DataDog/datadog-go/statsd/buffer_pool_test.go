package statsd

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestBufferPoolSize(t *testing.T) {
	bufferPool := newBufferPool(10, 1024, 20)

	assert.Equal(t, 10, cap(bufferPool.pool))
	assert.Equal(t, 10, len(bufferPool.pool))
}

func TestBufferPoolBufferCreation(t *testing.T) {
	bufferPool := newBufferPool(10, 1024, 20)
	buffer := bufferPool.borrowBuffer()

	assert.Equal(t, 1024, buffer.maxSize)
	assert.Equal(t, 20, buffer.maxElements)
}

func TestBufferPoolEmpty(t *testing.T) {
	bufferPool := newBufferPool(1, 1024, 20)
	bufferPool.borrowBuffer()

	assert.Equal(t, 0, len(bufferPool.pool))
	buffer := bufferPool.borrowBuffer()
	assert.NotNil(t, buffer.bytes())
}

func TestBufferReturn(t *testing.T) {
	bufferPool := newBufferPool(1, 1024, 20)
	buffer := bufferPool.borrowBuffer()
	buffer.writeCount("", nil, "", 1, nil, 1)

	assert.Equal(t, 0, len(bufferPool.pool))
	bufferPool.returnBuffer(buffer)
	assert.Equal(t, 1, len(bufferPool.pool))
	buffer = bufferPool.borrowBuffer()
	assert.Equal(t, 0, len(buffer.bytes()))
}
