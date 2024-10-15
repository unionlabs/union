package keyring

import (
	"errors"
	"fmt"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Deprecated: LegacyInfo is the publicly exposed information about a keypair
type LegacyInfo interface {
	// GetType human-readable type for key listing
	GetType() KeyType
	// GetName name of the key
	GetName() string
	// GetPubKey public key
	GetPubKey() cryptotypes.PubKey
	// GetAddress address
	GetAddress() sdk.AccAddress
	// GetPath bip44 path
	GetPath() (*hd.BIP44Params, error)
	// GetAlgo signing algorithm for the key
	GetAlgo() hd.PubKeyType
}

var (
	_ LegacyInfo = &legacyLocalInfo{}
	_ LegacyInfo = &legacyLedgerInfo{}
	_ LegacyInfo = &legacyOfflineInfo{}
	_ LegacyInfo = &LegacyMultiInfo{}
)

// legacyLocalInfo is the public information about a locally stored key
// Note: Algo must be last field in struct for backwards amino compatibility
type legacyLocalInfo struct {
	Name         string             `json:"name"`
	PubKey       cryptotypes.PubKey `json:"pubkey"`
	PrivKeyArmor string             `json:"privkey.armor"`
	Algo         hd.PubKeyType      `json:"algo"`
}

func infoKey(name string) string { return fmt.Sprintf("%s.%s", name, infoSuffix) }

// GetType returns human-readable type for key listing
func (i legacyLocalInfo) GetType() KeyType {
	return TypeLocal
}

// GetName returns name of the key
func (i legacyLocalInfo) GetName() string {
	return i.Name
}

// GetPubKey returns public key
func (i legacyLocalInfo) GetPubKey() cryptotypes.PubKey {
	return i.PubKey
}

// GetAddress returns address
func (i legacyLocalInfo) GetAddress() sdk.AccAddress {
	return i.PubKey.Address().Bytes()
}

// GetPrivKeyArmor returns privkey armor
func (i legacyLocalInfo) GetPrivKeyArmor() string {
	return i.PrivKeyArmor
}

// GetAlgo returns the signing algorithm for the key
func (i legacyLocalInfo) GetAlgo() hd.PubKeyType {
	return i.Algo
}

// GetPath returns bip44 path, but not available for this type
func (i legacyLocalInfo) GetPath() (*hd.BIP44Params, error) {
	return nil, errors.New("BIP44 Paths are not available for this type")
}

// legacyLedgerInfo is the public information about a Ledger key
// Note: Algo must be last field in struct for backwards amino compatibility
type legacyLedgerInfo struct {
	Name   string             `json:"name"`
	PubKey cryptotypes.PubKey `json:"pubkey"`
	Path   hd.BIP44Params     `json:"path"`
	Algo   hd.PubKeyType      `json:"algo"`
}

// GetType returns human-readable type for key listing
func (i legacyLedgerInfo) GetType() KeyType {
	return TypeLedger
}

// GetName returns name of the key
func (i legacyLedgerInfo) GetName() string {
	return i.Name
}

// GetPubKey returns public key
func (i legacyLedgerInfo) GetPubKey() cryptotypes.PubKey {
	return i.PubKey
}

// GetAddress returns address
func (i legacyLedgerInfo) GetAddress() sdk.AccAddress {
	return i.PubKey.Address().Bytes()
}

// GetAlgo returns the signing algorithm for the key
func (i legacyLedgerInfo) GetAlgo() hd.PubKeyType {
	return i.Algo
}

// GetPath returns bip44 path
func (i legacyLedgerInfo) GetPath() (*hd.BIP44Params, error) {
	tmp := i.Path
	return &tmp, nil
}

// legacyOfflineInfo is the public information about an offline key
// Note: Algo must be last field in struct for backwards amino compatibility
type legacyOfflineInfo struct {
	Name   string             `json:"name"`
	PubKey cryptotypes.PubKey `json:"pubkey"`
	Algo   hd.PubKeyType      `json:"algo"`
}

// GetType returns human-readable type for key listing
func (i legacyOfflineInfo) GetType() KeyType {
	return TypeOffline
}

// GetName returns name of the key
func (i legacyOfflineInfo) GetName() string {
	return i.Name
}

// GetPubKey returns public key
func (i legacyOfflineInfo) GetPubKey() cryptotypes.PubKey {
	return i.PubKey
}

// GetAlgo returns the signing algorithm for the key
func (i legacyOfflineInfo) GetAlgo() hd.PubKeyType {
	return i.Algo
}

// GetAddress returns address
func (i legacyOfflineInfo) GetAddress() sdk.AccAddress {
	return i.PubKey.Address().Bytes()
}

// GetPath returns bip44 path, but not available for this type
func (i legacyOfflineInfo) GetPath() (*hd.BIP44Params, error) {
	return nil, errors.New("BIP44 Paths are not available for this type")
}

// Deprecated: this structure is not used anymore and it's here only to allow
// decoding old multiInfo records from keyring.
// The problem with legacy.Cdc.UnmarshalLengthPrefixed - the legacy codec doesn't
// tolerate extensibility.
type multisigPubKeyInfo struct {
	PubKey cryptotypes.PubKey `json:"pubkey"`
	Weight uint               `json:"weight"`
}

// LegacyMultiInfo multiInfo is the public information about a multisig key
type LegacyMultiInfo struct {
	Name      string               `json:"name"`
	PubKey    cryptotypes.PubKey   `json:"pubkey"`
	Threshold uint                 `json:"threshold"`
	PubKeys   []multisigPubKeyInfo `json:"pubkeys"`
}

// NewLegacyMultiInfo creates a new legacyMultiInfo instance
func NewLegacyMultiInfo(name string, pub cryptotypes.PubKey) (LegacyInfo, error) {
	if _, ok := pub.(*multisig.LegacyAminoPubKey); !ok {
		return nil, fmt.Errorf("MultiInfo supports only multisig.LegacyAminoPubKey, got  %T", pub)
	}
	return &LegacyMultiInfo{
		Name:   name,
		PubKey: pub,
	}, nil
}

// GetType returns human-readable type for key listing
func (i LegacyMultiInfo) GetType() KeyType {
	return TypeMulti
}

// GetName returns name of the key
func (i LegacyMultiInfo) GetName() string {
	return i.Name
}

// GetPubKey returns public key
func (i LegacyMultiInfo) GetPubKey() cryptotypes.PubKey {
	return i.PubKey
}

// GetAddress returns address
func (i LegacyMultiInfo) GetAddress() sdk.AccAddress {
	return i.PubKey.Address().Bytes()
}

// GetAlgo returns the signing algorithm for the key
func (i LegacyMultiInfo) GetAlgo() hd.PubKeyType {
	return hd.MultiType
}

// GetPath returns bip44 path, but not available for this type
func (i LegacyMultiInfo) GetPath() (*hd.BIP44Params, error) {
	return nil, errors.New("BIP44 Paths are not available for this type")
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (i LegacyMultiInfo) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	multiPK := i.PubKey.(*multisig.LegacyAminoPubKey)

	return codectypes.UnpackInterfaces(multiPK, unpacker)
}

// MarshalInfo encoding info
func MarshalInfo(i LegacyInfo) []byte {
	return legacy.Cdc.MustMarshalLengthPrefixed(i)
}

// decoding info
func unMarshalLegacyInfo(bz []byte) (info LegacyInfo, err error) {
	err = legacy.Cdc.UnmarshalLengthPrefixed(bz, &info)
	if err != nil {
		return nil, err
	}

	// After unmarshalling into &info, if we notice that the info is a
	// multiInfo, then we unmarshal again, explicitly in a multiInfo this time.
	// Since multiInfo implements UnpackInterfacesMessage, this will correctly
	// unpack the underlying anys inside the multiInfo.
	//
	// This is a workaround, as go cannot check that an interface (Info)
	// implements another interface (UnpackInterfacesMessage).
	_, ok := info.(LegacyMultiInfo)
	if ok {
		var multi LegacyMultiInfo
		err = legacy.Cdc.UnmarshalLengthPrefixed(bz, &multi)

		return multi, err
	}

	return
}

// privKeyFromLegacyInfo exports a private key from LegacyInfo
func privKeyFromLegacyInfo(info LegacyInfo) (cryptotypes.PrivKey, error) {
	switch linfo := info.(type) {
	case legacyLocalInfo:
		if linfo.PrivKeyArmor == "" {
			return nil, errors.New("private key not available")
		}
		priv, err := legacy.PrivKeyFromBytes([]byte(linfo.PrivKeyArmor))
		if err != nil {
			return nil, err
		}

		return priv, nil
	// case legacyLedgerInfo, legacyOfflineInfo, LegacyMultiInfo:
	default:
		return nil, fmt.Errorf("only works on local private keys, provided %s", linfo)
	}
}
