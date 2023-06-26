package bls

import (
	"encoding/hex"
	"fmt"

	"github.com/pkg/errors"
)

// AggregatedSignature represents aggregated signature produced by AggregateBatch()
const AggregatedSignature = "bls aggregated signature"

// SignatureBatch refers to the defined set of
// signatures and its respective public keys and
// messages required to verify it.
type SignatureBatch struct {
	Signatures   [][]byte
	PublicKeys   []PublicKey
	Messages     [][32]byte
	Descriptions []string
}

// NewSet constructs an empty signature batch object.
func NewSet() *SignatureBatch {
	return &SignatureBatch{
		Signatures:   [][]byte{},
		PublicKeys:   []PublicKey{},
		Messages:     [][32]byte{},
		Descriptions: []string{},
	}
}

// Join merges the provided signature batch to out current one.
func (s *SignatureBatch) Join(set *SignatureBatch) *SignatureBatch {
	s.Signatures = append(s.Signatures, set.Signatures...)
	s.PublicKeys = append(s.PublicKeys, set.PublicKeys...)
	s.Messages = append(s.Messages, set.Messages...)
	s.Descriptions = append(s.Descriptions, set.Descriptions...)
	return s
}

// Verify the current signature batch using the batch verify algorithm.
func (s *SignatureBatch) Verify() (bool, error) {
	return VerifyMultipleSignatures(s.Signatures, s.Messages, s.PublicKeys)
}

// VerifyVerbosely verifies signatures as a whole at first, if fails, fallback
// to verify each single signature to identify invalid ones.
func (s *SignatureBatch) VerifyVerbosely() (bool, error) {
	valid, err := s.Verify()
	if err != nil || valid {
		return valid, err
	}

	// if signature batch is invalid, we then verify signatures one by one.

	errmsg := "some signatures are invalid. details:"
	for i := 0; i < len(s.Signatures); i++ {
		sig := s.Signatures[i]
		msg := s.Messages[i]
		pubKey := s.PublicKeys[i]

		valid, err := VerifySignature(sig, msg, pubKey)
		if !valid {
			desc := s.Descriptions[i]
			if err != nil {
				errmsg += fmt.Sprintf("\nsignature '%s' is invalid."+
					" signature: 0x%s, public key: 0x%s, message: 0x%v, error: %v",
					desc, hex.EncodeToString(sig), hex.EncodeToString(pubKey.Marshal()),
					hex.EncodeToString(msg[:]), err)
			} else {
				errmsg += fmt.Sprintf("\nsignature '%s' is invalid."+
					" signature: 0x%s, public key: 0x%s, message: 0x%v",
					desc, hex.EncodeToString(sig), hex.EncodeToString(pubKey.Marshal()),
					hex.EncodeToString(msg[:]))
			}
		}
	}

	return false, errors.Errorf(errmsg)
}

// Copy the attached signature batch and return it
// to the caller.
func (s *SignatureBatch) Copy() *SignatureBatch {
	signatures := make([][]byte, len(s.Signatures))
	pubkeys := make([]PublicKey, len(s.PublicKeys))
	messages := make([][32]byte, len(s.Messages))
	descriptions := make([]string, len(s.Descriptions))
	for i := range s.Signatures {
		sig := make([]byte, len(s.Signatures[i]))
		copy(sig, s.Signatures[i])
		signatures[i] = sig
	}
	for i := range s.PublicKeys {
		pubkeys[i] = s.PublicKeys[i].Copy()
	}
	for i := range s.Messages {
		copy(messages[i][:], s.Messages[i][:])
	}
	copy(descriptions, s.Descriptions)
	return &SignatureBatch{
		Signatures:   signatures,
		PublicKeys:   pubkeys,
		Messages:     messages,
		Descriptions: descriptions,
	}
}

