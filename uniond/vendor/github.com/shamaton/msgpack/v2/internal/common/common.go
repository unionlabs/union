package common

import "reflect"

// Common is used encoding/decoding
type Common struct {
}

// CheckField returns flag whether should encode/decode or not and field name
func (c *Common) CheckField(field reflect.StructField) (bool, string) {
	// A to Z
	if c.isPublic(field.Name) {
		if tag := field.Tag.Get("msgpack"); tag == "-" {
			return false, ""
		} else if len(tag) > 0 {
			return true, tag
		}
		return true, field.Name
	}
	return false, ""
}

func (c *Common) isPublic(name string) bool {
	return 0x41 <= name[0] && name[0] <= 0x5a
}
