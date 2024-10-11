package mock

import (
	"bytes"
	"fmt"

	"google.golang.org/protobuf/reflect/protoreflect"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	"cosmossdk.io/core/transaction"
	errorsmod "cosmossdk.io/errors"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	txsigning "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// KVStoreTx is an sdk.Tx which is its own sdk.Msg.
type KVStoreTx struct {
	key     []byte
	value   []byte
	bytes   []byte
	address sdk.AccAddress
}

// testPubKey is a dummy implementation of PubKey used for testing.
type testPubKey struct {
	address sdk.AccAddress
}

func (t testPubKey) Reset() { panic("not implemented") }

func (t testPubKey) String() string { panic("not implemented") }

func (t testPubKey) ProtoMessage() { panic("not implemented") }

func (t testPubKey) Address() cryptotypes.Address { return t.address.Bytes() }

func (t testPubKey) Bytes() []byte { panic("not implemented") }

func (t testPubKey) VerifySignature(msg, sig []byte) bool { panic("not implemented") }

func (t testPubKey) Equals(key cryptotypes.PubKey) bool { panic("not implemented") }

func (t testPubKey) Type() string { panic("not implemented") }

func (msg *KVStoreTx) GetSignaturesV2() (res []txsigning.SignatureV2, err error) {
	res = append(res, txsigning.SignatureV2{
		PubKey:   testPubKey{address: msg.address},
		Data:     nil,
		Sequence: 1,
	})

	return res, nil
}

func (msg *KVStoreTx) VerifySignature(msgByte, sig []byte) bool {
	panic("implement me")
}

func (msg *KVStoreTx) Address() cryptotypes.Address {
	panic("implement me")
}

func (msg *KVStoreTx) Bytes() []byte {
	panic("implement me")
}

func (msg *KVStoreTx) Equals(key cryptotypes.PubKey) bool {
	panic("implement me")
}

// dummy implementation of proto.Message

func (msg *KVStoreTx) Reset()         {}
func (msg *KVStoreTx) String() string { return "TODO" }
func (msg *KVStoreTx) ProtoMessage()  {}

var (
	_ sdk.Tx                  = &KVStoreTx{}
	_ sdk.Msg                 = &KVStoreTx{}
	_ signing.SigVerifiableTx = &KVStoreTx{}
	_ cryptotypes.PubKey      = &KVStoreTx{}
	_ cryptotypes.PubKey      = &testPubKey{}
)

func NewTx(key, value string, accAddress sdk.AccAddress) *KVStoreTx {
	bytes := fmt.Sprintf("%s=%s=%s", key, value, accAddress)
	return &KVStoreTx{
		key:     []byte(key),
		value:   []byte(value),
		bytes:   []byte(bytes),
		address: accAddress,
	}
}

func (msg *KVStoreTx) Hash() [32]byte {
	return [32]byte{}
}

func (msg *KVStoreTx) GetGasLimit() (uint64, error) {
	return 0, nil
}

func (msg *KVStoreTx) GetMessages() ([]transaction.Msg, error) {
	return nil, nil
}

func (msg *KVStoreTx) GetSenders() ([][]byte, error) {
	return nil, nil
}

func (msg *KVStoreTx) Type() string {
	return "kvstore_tx"
}

func (msg *KVStoreTx) GetMsgs() []sdk.Msg {
	return []sdk.Msg{msg}
}

func (msg *KVStoreTx) GetReflectMessages() ([]protoreflect.Message, error) {
	return []protoreflect.Message{(&bankv1beta1.MsgSend{FromAddress: msg.address.String()}).ProtoReflect()}, nil // this is a hack for tests
}

func (msg *KVStoreTx) GetSignBytes() []byte {
	return msg.bytes
}

// ValidateBasic should the app be calling this? or only handlers?
func (msg *KVStoreTx) ValidateBasic() error {
	return nil
}

func (msg *KVStoreTx) GetSigners() ([][]byte, error) {
	return nil, nil
}

func (msg *KVStoreTx) GetPubKeys() ([]cryptotypes.PubKey, error) { panic("GetPubKeys not implemented") }

// takes raw transaction bytes and decodes them into an sdk.Tx. An sdk.Tx has
// all the signatures and can be used to authenticate.
func decodeTx(txBytes []byte) (sdk.Tx, error) {
	var tx sdk.Tx

	split := bytes.Split(txBytes, []byte("="))
	switch len(split) {
	case 1:
		k := split[0]
		tx = &KVStoreTx{k, k, txBytes, nil}
	case 2:
		k, v := split[0], split[1]
		tx = &KVStoreTx{k, v, txBytes, nil}
	case 3:
		k, v, addr := split[0], split[1], split[2]
		tx = &KVStoreTx{k, v, txBytes, addr}
	default:
		return nil, errorsmod.Wrap(sdkerrors.ErrTxDecode, "too many '='")
	}

	return tx, nil
}
