package keyring

import (
	"errors"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	errorsmod "cosmossdk.io/errors"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types"
)

var (
	// ErrPrivKeyExtr is used to output an error if extraction of a private key from Local item fails.
	ErrPrivKeyExtr = errors.New("private key extraction works only for Local")
	// ErrPrivKeyNotAvailable is used when a Record_Local.PrivKey is nil.
	ErrPrivKeyNotAvailable = errors.New("private key is not available")
	// ErrCastAny is used to output an error if cast from types.Any fails.
	ErrCastAny = errors.New("unable to cast to cryptotypes")
)

func newRecord(name string, pk cryptotypes.PubKey, item isRecord_Item) (*Record, error) {
	any, err := codectypes.NewAnyWithValue(pk)
	if err != nil {
		return nil, err
	}

	return &Record{name, any, item}, nil
}

// NewLocalRecord creates a new Record with local key item
func NewLocalRecord(name string, priv cryptotypes.PrivKey, pk cryptotypes.PubKey) (*Record, error) {
	any, err := codectypes.NewAnyWithValue(priv)
	if err != nil {
		return nil, err
	}

	recordLocal := &Record_Local{any}
	recordLocalItem := &Record_Local_{recordLocal}

	return newRecord(name, pk, recordLocalItem)
}

// NewLedgerRecord creates a new Record with ledger item
func NewLedgerRecord(name string, pk cryptotypes.PubKey, path *hd.BIP44Params) (*Record, error) {
	recordLedger := &Record_Ledger{path}
	recordLedgerItem := &Record_Ledger_{recordLedger}
	return newRecord(name, pk, recordLedgerItem)
}

func (rl *Record_Ledger) GetPath() *hd.BIP44Params {
	return rl.Path
}

// NewOfflineRecord creates a new Record with offline item
func NewOfflineRecord(name string, pk cryptotypes.PubKey) (*Record, error) {
	recordOffline := &Record_Offline{}
	recordOfflineItem := &Record_Offline_{recordOffline}
	return newRecord(name, pk, recordOfflineItem)
}

// NewMultiRecord creates a new Record with multi item
func NewMultiRecord(name string, pk cryptotypes.PubKey) (*Record, error) {
	recordMulti := &Record_Multi{}
	recordMultiItem := &Record_Multi_{recordMulti}
	return newRecord(name, pk, recordMultiItem)
}

// GetPubKey fetches a public key of the record
func (k *Record) GetPubKey() (cryptotypes.PubKey, error) {
	pk, ok := k.PubKey.GetCachedValue().(cryptotypes.PubKey)
	if !ok {
		return nil, errorsmod.Wrap(ErrCastAny, "PubKey")
	}

	return pk, nil
}

// GetAddress fetches an address of the record
func (k Record) GetAddress() (types.AccAddress, error) {
	pk, err := k.GetPubKey()
	if err != nil {
		return nil, err
	}
	return pk.Address().Bytes(), nil
}

// GetType fetches type of the record
func (k Record) GetType() KeyType {
	switch {
	case k.GetLocal() != nil:
		return TypeLocal
	case k.GetLedger() != nil:
		return TypeLedger
	case k.GetMulti() != nil:
		return TypeMulti
	case k.GetOffline() != nil:
		return TypeOffline
	default:
		panic("unrecognized record type")
	}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (k *Record) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var pk cryptotypes.PubKey
	if err := unpacker.UnpackAny(k.PubKey, &pk); err != nil {
		return err
	}

	if l := k.GetLocal(); l != nil {
		var priv cryptotypes.PrivKey
		return unpacker.UnpackAny(l.PrivKey, &priv)
	}

	return nil
}

func extractPrivKeyFromRecord(k *Record) (cryptotypes.PrivKey, error) {
	rl := k.GetLocal()
	if rl == nil {
		return nil, ErrPrivKeyExtr
	}

	return extractPrivKeyFromLocal(rl)
}

func extractPrivKeyFromLocal(rl *Record_Local) (cryptotypes.PrivKey, error) {
	if rl.PrivKey == nil {
		return nil, ErrPrivKeyNotAvailable
	}

	priv, ok := rl.PrivKey.GetCachedValue().(cryptotypes.PrivKey)
	if !ok {
		return nil, errorsmod.Wrap(ErrCastAny, "PrivKey")
	}

	return priv, nil
}
