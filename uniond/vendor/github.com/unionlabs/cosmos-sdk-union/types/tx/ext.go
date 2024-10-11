package tx

import (
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"github.com/cosmos/cosmos-sdk/codec/types"
)

// TxExtensionOptionI defines the interface for tx extension options
type TxExtensionOptionI interface{}

// unpackTxExtensionOptionsI unpacks Any's to TxExtensionOptionI's.
func unpackTxExtensionOptionsI(unpacker gogoprotoany.AnyUnpacker, anys []*types.Any) error {
	for _, any := range anys {
		var opt TxExtensionOptionI
		err := unpacker.UnpackAny(any, &opt)
		if err != nil {
			return err
		}
	}

	return nil
}
