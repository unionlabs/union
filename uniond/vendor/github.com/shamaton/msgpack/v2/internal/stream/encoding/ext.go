package encoding

import (
	"reflect"

	"github.com/shamaton/msgpack/v2/ext"
	"github.com/shamaton/msgpack/v2/time"
)

var extCoderMap = map[reflect.Type]ext.StreamEncoder{time.StreamEncoder.Type(): time.StreamEncoder}
var extCoders = []ext.StreamEncoder{time.StreamEncoder}

// AddExtEncoder adds encoders for extension types.
func AddExtEncoder(f ext.StreamEncoder) {
	// ignore time
	if f.Type() == time.Encoder.Type() {
		return
	}

	_, ok := extCoderMap[f.Type()]
	if !ok {
		extCoderMap[f.Type()] = f
		updateExtCoders()
	}
}

// RemoveExtEncoder removes encoders for extension types.
func RemoveExtEncoder(f ext.StreamEncoder) {
	// ignore time
	if f.Type() == time.Encoder.Type() {
		return
	}

	_, ok := extCoderMap[f.Type()]
	if ok {
		delete(extCoderMap, f.Type())
		updateExtCoders()
	}
}

func updateExtCoders() {
	extCoders = make([]ext.StreamEncoder, len(extCoderMap))
	i := 0
	for k := range extCoderMap {
		extCoders[i] = extCoderMap[k]
		i++
	}
}
