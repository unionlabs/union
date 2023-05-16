package solomachine

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// Interface implementation checks.
var _, _, _, _ codectypes.UnpackInterfacesMessage = (*ClientState)(nil), (*ConsensusState)(nil), (*Header)(nil), (*HeaderData)(nil)

// Data is an interface used for all the signature data bytes proto definitions.
type Data interface{}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (cs ClientState) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return cs.ConsensusState.UnpackInterfaces(unpacker)
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (cs ConsensusState) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(cs.PublicKey, new(cryptotypes.PubKey))
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (h Header) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(h.NewPublicKey, new(cryptotypes.PubKey))
}

// UnpackInterfaces implements the UnpackInterfaceMessages.UnpackInterfaces method
func (hd HeaderData) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(hd.NewPubKey, new(cryptotypes.PubKey))
}
