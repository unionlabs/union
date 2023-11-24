// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package remote

import (
	"bytes"
	"context"
	"io"
	"os"
	"strings"
	"sync"
)

// NewInMem returns an in-memory implementation of the remote.Storage
// interface (for testing).
func NewInMem() Storage {
	store := &inMemStore{}
	store.mu.objects = make(map[string]*inMemObj)
	return store
}

// inMemStore is an in-memory implementation of the remote.Storage interface
// (for testing).
type inMemStore struct {
	mu struct {
		sync.Mutex
		objects map[string]*inMemObj
	}
}

var _ Storage = (*inMemStore)(nil)

type inMemObj struct {
	name string
	data []byte
}

func (s *inMemStore) Close() error {
	*s = inMemStore{}
	return nil
}

func (s *inMemStore) ReadObject(
	ctx context.Context, objName string,
) (_ ObjectReader, objSize int64, _ error) {
	obj, err := s.getObj(objName)
	if err != nil {
		return nil, 0, err
	}
	return &inMemReader{data: obj.data}, int64(len(obj.data)), nil
}

type inMemReader struct {
	data []byte
}

var _ ObjectReader = (*inMemReader)(nil)

func (r *inMemReader) ReadAt(ctx context.Context, p []byte, offset int64) error {
	if offset+int64(len(p)) > int64(len(r.data)) {
		return io.EOF
	}
	copy(p, r.data[offset:])
	return nil
}

func (r *inMemReader) Close() error {
	r.data = nil
	return nil
}

func (s *inMemStore) CreateObject(objName string) (io.WriteCloser, error) {
	return &inMemWriter{
		store: s,
		name:  objName,
	}, nil
}

type inMemWriter struct {
	store *inMemStore
	name  string
	buf   bytes.Buffer
}

var _ io.WriteCloser = (*inMemWriter)(nil)

func (o *inMemWriter) Write(p []byte) (n int, err error) {
	if o.store == nil {
		panic("Write after Close")
	}
	return o.buf.Write(p)
}

func (o *inMemWriter) Close() error {
	if o.store != nil {
		o.store.addObj(&inMemObj{
			name: o.name,
			data: o.buf.Bytes(),
		})
		o.store = nil
	}
	return nil
}

func (s *inMemStore) List(prefix, delimiter string) ([]string, error) {
	if delimiter != "" {
		panic("delimiter unimplemented")
	}

	s.mu.Lock()
	defer s.mu.Unlock()
	res := make([]string, 0, len(s.mu.objects))
	for name := range s.mu.objects {
		if strings.HasPrefix(name, prefix) {
			res = append(res, name)
		}
	}
	return res, nil
}

func (s *inMemStore) Delete(objName string) error {
	s.rmObj(objName)
	return nil
}

// Size returns the length of the named object in bytesWritten.
func (s *inMemStore) Size(objName string) (int64, error) {
	obj, err := s.getObj(objName)
	if err != nil {
		return 0, err
	}
	return int64(len(obj.data)), nil
}

func (s *inMemStore) IsNotExistError(err error) bool {
	return err == os.ErrNotExist
}

func (s *inMemStore) getObj(name string) (*inMemObj, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	obj, ok := s.mu.objects[name]
	if !ok {
		return nil, os.ErrNotExist
	}
	return obj, nil
}

func (s *inMemStore) addObj(o *inMemObj) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.mu.objects[o.name] = o
}

func (s *inMemStore) rmObj(name string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	delete(s.mu.objects, name)
}