// RemoveDuplicates removes duplicate signature sets from the signature batch.
func (s *SignatureBatch) RemoveDuplicates() (int, *SignatureBatch, error) {
	if len(s.Signatures) == 0 || len(s.PublicKeys) == 0 || len(s.Messages) == 0 {
		return 0, s, nil
	}
	if len(s.Signatures) != len(s.PublicKeys) || len(s.Signatures) != len(s.Messages) {
		return 0, s, errors.Errorf("mismatch number of signatures, publickeys and messages in signature batch. "+
			"Signatures %d, Public Keys %d , Messages %d", s.Signatures, s.PublicKeys, s.Messages)
	}
	sigMap := make(map[string]int)
	duplicateSet := make(map[int]bool)
	for i := 0; i < len(s.Signatures); i++ {
		if sigIdx, ok := sigMap[string(s.Signatures[i])]; ok {
			if s.PublicKeys[sigIdx].Equals(s.PublicKeys[i]) &&
				s.Messages[sigIdx] == s.Messages[i] {
				duplicateSet[i] = true
				continue
			}
		}
		sigMap[string(s.Signatures[i])] = i
	}

	sigs := s.Signatures[:0]
	pubs := s.PublicKeys[:0]
	msgs := s.Messages[:0]
	descs := s.Descriptions[:0]

	for i := 0; i < len(s.Signatures); i++ {
		if duplicateSet[i] {
			continue
		}
		sigs = append(sigs, s.Signatures[i])
		pubs = append(pubs, s.PublicKeys[i])
		msgs = append(msgs, s.Messages[i])
		descs = append(descs, s.Descriptions[i])
	}

	s.Signatures = sigs
	s.PublicKeys = pubs
	s.Messages = msgs
	s.Descriptions = descs

	return len(duplicateSet), s, nil
}

// AggregateBatch aggregates common messages in the provided batch to
// reduce the number of pairings required when we finally verify the
// whole batch.
func (s *SignatureBatch) AggregateBatch() (*SignatureBatch, error) {
	if len(s.Signatures) != len(s.PublicKeys) || len(s.Signatures) != len(s.Messages) || len(s.Signatures) != len(s.Descriptions) {
		return s, errors.Errorf("mismatch number of signatures, publickeys, messages and descriptions in signature batch. "+
			"Signatures %d, Public Keys %d , Messages %d, Descriptions %d", len(s.Signatures), len(s.PublicKeys), len(s.Messages), len(s.Descriptions))
	}
	if len(s.Signatures) == 0 {
		return s, nil
	}
	msgMap := make(map[[32]byte]*SignatureBatch)

	for i := 0; i < len(s.Messages); i++ {
		currMsg := s.Messages[i]
		currBatch, ok := msgMap[currMsg]
		if ok {
			currBatch.Signatures = append(currBatch.Signatures, s.Signatures[i])
			currBatch.Messages = append(currBatch.Messages, s.Messages[i])
			currBatch.PublicKeys = append(currBatch.PublicKeys, s.PublicKeys[i])
			currBatch.Descriptions = append(currBatch.Descriptions, s.Descriptions[i])
			continue
		}
		currBatch = &SignatureBatch{
			Signatures:   [][]byte{s.Signatures[i]},
			Messages:     [][32]byte{s.Messages[i]},
			PublicKeys:   []PublicKey{s.PublicKeys[i]},
			Descriptions: []string{s.Descriptions[i]},
		}
		msgMap[currMsg] = currBatch
	}
	newSt := NewSet()
	for rt, b := range msgMap {
		if len(b.PublicKeys) > 1 {
			aggPub := AggregateMultiplePubkeys(b.PublicKeys)
			aggSig, err := AggregateCompressedSignatures(b.Signatures)
			if err != nil {
				return nil, err
			}
			copiedRt := rt
			b.PublicKeys = []PublicKey{aggPub}
			b.Signatures = [][]byte{aggSig.Marshal()}
			b.Messages = [][32]byte{copiedRt}
			b.Descriptions = []string{AggregatedSignature}
		}
		newObj := *b
		newSt = newSt.Join(&newObj)
	}
	return newSt, nil
}
