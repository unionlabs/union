package server

import (
	"log"
	"reflect"
	"strings"

	"github.com/grpc-ecosystem/grpc-gateway/internal/casing"
	"google.golang.org/genproto/protobuf/field_mask"
)

func applyFieldMask(patchee, patcher interface{}, mask *field_mask.FieldMask) {
	if mask == nil {
		return
	}

	for _, path := range mask.GetPaths() {
		val := getField(patcher, path)
		if val.IsValid() {
			setValue(patchee, val, path)
		}
	}
}

func getField(obj interface{}, path string) (val reflect.Value) {
	// this func is lazy -- if anything bad happens just return nil
	defer func() {
		if r := recover(); r != nil {
			log.Printf("failed to get field:\npath: %q\nobj: %#v\nerr: %v", path, obj, r)
			val = reflect.Value{}
		}
	}()

	v := reflect.ValueOf(obj)
	if len(path) == 0 {
		return v
	}

	for _, s := range strings.Split(path, ".") {
		if v.Kind() == reflect.Ptr {
			v = reflect.Indirect(v)
		}
		v = v.FieldByName(casing.Camel(s))
	}

	return v
}

func setValue(obj interface{}, newValue reflect.Value, path string) {
	defer func() {
		if r := recover(); r != nil {
			log.Printf("failed to set value:\nnewValue: %#v\npath: %q\nobj: %#v\nerr: %v", newValue, path, obj, r)
		}
	}()
	getField(obj, path).Set(newValue)
}
