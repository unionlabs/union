package errors

import (
	"fmt"
)

type ErrMsgToProto struct {
	MessageName string
	Err         error
}

func (e ErrMsgToProto) Error() string {
	return fmt.Sprintf("%s to proto error: %s", e.MessageName, e.Err.Error())
}

func (e ErrMsgToProto) Unwrap() error {
	return e.Err
}

type ErrMsgFromProto struct {
	MessageName string
	Err         error
}

func (e ErrMsgFromProto) Error() string {
	return fmt.Sprintf("%s msg from proto error: %s", e.MessageName, e.Err.Error())
}

func (e ErrMsgFromProto) Unwrap() error {
	return e.Err
}
