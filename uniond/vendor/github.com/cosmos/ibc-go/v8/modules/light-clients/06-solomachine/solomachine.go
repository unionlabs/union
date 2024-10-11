package solomachine

import (
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// Interface implementation checks.
var _, _, _, _ gogoprotoany.UnpackInterfacesMessage = (*ClientState)(nil), (*ConsensusState)(nil), (*Header)(nil), (*HeaderData)(nil)

// Data is an interface used for all the signature data bytes proto definitions.
type Data interface{}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (cs ClientState) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return cs.ConsensusState.UnpackInterfaces(unpacker)
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (cs ConsensusState) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return unpacker.UnpackAny(cs.PublicKey, new(cryptotypes.PubKey))
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (h Header) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return unpacker.UnpackAny(h.NewPublicKey, new(cryptotypes.PubKey))
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (hd HeaderData) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	return unpacker.UnpackAny(hd.NewPubKey, new(cryptotypes.PubKey))
}
