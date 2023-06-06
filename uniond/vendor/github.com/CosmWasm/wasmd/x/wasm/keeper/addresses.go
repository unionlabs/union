package keeper

import (
	"encoding/binary"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// AddressGenerator abstract address generator to be used for a single contract address
type AddressGenerator func(ctx sdk.Context, codeID uint64, checksum []byte) sdk.AccAddress

// ClassicAddressGenerator generates a contract address using codeID and instanceID sequence
func (k Keeper) ClassicAddressGenerator() AddressGenerator {
	return func(ctx sdk.Context, codeID uint64, _ []byte) sdk.AccAddress {
		instanceID := k.autoIncrementID(ctx, types.KeyLastInstanceID)
		return BuildContractAddressClassic(codeID, instanceID)
	}
}

// PredicableAddressGenerator generates a predictable contract address
func PredicableAddressGenerator(creator sdk.AccAddress, salt []byte, msg []byte, fixMsg bool) AddressGenerator {
	return func(ctx sdk.Context, _ uint64, checksum []byte) sdk.AccAddress {
		if !fixMsg { // clear msg to not be included in the address generation
			msg = []byte{}
		}
		return BuildContractAddressPredictable(checksum, creator, salt, msg)
	}
}

// BuildContractAddressClassic builds an sdk account address for a contract.
func BuildContractAddressClassic(codeID, instanceID uint64) sdk.AccAddress {
	contractID := make([]byte, 16)
	binary.BigEndian.PutUint64(contractID[:8], codeID)
	binary.BigEndian.PutUint64(contractID[8:], instanceID)
	return address.Module(types.ModuleName, contractID)[:types.ContractAddrLen]
}

// BuildContractAddressPredictable generates a contract address for the wasm module with len = types.ContractAddrLen using the
// Cosmos SDK address.Module function.
// Internally a key is built containing:
// (len(checksum) | checksum | len(sender_address) | sender_address | len(salt) | salt| len(initMsg) | initMsg).
//
// All method parameter values must be valid and not nil.
func BuildContractAddressPredictable(checksum []byte, creator sdk.AccAddress, salt, initMsg types.RawContractMessage) sdk.AccAddress {
	if len(checksum) != 32 {
		panic("invalid checksum")
	}
	if err := sdk.VerifyAddressFormat(creator); err != nil {
		panic(fmt.Sprintf("creator: %s", err))
	}
	if err := types.ValidateSalt(salt); err != nil {
		panic(fmt.Sprintf("salt: %s", err))
	}
	if err := initMsg.ValidateBasic(); len(initMsg) != 0 && err != nil {
		panic(fmt.Sprintf("initMsg: %s", err))
	}
	checksum = UInt64LengthPrefix(checksum)
	creator = UInt64LengthPrefix(creator)
	salt = UInt64LengthPrefix(salt)
	initMsg = UInt64LengthPrefix(initMsg)
	key := make([]byte, len(checksum)+len(creator)+len(salt)+len(initMsg))
	copy(key[0:], checksum)
	copy(key[len(checksum):], creator)
	copy(key[len(checksum)+len(creator):], salt)
	copy(key[len(checksum)+len(creator)+len(salt):], initMsg)
	return address.Module(types.ModuleName, key)[:types.ContractAddrLen]
}

// UInt64LengthPrefix prepend big endian encoded byte length
func UInt64LengthPrefix(bz []byte) []byte {
	return append(sdk.Uint64ToBigEndian(uint64(len(bz))), bz...)
}
