package legacytx

import (
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// StdTxBuilder wraps StdTx to implement to the context.TxBuilder interface.
// Note that this type just exists for backwards compatibility with amino StdTx
// and will not work for protobuf transactions.
type StdTxBuilder struct {
	StdTx
	cdc *codec.LegacyAmino
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

// MarshalTx implements TxConfig.MarshalTx
func (s StdTxConfig) TxEncoder() sdk.TxEncoder {
	return DefaultTxEncoder(s.Cdc)
}

func (s StdTxConfig) TxJSONEncoder() sdk.TxEncoder {
	return func(tx sdk.Tx) ([]byte, error) {
		return s.Cdc.MarshalJSON(tx)
	}
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

// DefaultTxEncoder logic for standard transaction encoding
func DefaultTxEncoder(cdc *codec.LegacyAmino) sdk.TxEncoder {
	return func(tx sdk.Tx) ([]byte, error) {
		return cdc.Marshal(tx)
	}
}
