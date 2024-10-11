package offchain

import (
	"fmt"

	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/proto"

	apitx "cosmossdk.io/api/cosmos/tx/v1beta1"
	v2flags "cosmossdk.io/client/v2/internal/flags"
)

// marshaller marshals Messages.
type marshaller interface {
	Marshal(message proto.Message) ([]byte, error)
}

// getMarshaller returns the marshaller for the given marshaller id.
func getMarshaller(marshallerId, indent string, emitUnpopulated bool) (marshaller, error) {
	switch marshallerId {
	case v2flags.OutputFormatJSON:
		return protojson.MarshalOptions{
			Indent:          indent,
			EmitUnpopulated: emitUnpopulated,
		}, nil
	case v2flags.OutputFormatText:
		return prototext.MarshalOptions{
			Indent:      indent,
			EmitUnknown: emitUnpopulated,
		}, nil
	}
	return nil, fmt.Errorf("marshaller with id '%s' not identified", marshallerId)
}

// marshalOffChainTx marshals a Tx using given marshaller.
func marshalOffChainTx(tx *apitx.Tx, marshaller marshaller) (string, error) {
	bytesTx, err := marshaller.Marshal(tx)
	if err != nil {
		return "", err
	}
	return string(bytesTx), nil
}
