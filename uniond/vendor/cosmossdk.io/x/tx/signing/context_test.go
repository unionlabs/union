package signing

import (
	"encoding/hex"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	groupv1 "cosmossdk.io/api/cosmos/group/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/x/tx/internal/testpb"
)

var deeplyNestedRepeatedSigner = &testpb.DeeplyNestedRepeatedSigner{
	Inner: []*testpb.DeeplyNestedRepeatedSigner_Inner{
		{
			Inner: []*testpb.DeeplyNestedRepeatedSigner_Inner_Inner{
				{
					Inner: []*testpb.DeeplyNestedRepeatedSigner_Inner_Inner_Bottom{
						{
							Signer: []string{hex.EncodeToString([]byte("foo")), hex.EncodeToString([]byte("bar"))},
						},
					},
				},
			},
		},
		{
			Inner: []*testpb.DeeplyNestedRepeatedSigner_Inner_Inner{
				{
					Inner: []*testpb.DeeplyNestedRepeatedSigner_Inner_Inner_Bottom{
						{
							Signer: []string{hex.EncodeToString([]byte("baz"))},
						},
					},
				},
				{
					Inner: []*testpb.DeeplyNestedRepeatedSigner_Inner_Inner_Bottom{
						{
							Signer: []string{hex.EncodeToString([]byte("qux")), hex.EncodeToString([]byte("fuz"))},
						},
						{
							Signer: []string{hex.EncodeToString([]byte("bing")), hex.EncodeToString([]byte("bap"))},
						},
					},
				},
			},
		},
	},
}

func TestGetGetSignersFnConcurrent(t *testing.T) {
	ctx, err := NewContext(Options{
		AddressCodec:          dummyAddressCodec{},
		ValidatorAddressCodec: dummyValidatorAddressCodec{},
	})
	require.NoError(t, err)

	desc := (&testpb.RepeatedSigner{}).ProtoReflect().Descriptor()
	for i := 0; i < 50; i++ {
		go func() {
			_, _ = ctx.getGetSignersFn(desc)
		}()
	}
}

func TestGetSigners(t *testing.T) {
	ctx, err := NewContext(Options{
		AddressCodec:          dummyAddressCodec{},
		ValidatorAddressCodec: dummyValidatorAddressCodec{},
	})
	require.NoError(t, err)
	tests := []struct {
		name    string
		msg     proto.Message
		want    [][]byte
		wantErr bool
	}{
		{
			name: "MsgSend",
			msg: &bankv1beta1.MsgSend{
				FromAddress: hex.EncodeToString([]byte("foo")),
			},
			want: [][]byte{[]byte("foo")},
		},
		{
			name: "MsgMultiSend",
			msg: &bankv1beta1.MsgMultiSend{
				Inputs: []*bankv1beta1.Input{
					{Address: hex.EncodeToString([]byte("foo"))},
					{Address: hex.EncodeToString([]byte("bar"))},
				},
			},
			want: [][]byte{[]byte("foo"), []byte("bar")},
		},
		{
			name: "MsgSubmitProposal",
			msg: &groupv1.MsgSubmitProposal{
				Proposers: []string{
					hex.EncodeToString([]byte("foo")),
					hex.EncodeToString([]byte("bar")),
				},
			},
			want: [][]byte{[]byte("foo"), []byte("bar")},
		},
		{
			name: "simple",
			msg:  &testpb.SimpleSigner{Signer: hex.EncodeToString([]byte("foo"))},
			want: [][]byte{[]byte("foo")},
		},
		{
			name: "repeated",
			msg: &testpb.RepeatedSigner{Signer: []string{
				hex.EncodeToString([]byte("foo")),
				hex.EncodeToString([]byte("bar")),
			}},
			want: [][]byte{[]byte("foo"), []byte("bar")},
		},
		{
			name: "nested",
			msg:  &testpb.NestedSigner{Inner: &testpb.NestedSigner_Inner{Signer: hex.EncodeToString([]byte("foo"))}},
			want: [][]byte{[]byte("foo")},
		},
		{
			name: "nested repeated",
			msg: &testpb.NestedRepeatedSigner{Inner: &testpb.NestedRepeatedSigner_Inner{Signer: []string{
				hex.EncodeToString([]byte("foo")),
				hex.EncodeToString([]byte("bar")),
			}}},
			want: [][]byte{[]byte("foo"), []byte("bar")},
		},
		{
			name: "repeated nested",
			msg: &testpb.RepeatedNestedSigner{Inner: []*testpb.RepeatedNestedSigner_Inner{
				{Signer: hex.EncodeToString([]byte("foo"))},
				{Signer: hex.EncodeToString([]byte("bar"))},
			}},
			want: [][]byte{[]byte("foo"), []byte("bar")},
		},
		{
			name: "deeply nested",
			msg: &testpb.DeeplyNestedSigner{
				InnerOne: &testpb.DeeplyNestedSigner_InnerOne{
					InnerTwo: &testpb.DeeplyNestedSigner_InnerOne_InnerTwo{
						Signer: hex.EncodeToString([]byte("foo")),
					},
				},
			},
			want: [][]byte{[]byte("foo")},
		},
		{
			name: "nested repeated #1",
			msg: &testpb.NestedRepeatedSigner{Inner: &testpb.NestedRepeatedSigner_Inner{
				Signer: []string{
					hex.EncodeToString([]byte("foo")),
					hex.EncodeToString([]byte("bar")),
				},
			}},
			want: [][]byte{[]byte("foo"), []byte("bar")},
		},
		{
			name: "nested repeated #2",
			msg:  deeplyNestedRepeatedSigner,
			want: [][]byte{[]byte("foo"), []byte("bar"), []byte("baz"), []byte("qux"), []byte("fuz"), []byte("bing"), []byte("bap")},
		},
		{
			name: "repeated nested repeated",
			msg: &testpb.RepeatedNestedRepeatedSigner{Inner: []*testpb.RepeatedNestedRepeatedSigner_Inner{
				{Signer: []string{
					hex.EncodeToString([]byte("foo")),
					hex.EncodeToString([]byte("bar")),
				}},
				{Signer: []string{
					hex.EncodeToString([]byte("baz")),
					hex.EncodeToString([]byte("bam")),
				}},
				{Signer: []string{
					hex.EncodeToString([]byte("blah")),
				}},
			}},
			want: [][]byte{[]byte("foo"), []byte("bar"), []byte("baz"), []byte("bam"), []byte("blah")},
		},
		{
			name:    "bad",
			msg:     &testpb.BadSigner{},
			wantErr: true,
		},
		{
			name:    "no signer",
			msg:     &testpb.NoSignerOption{},
			wantErr: true,
		},
		{
			name: "validator signer",
			msg: &testpb.ValidatorSigner{
				Signer: "val" + hex.EncodeToString([]byte("foo")),
			},
			want: [][]byte{[]byte("foo")},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			signers, err := ctx.GetSigners(test.msg)
			if test.wantErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
			require.Equal(t, test.want, signers)
		})
	}
}

