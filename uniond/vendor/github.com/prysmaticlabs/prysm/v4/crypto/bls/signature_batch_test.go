package bls

import (
	"bytes"
	"fmt"
	"reflect"
	"sort"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

const TestSignature = "test signature"

func TestCopySignatureSet(t *testing.T) {
	t.Run("blst", func(t *testing.T) {
		key, err := RandKey()
		assert.NoError(t, err)
		key2, err := RandKey()
		assert.NoError(t, err)
		key3, err := RandKey()
		assert.NoError(t, err)

		message := [32]byte{'C', 'D'}
		message2 := [32]byte{'E', 'F'}
		message3 := [32]byte{'H', 'I'}

		sig := key.Sign(message[:])
		sig2 := key2.Sign(message2[:])
		sig3 := key3.Sign(message3[:])

		set := &SignatureBatch{
			Signatures:   [][]byte{sig.Marshal()},
			PublicKeys:   []PublicKey{key.PublicKey()},
			Messages:     [][32]byte{message},
			Descriptions: createDescriptions(1),
		}
		set2 := &SignatureBatch{
			Signatures:   [][]byte{sig2.Marshal()},
			PublicKeys:   []PublicKey{key.PublicKey()},
			Messages:     [][32]byte{message},
			Descriptions: createDescriptions(1),
		}
		set3 := &SignatureBatch{
			Signatures:   [][]byte{sig3.Marshal()},
			PublicKeys:   []PublicKey{key.PublicKey()},
			Messages:     [][32]byte{message},
			Descriptions: createDescriptions(1),
		}
		aggSet := set.Join(set2).Join(set3)
		aggSet2 := aggSet.Copy()

		assert.DeepEqual(t, aggSet, aggSet2)
	})
}

func TestVerifyVerbosely_AllSignaturesValid(t *testing.T) {
	set := NewValidSignatureSet(t, "good", 3)
	valid, err := set.VerifyVerbosely()
	assert.NoError(t, err)
	assert.Equal(t, true, valid, "SignatureSet is expected to be valid")
}

func TestVerifyVerbosely_SomeSignaturesInvalid(t *testing.T) {
	goodSet := NewValidSignatureSet(t, "good", 3)
	badSet := NewInvalidSignatureSet(t, "bad", 3, false)
	set := NewSet().Join(goodSet).Join(badSet)
	valid, err := set.VerifyVerbosely()
	assert.Equal(t, false, valid, "SignatureSet is expected to be invalid")
	assert.StringContains(t, "signature 'signature of bad0' is invalid", err.Error())
	assert.StringContains(t, "signature 'signature of bad1' is invalid", err.Error())
	assert.StringContains(t, "signature 'signature of bad2' is invalid", err.Error())
	assert.StringNotContains(t, "signature 'signature of good0' is invalid", err.Error())
	assert.StringNotContains(t, "signature 'signature of good1' is invalid", err.Error())
	assert.StringNotContains(t, "signature 'signature of good2' is invalid", err.Error())
}

func TestVerifyVerbosely_VerificationThrowsError(t *testing.T) {
	goodSet := NewValidSignatureSet(t, "good", 1)
	badSet := NewInvalidSignatureSet(t, "bad", 1, true)
	set := NewSet().Join(goodSet).Join(badSet)
	valid, err := set.VerifyVerbosely()
	assert.Equal(t, false, valid, "SignatureSet is expected to be invalid")
	assert.StringContains(t, "signature 'signature of bad0' is invalid", err.Error())
	assert.StringContains(t, "could not unmarshal bytes into signature", err.Error())
	assert.StringNotContains(t, "signature 'signature of good0' is invalid", err.Error())
}

func TestSignatureBatch_RemoveDuplicates(t *testing.T) {
	var keys []SecretKey
	for i := 0; i < 100; i++ {
		key, err := RandKey()
		assert.NoError(t, err)
		keys = append(keys, key)
	}
	tests := []struct {
		name         string
		batchCreator func() (input *SignatureBatch, output *SignatureBatch)
		want         int
	}{
		{
			name: "empty batch",
			batchCreator: func() (*SignatureBatch, *SignatureBatch) {
				return &SignatureBatch{}, &SignatureBatch{}
			},
			want: 0,
		},
		{
			name: "valid duplicates in batch",
			batchCreator: func() (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:20]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				allSigs := append(signatures, signatures...)
				allPubs := append(pubs, pubs...)
				allMsgs := append(messages, messages...)
				return &SignatureBatch{
						Signatures:   allSigs,
						PublicKeys:   allPubs,
						Messages:     allMsgs,
						Descriptions: createDescriptions(len(allMsgs)),
					}, &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(allMsgs)),
					}
			},
			want: 20,
		},
		{
			name: "valid duplicates in batch with multiple messages",
			batchCreator: func() (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:30]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				msg1 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '1'}
				msg2 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '2'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys[:10] {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[10:20] {
					s := k.Sign(msg1[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg1)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[20:30] {
					s := k.Sign(msg2[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg2)
					pubs = append(pubs, k.PublicKey())
				}
				allSigs := append(signatures, signatures...)
				allPubs := append(pubs, pubs...)
				allMsgs := append(messages, messages...)
				return &SignatureBatch{
						Signatures:   allSigs,
						PublicKeys:   allPubs,
						Messages:     allMsgs,
						Descriptions: createDescriptions(len(allMsgs)),
					}, &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(allMsgs)),
					}
			},
			want: 30,
		},
		{
			name: "no duplicates in batch with multiple messages",
			batchCreator: func() (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:30]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				msg1 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '1'}
				msg2 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '2'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys[:10] {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[10:20] {
					s := k.Sign(msg1[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg1)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[20:30] {
					s := k.Sign(msg2[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg2)
					pubs = append(pubs, k.PublicKey())
				}
				return &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(messages)),
					}, &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(messages)),
					}
			},
			want: 0,
		},
		{
			name: "valid duplicates and invalid duplicates in batch with multiple messages",
			batchCreator: func() (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:30]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				msg1 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '1'}
				msg2 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '2'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys[:10] {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[10:20] {
					s := k.Sign(msg1[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg1)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[20:30] {
					s := k.Sign(msg2[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg2)
					pubs = append(pubs, k.PublicKey())
				}
				allSigs := append(signatures, signatures...)
				// Make it a non-unique entry
				allSigs[10] = make([]byte, 96)
				allPubs := append(pubs, pubs...)
				allMsgs := append(messages, messages...)
				// Insert it back at the end
				signatures = append(signatures, signatures[10])
				pubs = append(pubs, pubs[10])
				messages = append(messages, messages[10])
				// Zero out to expected result
				signatures[10] = make([]byte, 96)
				return &SignatureBatch{
						Signatures:   allSigs,
						PublicKeys:   allPubs,
						Messages:     allMsgs,
						Descriptions: createDescriptions(len(allMsgs)),
					}, &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(allMsgs)),
					}
			},
			want: 29,
		},
		{
			name: "valid duplicates and invalid duplicates with signature,pubkey,message in batch with multiple messages",
			batchCreator: func() (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:30]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				msg1 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '1'}
				msg2 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '2'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys[:10] {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[10:20] {
					s := k.Sign(msg1[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg1)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[20:30] {
					s := k.Sign(msg2[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg2)
					pubs = append(pubs, k.PublicKey())
				}
				allSigs := append(signatures, signatures...)
				// Make it a non-unique entry
				allSigs[10] = make([]byte, 96)

				allPubs := append(pubs, pubs...)
				allPubs[20] = keys[len(keys)-1].PublicKey()

				allMsgs := append(messages, messages...)
				allMsgs[29] = [32]byte{'j', 'u', 'n', 'k'}

				// Insert it back at the end
				signatures = append(signatures, signatures[10])
				pubs = append(pubs, pubs[10])
				messages = append(messages, messages[10])
				// Zero out to expected result
				signatures[10] = make([]byte, 96)

				// Insert it back at the end
				signatures = append(signatures, signatures[20])
				pubs = append(pubs, pubs[20])
				messages = append(messages, messages[20])
				// Zero out to expected result
				pubs[20] = keys[len(keys)-1].PublicKey()

				// Insert it back at the end
				signatures = append(signatures, signatures[29])
				pubs = append(pubs, pubs[29])
				messages = append(messages, messages[29])
				messages[29] = [32]byte{'j', 'u', 'n', 'k'}

				return &SignatureBatch{
						Signatures:   allSigs,
						PublicKeys:   allPubs,
						Messages:     allMsgs,
						Descriptions: createDescriptions(len(allMsgs)),
					}, &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(messages)),
					}
			},
			want: 27,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input, output := tt.batchCreator()
			num, res, err := input.RemoveDuplicates()
			assert.NoError(t, err)
			if num != tt.want {
				t.Errorf("RemoveDuplicates() got = %v, want %v", num, tt.want)
			}
			if !reflect.DeepEqual(res.Signatures, output.Signatures) {
				t.Errorf("RemoveDuplicates() Signatures output = %v, want %v", res.Signatures, output.Signatures)
			}
			if !reflect.DeepEqual(res.PublicKeys, output.PublicKeys) {
				t.Errorf("RemoveDuplicates() Publickeys output = %v, want %v", res.PublicKeys, output.PublicKeys)
			}
			if !reflect.DeepEqual(res.Messages, output.Messages) {
				t.Errorf("RemoveDuplicates() Messages output = %v, want %v", res.Messages, output.Messages)
			}
		})
	}
}

