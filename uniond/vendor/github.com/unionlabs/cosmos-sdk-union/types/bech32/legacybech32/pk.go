// Deprecated: The module provides legacy bech32 functions which will be removed in a future
// release.
package legacybech32

import (
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/bech32"
)

// TODO: when removing this package remove:
// + sdk:config.GetBech32AccountPubPrefix (and other related functions)
// + Bech32PrefixAccAddr and other related constants

// Deprecated: Bech32PubKeyType defines a string type alias for a Bech32 public key type.
type Bech32PubKeyType string

// Bech32 conversion constants
const (
	AccPK  Bech32PubKeyType = "accpub"
	ValPK  Bech32PubKeyType = "valpub"
	ConsPK Bech32PubKeyType = "conspub"
)

// Deprecated: MarshalPubKey returns a Bech32 encoded string containing the appropriate
// prefix based on the key type provided for a given PublicKey.
func MarshalPubKey(pkt Bech32PubKeyType, pubkey cryptotypes.PubKey) (string, error) {
	bech32Prefix := getPrefix(pkt)
	return bech32.ConvertAndEncode(bech32Prefix, legacy.Cdc.MustMarshal(pubkey))
}

// Deprecated: MustMarshalPubKey calls MarshalPubKey and panics on error.
func MustMarshalPubKey(pkt Bech32PubKeyType, pubkey cryptotypes.PubKey) string {
	res, err := MarshalPubKey(pkt, pubkey)
	if err != nil {
		panic(err)
	}

	return res
}

func getPrefix(pkt Bech32PubKeyType) string {
	cfg := sdk.GetConfig()
	switch pkt {
	case AccPK:
		return cfg.GetBech32AccountPubPrefix()

	case ValPK:
		return cfg.GetBech32ValidatorPubPrefix()
	case ConsPK:
		return cfg.GetBech32ConsensusPubPrefix()
	}

	return ""
}

// Deprecated: UnmarshalPubKey returns a PublicKey from a bech32-encoded PublicKey with
// a given key type.
func UnmarshalPubKey(pkt Bech32PubKeyType, pubkeyStr string) (cryptotypes.PubKey, error) {
	bech32Prefix := getPrefix(pkt)

	bz, err := sdk.GetFromBech32(pubkeyStr, bech32Prefix)
	if err != nil {
		return nil, err
	}
	return legacy.PubKeyFromBytes(bz)
}
