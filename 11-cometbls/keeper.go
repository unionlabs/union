package cometbls

import (
	"bytes"
	"encoding/hex"

	"cosmossdk.io/collections"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
)

// Keeper defines the structure for your keeper
type Keeper struct {
	storeKey  storetypes.StoreKey
	cdc       codec.BinaryCodec
	authority string
	checksums collections.KeySet[[]byte]
}

// NewKeeper creates a new Keeper instance with authority
func NewKeeper(cdc codec.BinaryCodec, storeKey storetypes.StoreKey, authority string) Keeper {
	return Keeper{
		storeKey:  storeKey,
		cdc:       cdc,
		authority: authority,
	}
}

// SetAuthority sets the governance authority in the store
func (k Keeper) SetAuthority(ctx sdk.Context, address sdk.AccAddress) {
	store := ctx.KVStore(k.storeKey)
	store.Set([]byte(AuthorityKey), address.Bytes())
}

// GetAuthority retrieves the governance authority from the store
func (k Keeper) GetAuthority(ctx sdk.Context) (sdk.AccAddress, error) {
	store := ctx.KVStore(k.storeKey)
	authorityBytes := store.Get([]byte(AuthorityKey))
	if authorityBytes == nil {
		return nil, errorsmod.Wrapf(ErrUnauthorized, "authority not set")
	}

	return sdk.AccAddress(authorityBytes), nil

}

func (k Keeper) SetVerifyingKey(ctx sdk.Context, msg *MsgMyFunction) error {
	authority, err := k.GetAuthority(ctx)
	if err != nil {
		return err
	}

	if !msg.Sender.Equals(authority) {
		return errorsmod.Wrap(errors.ErrUnauthorized, "sender is not authorized")
	}

	vkHex := msg.vkHex
	vkBytes, err := hex.DecodeString(vkHex)
	if err != nil {
		return errorsmod.Wrapf(errors.ErrInvalidRequest, "could not decode the hex verifying key: '%s'", vkHex)
	}

	var verifyingKey backend_bn254.VerifyingKey
	_, err = verifyingKey.ReadFrom(bytes.NewReader(vkBytes))
	if err != nil {
		return errorsmod.Wrapf(errors.ErrInvalidRequest, "could not read the verifying key: '%s'", vkHex)
	}

	var keyBuffer bytes.Buffer
	_, err = verifyingKey.WriteTo(&keyBuffer)
	if err != nil {
		return errorsmod.Wrap(err, "failed to serialize the verifying key")
	}

	store := ctx.KVStore(k.storeKey)
	store.Set([]byte("verifyingKey"), keyBuffer.Bytes())

	return nil
}

func (k Keeper) GetVerifyingKey(ctx sdk.Context) (*backend_bn254.VerifyingKey, error) {
	store := ctx.KVStore(k.storeKey)

	vkBytes := store.Get([]byte("verifyingKey")) // Use the same key as when we stored it
	if vkBytes == nil {
		return nil, errorsmod.Wrapf(ErrVerifyingKeyNotFound, "verifying key not found")
	}

	var verifyingKey backend_bn254.VerifyingKey
	_, err := verifyingKey.ReadFrom(bytes.NewReader(vkBytes))
	if err != nil {
		return nil, errorsmod.Wrapf(ErrVerifyingKeyNotFound, "failed to deserialize verifying key")
	}

	return &verifyingKey, nil
}
