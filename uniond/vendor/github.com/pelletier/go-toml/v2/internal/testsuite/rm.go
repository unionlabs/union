package testsuite

import (
	"fmt"
	"strconv"
	"time"

	"github.com/pelletier/go-toml/v2"
)

// Remove JSON tags to a data structure as returned by toml-test.
func rmTag(typedJson interface{}) (interface{}, error) {
	// Check if key is in the table m.
	in := func(key string, m map[string]interface{}) bool {
		_, ok := m[key]
		return ok
	}

	// Switch on the data type.
	switch v := typedJson.(type) {

	// Object: this can either be a TOML table or a primitive with tags.
	case map[string]interface{}:
		// This value represents a primitive: remove the tags and return just
		// the primitive value.
		if len(v) == 2 && in("type", v) && in("value", v) {
			ut, err := untag(v)
			if err != nil {
				return ut, fmt.Errorf("tag.Remove: %w", err)
			}
			return ut, nil
		}

		// Table: remove tags on all children.
		m := make(map[string]interface{}, len(v))
		for k, v2 := range v {
			var err error
			m[k], err = rmTag(v2)
			if err != nil {
				return nil, err
			}
		}
		return m, nil

	// Array: remove tags from all itenm.
	case []interface{}:
		a := make([]interface{}, len(v))
		for i := range v {
			var err error
			a[i], err = rmTag(v[i])
			if err != nil {
				return nil, err
			}
		}
		return a, nil
	}

	// The top level must be an object or array.
	return nil, fmt.Errorf("unrecognized JSON format '%T'", typedJson)
}

// Return a primitive: read the "type" and convert the "value" to that.
func untag(typed map[string]interface{}) (interface{}, error) {
	t := typed["type"].(string)
	v := typed["value"].(string)
	switch t {
	case "string":
		return v, nil
	case "integer":
		n, err := strconv.ParseInt(v, 10, 64)
		if err != nil {
			return nil, fmt.Errorf("untag: %w", err)
		}
		return n, nil
	case "float":
		f, err := strconv.ParseFloat(v, 64)
		if err != nil {
			return nil, fmt.Errorf("untag: %w", err)
		}
		return f, nil

		//toml.LocalDate{Year:2020, Month:12, Day:12}
	case "datetime":
		return time.Parse("2006-01-02T15:04:05.999999999Z07:00", v)
	case "datetime-local":
		var t toml.LocalDateTime
		err := t.UnmarshalText([]byte(v))
		if err != nil {
			return nil, fmt.Errorf("untag: %w", err)
		}
		return t, nil
	case "date-local":
		var t toml.LocalDate
		err := t.UnmarshalText([]byte(v))
		if err != nil {
			return nil, fmt.Errorf("untag: %w", err)
		}
		return t, nil
	case "time-local":
		var t toml.LocalTime
		err := t.UnmarshalText([]byte(v))
		if err != nil {
			return nil, fmt.Errorf("untag: %w", err)
		}
		return t, nil
	case "bool":
		switch v {
		case "true":
			return true, nil
		case "false":
			return false, nil
		}
		return nil, fmt.Errorf("untag: could not parse %q as a boolean", v)
	}

	return nil, fmt.Errorf("untag: unrecognized tag type %q", t)
}

func parseTime(v, format string, local bool) (t time.Time, err error) {
	if local {
		t, err = time.ParseInLocation(format, v, time.Local)
	} else {
		t, err = time.Parse(format, v)
	}
	if err != nil {
		return time.Time{}, fmt.Errorf("Could not parse %q as a datetime: %w", v, err)
	}
	return t, nil
}
