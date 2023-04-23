package legacytx

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// StdTxBuilder wraps StdTx to implement to the context.TxBuilder interface.
// Note that this type just exists for backwards compatibility with amino StdTx
// and will not work for protobuf transactions.
type StdTxBuilder struct {
	StdTx
	cdc *codec.LegacyAmino
}

// ensure interface implementation
var _ client.TxBuilder = &StdTxBuilder{}

// GetTx implements TxBuilder.GetTx
func (s *StdTxBuilder) GetTx() authsigning.Tx {
	return s.StdTx
}

// SetMsgs implements TxBuilder.SetMsgs
func (s *StdTxBuilder) SetMsgs(msgs ...sdk.Msg) error {
	s.Msgs = msgs
	return nil
}

// SetSignatures implements TxBuilder.SetSignatures.
func (s *StdTxBuilder) SetSignatures(signatures ...signing.SignatureV2) error {
	sigs := make([]StdSignature, len(signatures))
	var err error
	for i, sig := range signatures {
		sigs[i], err = SignatureV2ToStdSignature(s.cdc, sig)
		if err != nil {
			return err
		}
	}

	s.Signatures = sigs
	return nil
}

func (s *StdTxBuilder) SetFeeAmount(amount sdk.Coins) {
	s.StdTx.Fee.Amount = amount
}

func (s *StdTxBuilder) SetGasLimit(limit uint64) {
	s.StdTx.Fee.Gas = limit
}

func (s *StdTxBuilder) SetTip(tip *tx.Tip) {
	panic("StdTxBuilder does not support tips")
}

// SetMemo implements TxBuilder.SetMemo
func (s *StdTxBuilder) SetMemo(memo string) {
	s.Memo = memo
}

// SetTimeoutHeight sets the transaction's height timeout.
func (s *StdTxBuilder) SetTimeoutHeight(height uint64) {
	s.TimeoutHeight = height
}

// SetFeeGranter does nothing for stdtx
func (s *StdTxBuilder) SetFeeGranter(_ sdk.AccAddress) {}

// SetFeePayer does nothing for stdtx
func (s *StdTxBuilder) SetFeePayer(_ sdk.AccAddress) {}

// AddAuxSignerData returns an error for StdTxBuilder.
func (s *StdTxBuilder) AddAuxSignerData(_ tx.AuxSignerData) error {
	return sdkerrors.ErrLogic.Wrap("cannot use AuxSignerData with StdTxBuilder")
}

// StdTxConfig is a context.TxConfig for StdTx
type StdTxConfig struct {
	Cdc *codec.LegacyAmino
}

var _ client.TxConfig = StdTxConfig{}

// NewTxBuilder implements TxConfig.NewTxBuilder
func (s StdTxConfig) NewTxBuilder() client.TxBuilder {
	return &StdTxBuilder{
		StdTx: StdTx{},
		cdc:   s.Cdc,
	}
}

// WrapTxBuilder returns a StdTxBuilder from provided transaction
func (s StdTxConfig) WrapTxBuilder(newTx sdk.Tx) (client.TxBuilder, error) {
	stdTx, ok := newTx.(StdTx)
	if !ok {
		return nil, fmt.Errorf("wrong type, expected %T, got %T", stdTx, newTx)
	}
	return &StdTxBuilder{StdTx: stdTx, cdc: s.Cdc}, nil
}

// MarshalTx implements TxConfig.MarshalTx
func (s StdTxConfig) TxEncoder() sdk.TxEncoder {
	return DefaultTxEncoder(s.Cdc)
}

func (s StdTxConfig) TxDecoder() sdk.TxDecoder {
	return mkDecoder(s.Cdc.Unmarshal)
}

func (s StdTxConfig) TxJSONEncoder() sdk.TxEncoder {
	return func(tx sdk.Tx) ([]byte, error) {
		return s.Cdc.MarshalJSON(tx)
	}
}

func (s StdTxConfig) TxJSONDecoder() sdk.TxDecoder {
	return mkDecoder(s.Cdc.UnmarshalJSON)
}

func (s StdTxConfig) MarshalSignatureJSON(sigs []signing.SignatureV2) ([]byte, error) {
	stdSigs := make([]StdSignature, len(sigs))
	for i, sig := range sigs {
		stdSig, err := SignatureV2ToStdSignature(s.Cdc, sig)
		if err != nil {
			return nil, err
		}

		stdSigs[i] = stdSig
	}
	return s.Cdc.MarshalJSON(stdSigs)
}

func (s StdTxConfig) UnmarshalSignatureJSON(bz []byte) ([]signing.SignatureV2, error) {
	var stdSigs []StdSignature
	err := s.Cdc.UnmarshalJSON(bz, &stdSigs)
	if err != nil {
		return nil, err
	}

	sigs := make([]signing.SignatureV2, len(stdSigs))
	for i, stdSig := range stdSigs {
		sig, err := StdSignatureToSignatureV2(s.Cdc, stdSig)
		if err != nil {
			return nil, err
		}
		sigs[i] = sig
	}

	return sigs, nil
}

func (s StdTxConfig) SignModeHandler() authsigning.SignModeHandler {
	return stdTxSignModeHandler{}
}

// SignatureV2ToStdSignature converts a SignatureV2 to a StdSignature
// [Deprecated]
func SignatureV2ToStdSignature(cdc *codec.LegacyAmino, sig signing.SignatureV2) (StdSignature, error) {
	var (
		sigBz []byte
		err   error
	)

	if sig.Data != nil {
		sigBz, err = SignatureDataToAminoSignature(cdc, sig.Data)
		if err != nil {
			return StdSignature{}, err
		}
	}

	return StdSignature{
		PubKey:    sig.PubKey,
		Signature: sigBz,
	}, nil
}

// Unmarshaler is a generic type for Unmarshal functions
type Unmarshaler func(bytes []byte, ptr interface{}) error

func mkDecoder(unmarshaler Unmarshaler) sdk.TxDecoder {
	return func(txBytes []byte) (sdk.Tx, error) {
		if len(txBytes) == 0 {
			return nil, sdkerrors.Wrap(sdkerrors.ErrTxDecode, "tx bytes are empty")
		}
		tx := StdTx{}
		// StdTx.Msg is an interface. The concrete types
		// are registered by MakeTxCodec
		err := unmarshaler(txBytes, &tx)
		if err != nil {
			return nil, sdkerrors.Wrap(sdkerrors.ErrTxDecode, err.Error())
		}
		return tx, nil
	}
}

// DefaultTxEncoder logic for standard transaction encoding
func DefaultTxEncoder(cdc *codec.LegacyAmino) sdk.TxEncoder {
	return func(tx sdk.Tx) ([]byte, error) {
		return cdc.Marshal(tx)
	}
}