func TestSignatureBatch_AggregateBatch(t *testing.T) {
	var keys []SecretKey
	for i := 0; i < 100; i++ {
		key, err := RandKey()
		assert.NoError(t, err)
		keys = append(keys, key)
	}
	tests := []struct {
		name         string
		batchCreator func(t *testing.T) (input *SignatureBatch, output *SignatureBatch)
		wantErr      bool
	}{
		{
			name: "empty batch",
			batchCreator: func(t *testing.T) (*SignatureBatch, *SignatureBatch) {
				return &SignatureBatch{Signatures: nil, Messages: nil, PublicKeys: nil, Descriptions: nil},
					&SignatureBatch{Signatures: nil, Messages: nil, PublicKeys: nil, Descriptions: nil}
			},
			wantErr: false,
		},
		{
			name: "mismatch number of signatures and messages in batch",
			batchCreator: func(t *testing.T) (*SignatureBatch, *SignatureBatch) {
				key1 := keys[0]
				key2 := keys[1]
				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				sig1 := key1.Sign(msg[:])
				sig2 := key2.Sign(msg[:])
				signatures := [][]byte{sig1.Marshal(), sig2.Marshal()}
				pubs := []common.PublicKey{key1.PublicKey(), key2.PublicKey()}
				messages := [][32]byte{msg}
				descs := createDescriptions(2)
				return &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: descs,
					}, &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: descs,
					}
			},
			wantErr: true,
		},
		{
			name: "valid signatures in batch",
			batchCreator: func(t *testing.T) (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:20]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				aggSig, err := AggregateCompressedSignatures(signatures)
				assert.NoError(t, err)
				aggPub := AggregateMultiplePubkeys(pubs)
				return &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(messages)),
					}, &SignatureBatch{
						Signatures:   [][]byte{aggSig.Marshal()},
						PublicKeys:   []PublicKey{aggPub},
						Messages:     [][32]byte{msg},
						Descriptions: createDescriptions(1, AggregatedSignature),
					}
			},
			wantErr: false,
		},
		{
			name: "invalid signatures in batch",
			batchCreator: func(t *testing.T) (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:20]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				signatures[10] = make([]byte, 96)
				return &SignatureBatch{
					Signatures:   signatures,
					PublicKeys:   pubs,
					Messages:     messages,
					Descriptions: createDescriptions(len(messages)),
				}, nil
			},
			wantErr: true,
		},
		{
			name: "valid aggregates in batch with multiple messages",
			batchCreator: func(t *testing.T) (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:30]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				msg1 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '1'}
				msg2 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '2'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys[:10] {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[10:20] {
					s := k.Sign(msg1[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg1)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[20:30] {
					s := k.Sign(msg2[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg2)
					pubs = append(pubs, k.PublicKey())
				}
				aggSig1, err := AggregateCompressedSignatures(signatures[:10])
				assert.NoError(t, err)
				aggSig2, err := AggregateCompressedSignatures(signatures[10:20])
				assert.NoError(t, err)
				aggSig3, err := AggregateCompressedSignatures(signatures[20:30])
				assert.NoError(t, err)
				aggPub1 := AggregateMultiplePubkeys(pubs[:10])
				aggPub2 := AggregateMultiplePubkeys(pubs[10:20])
				aggPub3 := AggregateMultiplePubkeys(pubs[20:30])
				return &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(messages)),
					}, &SignatureBatch{
						Signatures:   [][]byte{aggSig1.Marshal(), aggSig2.Marshal(), aggSig3.Marshal()},
						PublicKeys:   []PublicKey{aggPub1, aggPub2, aggPub3},
						Messages:     [][32]byte{msg, msg1, msg2},
						Descriptions: createDescriptions(3, AggregatedSignature),
					}
			},
			wantErr: false,
		},
		{
			name: "common and uncommon messages in batch with multiple messages",
			batchCreator: func(t *testing.T) (*SignatureBatch, *SignatureBatch) {
				chosenKeys := keys[:30]

				msg := [32]byte{'r', 'a', 'n', 'd', 'o', 'm'}
				msg1 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '1'}
				msg2 := [32]byte{'r', 'a', 'n', 'd', 'o', 'm', '2'}
				var signatures [][]byte
				var messages [][32]byte
				var pubs []PublicKey
				for _, k := range chosenKeys[:10] {
					s := k.Sign(msg[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[10:20] {
					s := k.Sign(msg1[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg1)
					pubs = append(pubs, k.PublicKey())
				}
				for _, k := range chosenKeys[20:30] {
					s := k.Sign(msg2[:])
					signatures = append(signatures, s.Marshal())
					messages = append(messages, msg2)
					pubs = append(pubs, k.PublicKey())
				}
				// Set a custom message
				messages[5][31] ^= byte(100)
				messages[15][31] ^= byte(100)
				messages[25][31] ^= byte(100)

				var newSigs [][]byte
				newSigs = append(newSigs, signatures[:5]...)
				newSigs = append(newSigs, signatures[6:10]...)

				aggSig1, err := AggregateCompressedSignatures(newSigs)
				assert.NoError(t, err)

				newSigs = [][]byte{}
				newSigs = append(newSigs, signatures[10:15]...)
				newSigs = append(newSigs, signatures[16:20]...)
				aggSig2, err := AggregateCompressedSignatures(newSigs)
				assert.NoError(t, err)

				newSigs = [][]byte{}
				newSigs = append(newSigs, signatures[20:25]...)
				newSigs = append(newSigs, signatures[26:30]...)
				aggSig3, err := AggregateCompressedSignatures(newSigs)
				assert.NoError(t, err)

				var newPubs []PublicKey
				newPubs = append(newPubs, pubs[:5]...)
				newPubs = append(newPubs, pubs[6:10]...)

				aggPub1 := AggregateMultiplePubkeys(newPubs)

				newPubs = []PublicKey{}
				newPubs = append(newPubs, pubs[10:15]...)
				newPubs = append(newPubs, pubs[16:20]...)
				aggPub2 := AggregateMultiplePubkeys(newPubs)

				newPubs = []PublicKey{}
				newPubs = append(newPubs, pubs[20:25]...)
				newPubs = append(newPubs, pubs[26:30]...)
				aggPub3 := AggregateMultiplePubkeys(newPubs)

				return &SignatureBatch{
						Signatures:   signatures,
						PublicKeys:   pubs,
						Messages:     messages,
						Descriptions: createDescriptions(len(messages)),
					}, &SignatureBatch{
						Signatures:   [][]byte{aggSig1.Marshal(), signatures[5], aggSig2.Marshal(), signatures[15], aggSig3.Marshal(), signatures[25]},
						PublicKeys:   []PublicKey{aggPub1, pubs[5], aggPub2, pubs[15], aggPub3, pubs[25]},
						Messages:     [][32]byte{msg, messages[5], msg1, messages[15], msg2, messages[25]},
						Descriptions: []string{AggregatedSignature, TestSignature, AggregatedSignature, TestSignature, AggregatedSignature, TestSignature},
					}
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			input, output := tt.batchCreator(t)
			got, err := input.AggregateBatch()
			if (err != nil) != tt.wantErr {
				t.Errorf("AggregateBatch() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if tt.wantErr {
				return
			}
			got = sortSet(got)
			output = sortSet(output)

			if !reflect.DeepEqual(got.Signatures, output.Signatures) {
				t.Errorf("AggregateBatch() Signatures got = %v, want %v", got.Signatures, output.Signatures)
			}
			if !reflect.DeepEqual(got.PublicKeys, output.PublicKeys) {
				t.Errorf("AggregateBatch() PublicKeys got = %v, want %v", got.PublicKeys, output.PublicKeys)
			}
			if !reflect.DeepEqual(got.Messages, output.Messages) {
				t.Errorf("AggregateBatch() Messages got = %v, want %v", got.Messages, output.Messages)
			}
			if !reflect.DeepEqual(got.Descriptions, output.Descriptions) {
				t.Errorf("AggregateBatch() Descriptions got = %v, want %v", got.Descriptions, output.Descriptions)
			}
		})
	}
}

func NewValidSignatureSet(t *testing.T, msgBody string, num int) *SignatureBatch {
	set := &SignatureBatch{
		Signatures:   make([][]byte, num),
		PublicKeys:   make([]common.PublicKey, num),
		Messages:     make([][32]byte, num),
		Descriptions: make([]string, num),
	}

	for i := 0; i < num; i++ {
		priv, err := RandKey()
		require.NoError(t, err)
		pubkey := priv.PublicKey()
		msg := messageBytes(fmt.Sprintf("%s%d", msgBody, i))
		sig := priv.Sign(msg[:]).Marshal()
		desc := fmt.Sprintf("signature of %s%d", msgBody, i)

		set.Signatures[i] = sig
		set.PublicKeys[i] = pubkey
		set.Messages[i] = msg
		set.Descriptions[i] = desc
	}

	return set
}

func NewInvalidSignatureSet(t *testing.T, msgBody string, num int, throwErr bool) *SignatureBatch {
	set := &SignatureBatch{
		Signatures:   make([][]byte, num),
		PublicKeys:   make([]common.PublicKey, num),
		Messages:     make([][32]byte, num),
		Descriptions: make([]string, num),
	}

	for i := 0; i < num; i++ {
		priv, err := RandKey()
		require.NoError(t, err)
		pubkey := priv.PublicKey()
		msg := messageBytes(fmt.Sprintf("%s%d", msgBody, i))
		var sig []byte
		if throwErr {
			sig = make([]byte, 96)
		} else {
			badMsg := messageBytes("badmsg")
			sig = priv.Sign(badMsg[:]).Marshal()
		}
		desc := fmt.Sprintf("signature of %s%d", msgBody, i)

		set.Signatures[i] = sig
		set.PublicKeys[i] = pubkey
		set.Messages[i] = msg
		set.Descriptions[i] = desc
	}

	return set
}

func messageBytes(message string) [32]byte {
	var bytes [32]byte
	copy(bytes[:], message)
	return bytes
}

func createDescriptions(length int, text ...string) []string {
	desc := make([]string, length)
	for i := range desc {
		if len(text) > 0 {
			desc[i] = text[0]
		} else {
			desc[i] = TestSignature
		}
	}
	return desc
}

func sortSet(s *SignatureBatch) *SignatureBatch {
	sort.Sort(sorter{set: s})
	return s
}

type sorter struct {
	set *SignatureBatch
}

func (s sorter) Len() int {
	return len(s.set.Messages)
}

func (s sorter) Swap(i, j int) {
	s.set.Signatures[i], s.set.Signatures[j] = s.set.Signatures[j], s.set.Signatures[i]
	s.set.PublicKeys[i], s.set.PublicKeys[j] = s.set.PublicKeys[j], s.set.PublicKeys[i]
	s.set.Messages[i], s.set.Messages[j] = s.set.Messages[j], s.set.Messages[i]
	s.set.Descriptions[i], s.set.Descriptions[j] = s.set.Descriptions[j], s.set.Descriptions[i]
}

func (s sorter) Less(i, j int) bool {
	return bytes.Compare(s.set.Messages[i][:], s.set.Messages[j][:]) == -1
}
