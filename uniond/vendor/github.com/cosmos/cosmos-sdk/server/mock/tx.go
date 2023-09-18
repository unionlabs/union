package mock

import (
	"bytes"
	"fmt"

	"github.com/cosmos/cosmos-sdk/x/auth/signing"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	txsigning "github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// An sdk.Tx which is its own sdk.Msg.
type kvstoreTx struct {
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

func (t testPubKey) VerifySignature(msg []byte, sig []byte) bool { panic("not implemented") }

func (t testPubKey) Equals(key cryptotypes.PubKey) bool { panic("not implemented") }

func (t testPubKey) Type() string { panic("not implemented") }

func (msg *kvstoreTx) GetSignaturesV2() (res []txsigning.SignatureV2, err error) {
	res = append(res, txsigning.SignatureV2{
		PubKey:   testPubKey{address: msg.address},
		Data:     nil,
		Sequence: 1,
	})

	return res, nil
}

func (msg *kvstoreTx) VerifySignature(msgByte []byte, sig []byte) bool {
	panic("implement me")
}

func (msg *kvstoreTx) Address() cryptotypes.Address {
	panic("implement me")
}

func (msg *kvstoreTx) Bytes() []byte {
	panic("implement me")
}

func (msg *kvstoreTx) Equals(key cryptotypes.PubKey) bool {
	panic("implement me")
}

// dummy implementation of proto.Message
func (msg *kvstoreTx) Reset()         {}
func (msg *kvstoreTx) String() string { return "TODO" }
func (msg *kvstoreTx) ProtoMessage()  {}

var (
	_ sdk.Tx                  = &kvstoreTx{}
	_ sdk.Msg                 = &kvstoreTx{}
	_ signing.SigVerifiableTx = &kvstoreTx{}
	_ cryptotypes.PubKey      = &kvstoreTx{}
	_ cryptotypes.PubKey      = &testPubKey{}
)

func NewTx(key, value string, accAddress sdk.AccAddress) *kvstoreTx {
	bytes := fmt.Sprintf("%s=%s", key, value)
	return &kvstoreTx{
		key:     []byte(key),
		value:   []byte(value),
		bytes:   []byte(bytes),
		address: accAddress,
	}
}

func (tx *kvstoreTx) Type() string {
	return "kvstore_tx"
}

func (tx *kvstoreTx) GetMsgs() []sdk.Msg {
	return []sdk.Msg{tx}
}

func (tx *kvstoreTx) GetSignBytes() []byte {
	return tx.bytes
}

// Should the app be calling this? Or only handlers?
func (tx *kvstoreTx) ValidateBasic() error {
	return nil
}

func (tx *kvstoreTx) GetSigners() []sdk.AccAddress {
	return nil
}

func (tx *kvstoreTx) GetPubKeys() ([]cryptotypes.PubKey, error) { panic("GetPubKeys not implemented") }

// takes raw transaction bytes and decodes them into an sdk.Tx. An sdk.Tx has
// all the signatures and can be used to authenticate.
func decodeTx(txBytes []byte) (sdk.Tx, error) {
	var tx sdk.Tx

	split := bytes.Split(txBytes, []byte("="))
	if len(split) == 1 { //nolint:gocritic
		k := split[0]
		tx = &kvstoreTx{k, k, txBytes, nil}
	} else if len(split) == 2 {
		k, v := split[0], split[1]
		tx = &kvstoreTx{k, v, txBytes, nil}
	} else {
		return nil, sdkerrors.Wrap(sdkerrors.ErrTxDecode, "too many '='")
	}

	return tx, nil
}
