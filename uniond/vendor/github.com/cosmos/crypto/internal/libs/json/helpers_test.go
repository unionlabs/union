package json_test

import (
	"github.com/cosmos/crypto/internal/libs/json"
	"time"
)

// Register Car, an instance of the Vehicle interface.
func init() {
	json.RegisterType(&Car{}, "vehicle/car")
	json.RegisterType(Boat{}, "vehicle/boat")
	json.RegisterType(PublicKey{}, "key/public")
	json.RegisterType(PrivateKey{}, "key/private")
}

type Vehicle interface {
	Drive() error
}

// Car is a pointer implementation of Vehicle.
type Car struct {
	Wheels int32
}

func (*Car) Drive() error { return nil }

// Boat is a value implementation of Vehicle.
type Boat struct {
	Sail bool
}

func (Boat) Drive() error { return nil }

// These are public and private encryption keys.
type (
	PublicKey  [8]byte
	PrivateKey [8]byte
)

// Custom has custom marshalers and unmarshalers, taking pointer receivers.
type CustomPtr struct {
	Value string
}

func (*CustomPtr) MarshalJSON() ([]byte, error) {
	return []byte("\"custom\""), nil
}

func (c *CustomPtr) UnmarshalJSON(_ []byte) error {
	c.Value = "custom"
	return nil
}

// CustomValue has custom marshalers and unmarshalers, taking value receivers (which usually doesn't
// make much sense since the unmarshaler can't change anything).
type CustomValue struct {
	Value string
}

func (CustomValue) MarshalJSON() ([]byte, error) {
	return []byte("\"custom\""), nil
}

func (CustomValue) UnmarshalJSON(_ []byte) error {
	return nil
}

// Tags tests JSON tags.
type Tags struct {
	JSONName  string `json:"name"`
	OmitEmpty string `json:",omitempty"`
	Hidden    string `json:"-"`
	Tags      *Tags  `json:"tags,omitempty"`
}

// Struct tests structs with lots of contents.
type Struct struct {
	Bool         bool
	Float64      float64
	Int32        int32
	Int64        int64
	Int64Ptr     *int64
	String       string
	StringPtrPtr **string
	Bytes        []byte
	Time         time.Time
	Car          *Car
	Boat         Boat
	Vehicles     []Vehicle
	Child        *Struct
	private      string
}
