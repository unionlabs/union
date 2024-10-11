package codec_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	protov2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/dynamicpb"

	counterv1 "cosmossdk.io/api/cosmos/counter/v1"
	"cosmossdk.io/core/address"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	countertypes "github.com/cosmos/cosmos-sdk/testutil/x/counter/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var ac = codectestutil.CodecOptions{}.GetAddressCodec()

type msgCounterWrapper struct {
	*countertypes.MsgIncreaseCounter
	ac address.Codec
}

func (msg msgCounterWrapper) GetSigners() []sdk.AccAddress {
	fromAddress, _ := msg.ac.StringToBytes(msg.Signer)
	return []sdk.AccAddress{fromAddress}
}

func BenchmarkLegacyGetSigners(b *testing.B) {
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := ac.BytesToString(addr)
	require.NoError(b, err)
	msg := msgCounterWrapper{&countertypes.MsgIncreaseCounter{
		Signer: addrStr,
		Count:  2,
	}, ac}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = msg.GetSigners()
	}
}

func BenchmarkProtoreflectGetSigners(b *testing.B) {
	cdc := codectestutil.CodecOptions{}.NewCodec()
	signingCtx := cdc.InterfaceRegistry().SigningContext()
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := ac.BytesToString(addr)
	require.NoError(b, err)
	// use a pulsar message
	msg := &counterv1.MsgIncreaseCounter{
		Signer: addrStr,
		Count:  1,
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := signingCtx.GetSigners(msg)
		if err != nil {
			panic(err)
		}
	}
}

func BenchmarkProtoreflectGetSignersWithUnmarshal(b *testing.B) {
	cdc := codectestutil.CodecOptions{}.NewCodec()
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := ac.BytesToString(addr)
	require.NoError(b, err)
	// start with a protoreflect message
	msg := &countertypes.MsgIncreaseCounter{
		Signer: addrStr,
		Count:  1,
	}
	// marshal to an any first because this is what we get from the wire
	a, err := codectypes.NewAnyWithValue(msg)
	require.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, _, err := cdc.GetMsgAnySigners(a)
		if err != nil {
			panic(err)
		}
	}
}

func BenchmarkProtoreflectGetSignersDynamicpb(b *testing.B) {
	cdc := codectestutil.CodecOptions{}.NewCodec()
	signingCtx := cdc.InterfaceRegistry().SigningContext()
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := ac.BytesToString(addr)
	require.NoError(b, err)
	msg := &counterv1.MsgIncreaseCounter{
		Signer: addrStr,
		Count:  1,
	}
	bz, err := protov2.Marshal(msg)
	require.NoError(b, err)

	dynamicmsg := dynamicpb.NewMessage(msg.ProtoReflect().Descriptor())
	err = protov2.Unmarshal(bz, dynamicmsg)
	require.NoError(b, err)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := signingCtx.GetSigners(dynamicmsg)
		if err != nil {
			panic(err)
		}
	}
}
