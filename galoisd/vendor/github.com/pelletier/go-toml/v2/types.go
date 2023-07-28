package toml

import (
	"encoding"
	"reflect"
	"time"
)

var timeType = reflect.TypeOf(time.Time{})
var textMarshalerType = reflect.TypeOf(new(encoding.TextMarshaler)).Elem()
var textUnmarshalerType = reflect.TypeOf(new(encoding.TextUnmarshaler)).Elem()
var mapStringInterfaceType = reflect.TypeOf(map[string]interface{}{})
var sliceInterfaceType = reflect.TypeOf([]interface{}{})
var stringType = reflect.TypeOf("")
