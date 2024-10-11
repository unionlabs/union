package v1beta2

import (
	"bytes"

	"github.com/cosmos/gogoproto/jsonpb"
)

// IsAccepted returns true if Code is ACCEPT
func (r ResponseProcessProposal) IsAccepted() bool {
	return r.Status == ResponseProcessProposal_ACCEPT
}

// IsStatusUnknown returns true if Code is UNKNOWN
func (r ResponseProcessProposal) IsStatusUnknown() bool {
	return r.Status == ResponseProcessProposal_UNKNOWN
}

// ---------------------------------------------------------------------------
// override JSON marshaling so we emit defaults (ie. disable omitempty)

var (
	jsonpbMarshaller = jsonpb.Marshaler{
		EnumsAsInts:  true,
		EmitDefaults: true,
	}
	jsonpbUnmarshaller = jsonpb.Unmarshaler{}
)

func (r *EventAttribute) MarshalJSON() ([]byte, error) {
	s, err := jsonpbMarshaller.MarshalToString(r)
	return []byte(s), err
}

func (r *EventAttribute) UnmarshalJSON(b []byte) error {
	reader := bytes.NewBuffer(b)
	return jsonpbUnmarshaller.Unmarshal(reader, r)
}
