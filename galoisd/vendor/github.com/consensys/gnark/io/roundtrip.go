package io

import (
	"bytes"
	"errors"
	"io"
	"reflect"
)

// RoundTripCheck is a helper to check that a serialization round trip is correct.
// It writes the object to a buffer, then reads it back and checks that the reconstructed object is equal to the original.
// It supports both io.ReaderFrom and UnsafeReaderFrom interfaces (to object)
// It also supports both io.WriterTo and WriterRawTo interfaces (from object)
func RoundTripCheck(from any, to func() any) error {
	var buf bytes.Buffer

	reconstruct := func(written int64) error {
		// if builder implements io.ReaderFrom
		if r, ok := to().(io.ReaderFrom); ok {
			read, err := r.ReadFrom(bytes.NewReader(buf.Bytes()))
			if err != nil {
				return err
			}
			if !reflect.DeepEqual(from, r) {
				return errors.New("reconstructed object don't match original (ReadFrom)")
			}
			if written != read {
				return errors.New("bytes written / read don't match")
			}
		}

		// if builder implements gnarkio.UnsafeReaderFrom
		if r, ok := to().(UnsafeReaderFrom); ok {
			read, err := r.UnsafeReadFrom(bytes.NewReader(buf.Bytes()))
			if err != nil {
				return err
			}
			if !reflect.DeepEqual(from, r) {
				return errors.New("reconstructed object don't match original (UnsafeReadFrom)")
			}
			if written != read {
				return errors.New("bytes written / read don't match")
			}
		}
		return nil
	}

	// if from implements io.WriterTo
	if w, ok := from.(io.WriterTo); ok {
		written, err := w.WriteTo(&buf)
		if err != nil {
			return err
		}

		if err := reconstruct(written); err != nil {
			return err
		}
	}

	buf.Reset()

	// if from implements gnarkio.WriterRawTo
	if w, ok := from.(WriterRawTo); ok {
		written, err := w.WriteRawTo(&buf)
		if err != nil {
			return err
		}

		if err := reconstruct(written); err != nil {
			return err
		}
	}

	return nil
}

func DumpRoundTripCheck(from any, to func() any) error {
	var buf bytes.Buffer

	if err := from.(BinaryDumper).WriteDump(&buf); err != nil {
		return err
	}

	r := to().(BinaryDumper)
	if err := r.ReadDump(bytes.NewReader(buf.Bytes())); err != nil {
		return err
	}
	if !reflect.DeepEqual(from, r) {
		return errors.New("reconstructed object don't match original (ReadDump)")
	}
	return nil
}
