package v1

import (
	"bytes"

	"github.com/cosmos/gogoproto/jsonpb"
)

const (
	CodeTypeOK uint32 = 0
)

// IsOK returns true if Code is OK.
func (r CheckTxResponse) IsOK() bool {
	return r.Code == CodeTypeOK
}

// IsErr returns true if Code is something other than OK.
func (r CheckTxResponse) IsErr() bool {
	return r.Code != CodeTypeOK
}

// IsOK returns true if Code is OK.
func (r QueryResponse) IsOK() bool {
	return r.Code == CodeTypeOK
}

// IsErr returns true if Code is something other than OK.
func (r QueryResponse) IsErr() bool {
	return r.Code != CodeTypeOK
}

// IsAccepted returns true if Code is ACCEPT
func (r ProcessProposalResponse) IsAccepted() bool {
	return r.Status == PROCESS_PROPOSAL_STATUS_ACCEPT
}

// IsStatusUnknown returns true if Code is UNKNOWN
func (r ProcessProposalResponse) IsStatusUnknown() bool {
	return r.Status == PROCESS_PROPOSAL_STATUS_UNKNOWN
}

func (r VerifyVoteExtensionResponse) IsAccepted() bool {
	return r.Status == VERIFY_VOTE_EXTENSION_STATUS_ACCEPT
}

// IsStatusUnknown returns true if Code is Unknown
func (r VerifyVoteExtensionResponse) IsStatusUnknown() bool {
	return r.Status == VERIFY_VOTE_EXTENSION_STATUS_UNKNOWN
}

// IsOK returns true if Code is OK.
func (r ExecTxResult) IsOK() bool {
	return r.Code == CodeTypeOK
}

// IsErr returns true if Code is something other than OK.
func (r ExecTxResult) IsErr() bool {
	return r.Code != CodeTypeOK
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

func (r *CheckTxResponse) MarshalJSON() ([]byte, error) {
	s, err := jsonpbMarshaller.MarshalToString(r)
	return []byte(s), err
}

func (r *CheckTxResponse) UnmarshalJSON(b []byte) error {
	reader := bytes.NewBuffer(b)
	return jsonpbUnmarshaller.Unmarshal(reader, r)
}

func (r *QueryResponse) MarshalJSON() ([]byte, error) {
	s, err := jsonpbMarshaller.MarshalToString(r)
	return []byte(s), err
}

func (r *QueryResponse) UnmarshalJSON(b []byte) error {
	reader := bytes.NewBuffer(b)
	return jsonpbUnmarshaller.Unmarshal(reader, r)
}

func (r *CommitResponse) MarshalJSON() ([]byte, error) {
	s, err := jsonpbMarshaller.MarshalToString(r)
	return []byte(s), err
}

func (r *CommitResponse) UnmarshalJSON(b []byte) error {
	reader := bytes.NewBuffer(b)
	return jsonpbUnmarshaller.Unmarshal(reader, r)
}

func (r *EventAttribute) MarshalJSON() ([]byte, error) {
	s, err := jsonpbMarshaller.MarshalToString(r)
	return []byte(s), err
}

func (r *EventAttribute) UnmarshalJSON(b []byte) error {
	reader := bytes.NewBuffer(b)
	return jsonpbUnmarshaller.Unmarshal(reader, r)
}

func (r *ExecTxResult) MarshalJSON() ([]byte, error) {
	s, err := jsonpbMarshaller.MarshalToString(r)
	return []byte(s), err
}

func (r *ExecTxResult) UnmarshalJSON(b []byte) error {
	reader := bytes.NewBuffer(b)
	return jsonpbUnmarshaller.Unmarshal(reader, r)
}
