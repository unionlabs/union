package statsd

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

type mockedWriter struct {
	mock.Mock
}

func (w *mockedWriter) Write(data []byte) (n int, err error) {
	args := w.Called(data)
	return args.Int(0), args.Error(1)
}
func (w *mockedWriter) SetWriteTimeout(d time.Duration) error {
	args := w.Called(d)
	return args.Error(0)
}
func (w *mockedWriter) Close() error {
	args := w.Called()
	return args.Error(0)
}

func TestSender(t *testing.T) {
	writer := new(mockedWriter)
	writer.On("Write", mock.Anything).Return(0, nil)
	writer.On("Close").Return(nil)
	pool := newBufferPool(10, 1024, 1)
	sender := newSender(writer, 10, pool)
	buffer := pool.borrowBuffer()

	sender.send(buffer)

	err := sender.close()
	assert.Nil(t, err)
	writer.AssertCalled(t, "Write", buffer.bytes())
	assert.Equal(t, 10, len(pool.pool))
}
