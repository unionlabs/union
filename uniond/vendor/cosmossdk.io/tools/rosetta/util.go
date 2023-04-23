package rosetta

import (
	"encoding/json"
	"time"

	crgerrs "cosmossdk.io/tools/rosetta/lib/errors"
)

// timeToMilliseconds converts time to milliseconds timestamp
func timeToMilliseconds(t time.Time) int64 {
	return t.UnixNano() / (int64(time.Millisecond) / int64(time.Nanosecond))
}

// unmarshalMetadata unmarshals the given meta to the target
func unmarshalMetadata(meta map[string]interface{}, target interface{}) error {
	b, err := json.Marshal(meta)
	if err != nil {
		return crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	err = json.Unmarshal(b, target)
	if err != nil {
		return crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	return nil
}

// marshalMetadata marshals the given interface to map[string]interface{}
func marshalMetadata(o interface{}) (meta map[string]interface{}, err error) {
	b, err := json.Marshal(o)
	if err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}
	meta = make(map[string]interface{})
	err = json.Unmarshal(b, &meta)
	if err != nil {
		return nil, err
	}

	return
}
