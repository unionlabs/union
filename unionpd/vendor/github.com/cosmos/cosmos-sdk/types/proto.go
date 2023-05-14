package types

// CustomProtobufType defines the interface custom gogo proto types must implement
// in order to be used as a "customtype" extension.
//
// ref: https://github.com/cosmos/gogoproto/blob/master/custom_types.md
type CustomProtobufType interface {
	Marshal() ([]byte, error)
	MarshalTo(data []byte) (n int, err error)
	Unmarshal(data []byte) error
	Size() int

	MarshalJSON() ([]byte, error)
	UnmarshalJSON(data []byte) error
}
