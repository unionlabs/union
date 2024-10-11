package tx

import (
	"fmt"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/proto"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdktx "github.com/cosmos/cosmos-sdk/types/tx"
)

// DefaultTxEncoder returns a default protobuf TxEncoder using the provided Marshaler
func DefaultTxEncoder() sdk.TxEncoder {
	return func(tx sdk.Tx) ([]byte, error) {
		gogoWrapper, ok := tx.(*gogoTxWrapper)
		if !ok {
			return nil, fmt.Errorf("unexpected tx type: %T", tx)
		}
		return marshalOption.Marshal(gogoWrapper.TxRaw)
	}
}

// DefaultJSONTxEncoder returns a default protobuf JSON TxEncoder using the provided Marshaler.
func DefaultJSONTxEncoder(cdc codec.Codec) sdk.TxEncoder {
	return func(tx sdk.Tx) ([]byte, error) {
		// 1) Unwrap the tx
		gogoWrapper, ok := tx.(*gogoTxWrapper)
		if !ok {
			return nil, fmt.Errorf("unexpected tx type: %T", tx)
		}
		// The unwrapped tx is a pulsar message, but SDK spec for marshaling JSON is AminoJSON.
		// AminoJSON only operates on gogoproto structures, so we need to convert the pulsar message to a "v1" (gogoproto) Tx.
		// see: https://github.com/cosmos/cosmos-sdk/issues/20431 and associated PRs for an eventual fix.
		//
		// 2) Marshal the pulsar message to bytes
		bz, err := proto.Marshal(gogoWrapper.Tx)
		if err != nil {
			return nil, err
		}
		// 3) Umarshal the bytes to a "v1" (gogoproto) Tx
		v1Tx := &sdktx.Tx{}
		err = gogoproto.Unmarshal(bz, v1Tx)
		if err != nil {
			return nil, err
		}
		// 4) Marshal the "v1" (gogoproto) to Amino ProtoJSON
		return cdc.MarshalJSON(v1Tx)
	}
}
