package tx

import (
	gogoproto "github.com/cosmos/gogoproto/proto"

	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/core/address"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/tx/decode"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
)

// DefaultJSONTxDecoder returns a default protobuf JSON TxDecoder using the provided Marshaler.
func DefaultJSONTxDecoder(addrCodec address.Codec, cdc codec.Codec, decoder *decode.Decoder) sdk.TxDecoder {
	return func(txBytes []byte) (sdk.Tx, error) {
		var jsonTx tx.Tx
		err := cdc.UnmarshalJSON(txBytes, &jsonTx)
		if err != nil {
			return nil, errorsmod.Wrap(sdkerrors.ErrTxDecode, err.Error())
		}

		// need to convert jsonTx into raw tx.
		bodyBytes, err := gogoproto.Marshal(jsonTx.Body)
		if err != nil {
			return nil, err
		}

		authInfoBytes, err := gogoproto.Marshal(jsonTx.AuthInfo)
		if err != nil {
			return nil, err
		}

		protoTxBytes, err := marshalOption.Marshal(&txv1beta1.TxRaw{
			BodyBytes:     bodyBytes,
			AuthInfoBytes: authInfoBytes,
			Signatures:    jsonTx.Signatures,
		})
		if err != nil {
			return nil, err
		}

		decodedTx, err := decoder.Decode(protoTxBytes)
		if err != nil {
			return nil, err
		}
		return newWrapperFromDecodedTx(addrCodec, cdc, decodedTx)
	}
}
