package types

import (
	"github.com/cosmos/gogoproto/proto"
	protov2 "google.golang.org/protobuf/proto"
)

// MsgTypeURL returns the TypeURL of a `sdk.Msg`.
func MsgTypeURL(msg proto.Message) string {
	if m, ok := msg.(protov2.Message); ok {
		return "/" + string(m.ProtoReflect().Descriptor().FullName())
	}

	return "/" + proto.MessageName(msg)
}
