package payload

import (
	"bytes"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"math"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

const (
	keyPrefix      = "a="
	maxPayloadSize = 4 * 1024 * 1024
)

// NewBytes generates a new payload and returns the encoded representation of
// the payload as a slice of bytes. NewBytes uses the fields on the Options
// to create the payload.
func NewBytes(p *Payload) ([]byte, error) {
	p.Padding = make([]byte, 1)
	if p.GetTime() == nil {
		p.Time = timestamppb.Now()
	}
	us, err := CalculateUnpaddedSize(p)
	if err != nil {
		return nil, err
	}
	if p.GetSize() > maxPayloadSize {
		return nil, fmt.Errorf("configured size %d is too large (>%d)", p.GetSize(), maxPayloadSize)
	}
	pSize := int(p.GetSize()) // #nosec -- The "if" above makes this cast safe
	if pSize < us {
		return nil, fmt.Errorf("configured size %d not large enough to fit unpadded transaction of size %d", pSize, us)
	}

	// We halve the padding size because we transform the TX to hex
	p.Padding = make([]byte, (pSize-us)/2)
	_, err = rand.Read(p.GetPadding())
	if err != nil {
		return nil, err
	}
	b, err := proto.Marshal(p)
	if err != nil {
		return nil, err
	}
	h := []byte(hex.EncodeToString(b))

	// prepend a single key so that the kv store only ever stores a single
	// transaction instead of storing all tx and ballooning in size.
	return append([]byte(keyPrefix), h...), nil
}

// FromBytes extracts a paylod from the byte representation of the payload.
// FromBytes leaves the padding untouched, returning it to the caller to handle
// or discard per their preference.
func FromBytes(b []byte) (*Payload, error) {
	trH := bytes.TrimPrefix(b, []byte(keyPrefix))
	if bytes.Equal(b, trH) {
		return nil, fmt.Errorf("payload bytes missing key prefix '%s'", keyPrefix)
	}
	trB, err := hex.DecodeString(string(trH))
	if err != nil {
		return nil, err
	}

	p := &Payload{}
	err = proto.Unmarshal(trB, p)
	if err != nil {
		return nil, err
	}
	return p, nil
}

// MaxUnpaddedSize returns the maximum size that a payload may be if no padding
// is included.
func MaxUnpaddedSize() (int, error) {
	p := &Payload{
		Time:        timestamppb.Now(),
		Connections: math.MaxUint64,
		Rate:        math.MaxUint64,
		Size:        math.MaxUint64,
		Padding:     make([]byte, 1),
	}
	return CalculateUnpaddedSize(p)
}

// CalculateUnpaddedSize calculates the size of the passed in payload for the
// purpose of determining how much padding to add to add to reach the target size.
// CalculateUnpaddedSize returns an error if the payload Padding field is longer than 1.
func CalculateUnpaddedSize(p *Payload) (int, error) {
	if len(p.GetPadding()) != 1 {
		return 0, fmt.Errorf("expected length of padding to be 1, received %d", len(p.GetPadding()))
	}
	b, err := proto.Marshal(p)
	if err != nil {
		return 0, err
	}
	h := []byte(hex.EncodeToString(b))
	return len(h) + len(keyPrefix), nil
}
