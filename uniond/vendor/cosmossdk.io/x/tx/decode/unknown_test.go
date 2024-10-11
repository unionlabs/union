package decode_test

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/encoding/protowire"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/known/anypb"

	"cosmossdk.io/x/tx/decode"
	"cosmossdk.io/x/tx/internal/testpb"
)

func errUnknownField(typ string, tagNum int, wireType protowire.Type) error {
	var wt string
	if wireType >= 0 && wireType < 6 {
		wt = decode.WireTypeToString(wireType)
	}
	return decode.ErrUnknownField.Wrapf("%s: {TagNum: %d, WireType:%q}", typ, tagNum, wt)
}

var ProtoResolver = protoregistry.GlobalFiles

func TestRejectUnknownFieldsRepeated(t *testing.T) {
	tests := []struct {
		name                     string
		in                       proto.Message
		recv                     proto.Message
		wantErr                  error
		allowUnknownNonCriticals bool
		hasUnknownNonCriticals   bool
	}{
		{
			name: "Unknown field in midst of repeated values",
			in: &testpb.TestVersion2{
				C: []*testpb.TestVersion2{
					{
						C: []*testpb.TestVersion2{
							{
								Sum: &testpb.TestVersion2_F{
									F: &testpb.TestVersion2{
										A: &testpb.TestVersion2{
											B: &testpb.TestVersion2{
												H: []*testpb.TestVersion1{
													{
														X: 0x01,
													},
												},
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion2_F{
									F: &testpb.TestVersion2{
										A: &testpb.TestVersion2{
											B: &testpb.TestVersion2{
												H: []*testpb.TestVersion1{
													{
														X: 0x02,
													},
												},
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion2_F{
									F: &testpb.TestVersion2{
										NewField_: 411,
									},
								},
							},
						},
					},
				},
			},
			recv: new(testpb.TestVersion1),
			wantErr: errUnknownField(
				"testpb.TestVersion1",
				25,
				0),
		},
		{
			name:                     "Unknown field in midst of repeated values, allowUnknownNonCriticals set",
			allowUnknownNonCriticals: true,
			in: &testpb.TestVersion2{
				C: []*testpb.TestVersion2{
					{
						C: []*testpb.TestVersion2{
							{
								Sum: &testpb.TestVersion2_F{
									F: &testpb.TestVersion2{
										A: &testpb.TestVersion2{
											B: &testpb.TestVersion2{
												H: []*testpb.TestVersion1{
													{
														X: 0x01,
													},
												},
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion2_F{
									F: &testpb.TestVersion2{
										A: &testpb.TestVersion2{
											B: &testpb.TestVersion2{
												H: []*testpb.TestVersion1{
													{
														X: 0x02,
													},
												},
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion2_F{
									F: &testpb.TestVersion2{
										NewField_: 411,
									},
								},
							},
						},
					},
				},
			},
			recv: new(testpb.TestVersion1),
			wantErr: errUnknownField(
				"testpb.TestVersion1",
				25,
				0),
		},
		{
			name: "Unknown field in midst of repeated values, non-critical field to be rejected",
			in: &testpb.TestVersion3{
				C: []*testpb.TestVersion3{
					{
						C: []*testpb.TestVersion3{
							{
								Sum: &testpb.TestVersion3_F{
									F: &testpb.TestVersion3{
										A: &testpb.TestVersion3{
											B: &testpb.TestVersion3{
												X: 0x01,
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion3_F{
									F: &testpb.TestVersion3{
										A: &testpb.TestVersion3{
											B: &testpb.TestVersion3{
												X: 0x02,
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion3_F{
									F: &testpb.TestVersion3{
										NonCriticalField: "non-critical",
									},
								},
							},
						},
					},
				},
			},
			recv: new(testpb.TestVersion1),
			wantErr: errUnknownField(
				"testpb.TestVersion1",
				1031,
				2),
			hasUnknownNonCriticals: true,
		},
		{
			name:                     "Unknown field in midst of repeated values, non-critical field ignored",
			allowUnknownNonCriticals: true,
			in: &testpb.TestVersion3{
				C: []*testpb.TestVersion3{
					{
						C: []*testpb.TestVersion3{
							{
								Sum: &testpb.TestVersion3_F{
									F: &testpb.TestVersion3{
										A: &testpb.TestVersion3{
											B: &testpb.TestVersion3{
												X: 0x01,
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion3_F{
									F: &testpb.TestVersion3{
										A: &testpb.TestVersion3{
											B: &testpb.TestVersion3{
												X: 0x02,
											},
										},
									},
								},
							},
							{
								Sum: &testpb.TestVersion3_F{
									F: &testpb.TestVersion3{
										NonCriticalField: "non-critical",
									},
								},
							},
						},
					},
				},
			},
			recv:                   new(testpb.TestVersion1),
			wantErr:                nil,
			hasUnknownNonCriticals: true,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			protoBlob, err := proto.Marshal(tt.in)
			if err != nil {
				t.Fatal(err)
			}
			desc := tt.recv.ProtoReflect().Descriptor()
			hasUnknownNonCriticals, gotErr := decode.RejectUnknownFields(
				protoBlob, desc, tt.allowUnknownNonCriticals, ProtoResolver)
			if tt.wantErr != nil {
				require.EqualError(t, gotErr, tt.wantErr.Error())
			} else {
				require.NoError(t, gotErr)
			}
			require.Equal(t, tt.hasUnknownNonCriticals, hasUnknownNonCriticals)
		})
	}
}

func TestRejectUnknownFields_allowUnknownNonCriticals(t *testing.T) {
	tests := []struct {
		name                     string
		in                       proto.Message
		allowUnknownNonCriticals bool
		wantErr                  error
	}{
		{
			name: "Field that's in the reserved range, should fail by default",
			in: &testpb.Customer2{
				Id:       289,
				Reserved: 99,
			},
			wantErr: errUnknownField(
				"testpb.Customer1",
				1047,
				0),
		},
		{
			name:                     "Field that's in the reserved range, toggle allowUnknownNonCriticals",
			allowUnknownNonCriticals: true,
			in: &testpb.Customer2{
				Id:       289,
				Reserved: 99,
			},
			wantErr: nil,
		},
		{
			name:                     "Unknown fields that are critical, but with allowUnknownNonCriticals set",
			allowUnknownNonCriticals: true,
			in: &testpb.Customer2{
				Id:   289,
				City: testpb.Customer2_PaloAlto,
			},
			wantErr: errUnknownField(
				"testpb.Customer1",
				6,
				0),
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			blob, err := proto.Marshal(tt.in)
			if err != nil {
				t.Fatalf("Failed to marshal input: %v", err)
			}

			c1 := new(testpb.Customer1).ProtoReflect().Descriptor()
			_, gotErr := decode.RejectUnknownFields(blob, c1, tt.allowUnknownNonCriticals, ProtoResolver)
			if tt.wantErr != nil {
				require.EqualError(t, gotErr, tt.wantErr.Error())
			} else {
				require.NoError(t, gotErr)
			}
		})
	}
}

func TestRejectUnknownFieldsNested(t *testing.T) {
	tests := []struct {
		name    string
		in      proto.Message
		recv    proto.Message
		wantErr error
	}{
		{
			name: "TestVersion3 from TestVersionFD1",
			in: &testpb.TestVersion2{
				X: 5,
				Sum: &testpb.TestVersion2_E{
					E: 100,
				},
				H: []*testpb.TestVersion1{
					{X: 999},
					{X: -55},
					{
						X: 102,
						Sum: &testpb.TestVersion1_F{
							F: &testpb.TestVersion1{
								X: 4,
							},
						},
					},
				},
				K: &testpb.Customer1{
					Id:              45,
					Name:            "customer1",
					SubscriptionFee: 99,
				},
			},
			recv: new(testpb.TestVersionFD1),
			wantErr: errUnknownField(
				"testpb.TestVersionFD1",
				12,
				2),
		},
		{
			name: "Alternating oneofs",
			in: &testpb.TestVersion3{
				Sum: &testpb.TestVersion3_E{
					E: 99,
				},
			},
			recv:    new(testpb.TestVersion3LoneOneOfValue),
			wantErr: nil,
		},
		{
			name: "Alternating oneofs mismatched field",
			in: &testpb.TestVersion3{
				Sum: &testpb.TestVersion3_F{
					F: &testpb.TestVersion3{
						X: 99,
					},
				},
			},
			recv: new(testpb.TestVersion3LoneOneOfValue),
			wantErr: errUnknownField(
				"testpb.TestVersion3LoneOneOfValue",
				7,
				2),
		},
		{
			name: "Discrepancy in a deeply nested one of field",
			in: &testpb.TestVersion3{
				Sum: &testpb.TestVersion3_F{
					F: &testpb.TestVersion3{
						Sum: &testpb.TestVersion3_F{
							F: &testpb.TestVersion3{
								X: 19,
								Sum: &testpb.TestVersion3_E{
									E: 99,
								},
							},
						},
					},
				},
			},
			recv: new(testpb.TestVersion3LoneNesting),
			wantErr: errUnknownField(
				"testpb.TestVersion3LoneNesting",
				6,
				0),
		},
		{
			name: "unknown field types.Any in G",
			in: &testpb.TestVersion3{
				G: &anypb.Any{
					TypeUrl: "/testpb.TestVersion1",
					Value: mustMarshal(&testpb.TestVersion2{
						Sum: &testpb.TestVersion2_F{
							F: &testpb.TestVersion2{
								NewField_: 999,
							},
						},
					}),
				},
			},
			recv: new(testpb.TestVersion3),
			wantErr: errUnknownField(
				"testpb.TestVersion1",
				25, 0),
		},
		{
			name: "types.Any with extra fields",
			in: &testpb.TestVersionFD1WithExtraAny{
				G: &testpb.AnyWithExtra{
					A: &anypb.Any{
						TypeUrl: "/testpb.TestVersion1",
						Value: mustMarshal(&testpb.TestVersion2{
							Sum: &testpb.TestVersion2_F{
								F: &testpb.TestVersion2{
									NewField_: 999,
								},
							},
						}),
					},
					B: 3,
					C: 2,
				},
			},
			recv: new(testpb.TestVersion3),
			wantErr: errUnknownField(
				"google.protobuf.Any",
				3,
				0),
		},
		{
			name: "mismatched types.Any in G",
			in: &testpb.TestVersion1{
				G: &anypb.Any{
					TypeUrl: "/testpb.TestVersion4LoneNesting",
					Value: mustMarshal(&testpb.TestVersion3LoneNesting_Inner1{
						Inner: &testpb.TestVersion3LoneNesting_Inner1_InnerInner{
							Id:   "ID",
							City: "Gotham",
						},
					}),
				},
			},
			recv: new(testpb.TestVersion1),
			// behavior change from previous implementation: we allow mismatched wire -> proto types,
			// but this will still error on ConsumeFieldValue
			wantErr: errors.New("cannot parse reserved wire type"),
		},
		{
			name: "From nested proto message, message index 0",
			in: &testpb.TestVersion3LoneNesting{
				Inner1: &testpb.TestVersion3LoneNesting_Inner1{
					Id:   10,
					Name: "foo",
					Inner: &testpb.TestVersion3LoneNesting_Inner1_InnerInner{
						Id:   "ID",
						City: "Palo Alto",
					},
				},
			},
			recv:    new(testpb.TestVersion4LoneNesting),
			wantErr: nil,
		},
		{
			name: "From nested proto message, message index 1",
			in: &testpb.TestVersion3LoneNesting{
				Inner2: &testpb.TestVersion3LoneNesting_Inner2{
					Id:      "ID",
					Country: "Maldives",
					Inner: &testpb.TestVersion3LoneNesting_Inner2_InnerInner{
						Id:   "ID",
						City: "Unknown",
					},
				},
			},
			recv:    new(testpb.TestVersion4LoneNesting),
			wantErr: nil,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			protoBlob, err := proto.Marshal(tt.in)
			if err != nil {
				t.Fatal(err)
			}

			desc := tt.recv.ProtoReflect().Descriptor()
			gotErr := decode.RejectUnknownFieldsStrict(protoBlob, desc, ProtoResolver)
			if tt.wantErr != nil {
				require.ErrorContains(t, gotErr, tt.wantErr.Error())
			} else {
				require.NoError(t, gotErr)
			}
		})
	}
}

func TestRejectUnknownFieldsFlat(t *testing.T) {
	tests := []struct {
		name    string
		in      proto.Message
		wantErr error
	}{
		{
			name: "Oneof with same field number, shouldn't complain",
			in: &testpb.Customer3{
				Id:   68,
				Name: "ACME3",
				Payment: &testpb.Customer3_CreditCardNo{
					CreditCardNo: "123-XXXX-XXX881",
				},
			},
			wantErr: nil,
		},
		{
			name: "Oneof with different field number, should fail",
			in: &testpb.Customer3{
				Id:   68,
				Name: "ACME3",
				Payment: &testpb.Customer3_ChequeNo{
					ChequeNo: "123XXXXXXX881",
				},
			},
			wantErr: errUnknownField(
				"testpb.Customer1",
				8, 2),
		},
		{
			name: "Any in a field, the extra field will be serialized so should fail",
			in: &testpb.Customer2{
				Miscellaneous: &anypb.Any{},
			},
			wantErr: errUnknownField(
				"testpb.Customer1",
				10,
				2),
		},
		{
			name: "With a nested struct as a field",
			in: &testpb.Customer3{
				Id: 289,
				Original: &testpb.Customer1{
					Id: 991,
				},
			},
			wantErr: errUnknownField(
				"testpb.Customer1",
				9,
				2),
		},
		{
			name: "An extra field that's non-existent in Customer1",
			in: &testpb.Customer2{
				Id:       289,
				Name:     "Customer1",
				Industry: 5299,
				Fewer:    199.9,
			},
			wantErr: errUnknownField("testpb.Customer1", 4, 5),
		},
		{
			name: "Using a field that's in the reserved range, should fail by default",
			in: &testpb.Customer2{
				Id:       289,
				Reserved: 99,
			},
			wantErr: errUnknownField(
				"testpb.Customer1",
				1047,
				0),
		},
		{
			name: "Only fields matching",
			in: &testpb.Customer2{
				Id:   289,
				Name: "CustomerCustomerCustomerCustomerCustomer11111Customer1",
			},
			// behavior change from previous implementation: we allow mismatched wire -> proto types.
			// wantErr: errMismatchedField("testpb.Customer1", 4, 5),
		},
		{
			name: "Extra field that's non-existent in Customer1, along with Reserved set",
			in: &testpb.Customer2{
				Id:       289,
				Name:     "Customer1",
				Industry: 5299,
				Fewer:    199.9,
				Reserved: 819,
			},
			wantErr: errUnknownField("testpb.Customer1", 4, 5),
		},
		{
			name: "Using enumerated field",
			in: &testpb.Customer2{
				Id:       289,
				Name:     "Customer1",
				Industry: 5299,
				City:     testpb.Customer2_PaloAlto,
			},
			wantErr: errUnknownField("testpb.Customer1", 6, 0),
		},
		{
			name: "multiple extraneous fields",
			in: &testpb.Customer2{
				Id:       289,
				Name:     "Customer1",
				Industry: 5299,
				City:     testpb.Customer2_PaloAlto,
				Fewer:    45,
			},
			wantErr: errUnknownField("testpb.Customer1", 4, 5),
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			blob, err := proto.Marshal(tt.in)
			if err != nil {
				t.Fatalf("Failed to marshal input: %v", err)
			}

			c1 := new(testpb.Customer1)
			c1Desc := c1.ProtoReflect().Descriptor()
			gotErr := decode.RejectUnknownFieldsStrict(blob, c1Desc, ProtoResolver)
			if tt.wantErr != nil {
				require.EqualError(t, gotErr, tt.wantErr.Error())
			} else {
				require.NoError(t, gotErr)
			}
		})
	}
}

// Issue https://github.com/cosmos/cosmos-sdk/issues/7222, we need to ensure that repeated
// uint64 are recognized as packed.
func TestPackedEncoding(t *testing.T) {
	data := &testpb.TestRepeatedUints{Nums: []uint64{12, 13}}

	marshaled, err := proto.Marshal(data)
	require.NoError(t, err)

	unmarshalled := data.ProtoReflect().Descriptor()
	_, err = decode.RejectUnknownFields(marshaled, unmarshalled, false, ProtoResolver)
	require.NoError(t, err)
}

func mustMarshal(msg proto.Message) []byte {
	blob, err := proto.Marshal(msg)
	if err != nil {
		panic(err)
	}
	return blob
}
