package types

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

type (
	// Msg defines the interface a transaction message needed to fulfill.
	Msg = transaction.Msg

	// LegacyMsg defines the interface a transaction message needed to fulfill up through
	// v0.47.
	LegacyMsg interface {
		Msg

		// GetSigners returns the addrs of signers that must sign.
		// CONTRACT: All signatures must be present to be valid.
		// CONTRACT: Returns addrs in some deterministic order.
		GetSigners() []AccAddress
	}

	// Fee defines an interface for an application application-defined concrete
	// transaction type to be able to set and return the transaction fee.
	Fee interface {
		GetGas() uint64
		GetAmount() Coins
	}

	// Signature defines an interface for an application application-defined
	// concrete transaction type to be able to set and return transaction signatures.
	Signature interface {
		GetPubKey() cryptotypes.PubKey
		GetSignature() []byte
	}

	// HasMsgs defines an interface a transaction must fulfill.
	HasMsgs interface {
		// GetMsgs gets the all the transaction's messages.
		GetMsgs() []Msg
	}

	// Tx defines an interface a transaction must fulfill.
	Tx interface {
		transaction.Tx

		HasMsgs

		// GetReflectMessages gets a reflected version of the transaction's messages
		// that can be used by dynamic APIs. These messages should not be used for actual
		// processing as they cannot be guaranteed to be what handlers are expecting, but
		// they can be used for dynamically reading specific fields from the message such
		// as signers or validation data. Message processors should ALWAYS use the messages
		// returned by GetMsgs.
		GetReflectMessages() ([]protoreflect.Message, error)
	}

	// FeeTx defines the interface to be implemented by Tx to use the FeeDecorators
	FeeTx interface {
		Tx
		GetGas() uint64
		GetFee() Coins
		FeePayer() []byte
		FeeGranter() []byte
	}

	// TxWithMemo must have GetMemo() method to use ValidateMemoDecorator
	TxWithMemo interface {
		Tx
		GetMemo() string
	}

	// TxWithTimeoutTimeStamp extends the Tx interface by allowing a transaction to
	// set a timeout timestamp.
	TxWithTimeoutTimeStamp interface {
		Tx

		GetTimeoutTimeStamp() time.Time
	}

	// TxWithTimeoutHeight extends the Tx interface by allowing a transaction to
	// set a height timeout.
	TxWithTimeoutHeight interface {
		Tx

		GetTimeoutHeight() uint64
	}

	// TxWithUnordered extends the Tx interface by allowing a transaction to set
	// the unordered field, which implicitly relies on TxWithTimeoutTimeStamp.
	TxWithUnordered interface {
		TxWithTimeoutTimeStamp

		GetUnordered() bool
	}

	// HasValidateBasic defines a type that has a ValidateBasic method.
	// ValidateBasic is deprecated and now facultative.
	// Prefer validating messages directly in the msg server.
	HasValidateBasic interface {
		// ValidateBasic does a simple validation check that
		// doesn't require access to any other information.
		ValidateBasic() error
	}
)

// TxDecoder unmarshals transaction bytes
type TxDecoder func(txBytes []byte) (Tx, error)

// TxEncoder marshals transaction to bytes
type TxEncoder func(tx Tx) ([]byte, error)

// MsgTypeURL returns the TypeURL of a `sdk.Msg`.
var MsgTypeURL = codectypes.MsgTypeURL

// GetMsgFromTypeURL returns a `sdk.Msg` message type from a type URL
func GetMsgFromTypeURL(cdc codec.Codec, input string) (Msg, error) {
	var msg Msg
	bz, err := json.Marshal(struct {
		Type string `json:"@type"`
	}{
		Type: input,
	})
	if err != nil {
		return nil, err
	}

	if err := cdc.UnmarshalInterfaceJSON(bz, &msg); err != nil {
		return nil, fmt.Errorf("failed to determine sdk.Msg for %s URL : %w", input, err)
	}

	return msg, nil
}

// GetModuleNameFromTypeURL assumes that module name is the second element of the msg type URL
// e.g. "cosmos.bank.v1beta1.MsgSend" => "bank"
// It returns an empty string if the input is not a valid type URL
func GetModuleNameFromTypeURL(input string) string {
	moduleName := strings.Split(input, ".")
	if len(moduleName) > 1 {
		return moduleName[1]
	}

	return ""
}
