package utils

import (
	"bytes"
	"github.com/stretchr/testify/assert"
	"io"
	"reflect"
	"testing"
)

type Serializable interface {
	io.ReaderFrom
	io.WriterTo
}

type RawSerializable interface {
	WriteRawTo(io.Writer) (int64, error)
}

func SerializationRoundTrip(o Serializable) func(*testing.T) {
	return func(t *testing.T) {
		// serialize it...
		var buf bytes.Buffer
		_, err := o.WriteTo(&buf)
		assert.NoError(t, err)

		// reconstruct the object
		_o := reflect.New(reflect.TypeOf(o).Elem()).Interface().(Serializable)
		_, err = _o.ReadFrom(&buf)
		assert.NoError(t, err)

		// compare
		assert.Equal(t, o, _o)
	}
}

func SerializationRoundTripRaw(o RawSerializable) func(*testing.T) {
	return func(t *testing.T) {
		// serialize it...
		var buf bytes.Buffer
		_, err := o.WriteRawTo(&buf)
		assert.NoError(t, err)

		// reconstruct the object
		_o := reflect.New(reflect.TypeOf(o).Elem()).Interface().(Serializable)
		_, err = _o.ReadFrom(&buf)
		assert.NoError(t, err)

		// compare
		assert.Equal(t, o, _o)
	}
}
