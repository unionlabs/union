package blocksync

import (
	"errors"
	"fmt"

	"github.com/cosmos/gogoproto/proto"
)

var (
	// ErrNilMessage is returned when provided message is empty
	ErrNilMessage = errors.New("message cannot be nil")
)

// ErrInvalidBase is returned when peer informs of a status with invalid height
type ErrInvalidHeight struct {
	Height int64
	Reason string
}

func (e ErrInvalidHeight) Error() string {
	return fmt.Sprintf("invalid height %v: %s", e.Height, e.Reason)
}

// ErrInvalidBase is returned when peer informs of a status with invalid base
type ErrInvalidBase struct {
	Base   int64
	Reason string
}

func (e ErrInvalidBase) Error() string {
	return fmt.Sprintf("invalid base %v: %s", e.Base, e.Reason)
}

type ErrUnknownMessageType struct {
	Msg proto.Message
}

func (e ErrUnknownMessageType) Error() string {
	return fmt.Sprintf("unknown message type %T", e.Msg)
}

type ErrReactorValidation struct {
	Err error
}

func (e ErrReactorValidation) Error() string {
	return fmt.Sprintf("reactor validation error: %v", e.Err)
}

func (e ErrReactorValidation) Unwrap() error {
	return e.Err
}
