package grpc_testing

import (
	"io"
	"sync"
)

// MutexReadWriter is a io.ReadWriter that can be read and worked on from multiple go routines.
type MutexReadWriter struct {
	sync.Mutex
	rw io.ReadWriter
}

// NewMutexReadWriter creates a new thread-safe io.ReadWriter.
func NewMutexReadWriter(rw io.ReadWriter) *MutexReadWriter {
	return &MutexReadWriter{rw: rw}
}

// Write implements the io.Writer interface.
func (m *MutexReadWriter) Write(p []byte) (int, error) {
	m.Lock()
	defer m.Unlock()
	return m.rw.Write(p)
}

// Read implements the io.Reader interface.
func (m *MutexReadWriter) Read(p []byte) (int, error) {
	m.Lock()
	defer m.Unlock()
	return m.rw.Read(p)
}