func TestMaxRecursionDepth(t *testing.T) {
	ctx, err := NewContext(Options{
		AddressCodec:          dummyAddressCodec{},
		ValidatorAddressCodec: dummyValidatorAddressCodec{},
		MaxRecursionDepth:     1,
	})
	require.NoError(t, err)

	_, err = ctx.GetSigners(deeplyNestedRepeatedSigner)
	require.ErrorContains(t, err, "maximum recursion depth exceeded")

	ctx, err = NewContext(Options{
		AddressCodec:          dummyAddressCodec{},
		ValidatorAddressCodec: dummyValidatorAddressCodec{},
		MaxRecursionDepth:     5,
	})
	require.NoError(t, err)
	_, err = ctx.GetSigners(deeplyNestedRepeatedSigner)
	require.NoError(t, err)
}

func TestDefineCustomGetSigners(t *testing.T) {
	customMsg := &testpb.Ballot{}
	signers := [][]byte{[]byte("foo")}
	options := Options{
		AddressCodec:          dummyAddressCodec{},
		ValidatorAddressCodec: dummyValidatorAddressCodec{},
	}
	context, err := NewContext(options)
	require.NoError(t, err)

	_, err = context.GetSigners(customMsg)
	// without a custom signer we should get an error
	require.ErrorContains(t, err, "use DefineCustomGetSigners to specify")

	// create a new context with a custom signer
	options.DefineCustomGetSigners(proto.MessageName(customMsg), func(msg proto.Message) ([][]byte, error) {
		return signers, nil
	})
	context, err = NewContext(options)
	require.NoError(t, err)
	gotSigners, err := context.GetSigners(customMsg)
	// now that a custom signer has been defined, we should get no error and the expected result
	require.NoError(t, err)
	require.Equal(t, signers, gotSigners)

	// test that registering a custom signer for a message that already has proto annotation defined signer
	// fails validation
	simpleSigner := &testpb.SimpleSigner{Signer: hex.EncodeToString([]byte("foo"))}
	options.DefineCustomGetSigners(proto.MessageName(simpleSigner), func(msg proto.Message) ([][]byte, error) {
		return [][]byte{[]byte("qux")}, nil
	})
	context, err = NewContext(options)
	require.NoError(t, err)
	require.ErrorContains(t, context.Validate(), "a custom signer function as been defined for message SimpleSigner")
}

type dummyAddressCodec struct{}

func (d dummyAddressCodec) StringToBytes(text string) ([]byte, error) {
	return hex.DecodeString(text)
}

func (d dummyAddressCodec) BytesToString(bz []byte) (string, error) {
	return hex.EncodeToString(bz), nil
}

var _ address.Codec = dummyAddressCodec{}

type dummyValidatorAddressCodec struct{}

func (d dummyValidatorAddressCodec) StringToBytes(text string) ([]byte, error) {
	return hex.DecodeString(strings.TrimPrefix(text, "val"))
}

func (d dummyValidatorAddressCodec) BytesToString(bz []byte) (string, error) {
	return "val" + hex.EncodeToString(bz), nil
}

var _ address.Codec = dummyValidatorAddressCodec{}
