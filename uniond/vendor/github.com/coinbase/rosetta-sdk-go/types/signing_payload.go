// Copyright 2020 Coinbase, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package types

import (
	"encoding/hex"
	"encoding/json"
)

// SigningPayload SigningPayload is signed by the client with the keypair associated with an
// AccountIdentifier using the specified SignatureType. SignatureType can be optionally populated if
// there is a restriction on the signature scheme that can be used to sign the payload.
type SigningPayload struct {
	AccountIdentifier *AccountIdentifier `json:"account_identifier,omitempty"`
	Bytes             []byte             `json:"hex_bytes"`
	SignatureType     SignatureType      `json:"signature_type,omitempty"`
}

// MarshalJSON overrides the default JSON marshaler
// and encodes bytes as hex instead of base64. It also
// writes the deprecated "address" field to the response.
func (s *SigningPayload) MarshalJSON() ([]byte, error) {
	type Alias SigningPayload
	addressString := ""
	if s.AccountIdentifier != nil {
		addressString = s.AccountIdentifier.Address
	}

	j, err := json.Marshal(struct {
		// [DEPRECATED by `account_identifier` in `v1.4.4`] Address is the network-specific
		// address of the account that should sign the payload.
		Address string `json:"address,omitempty"`
		Bytes   string `json:"hex_bytes"`
		*Alias
	}{
		Address: addressString,
		Bytes:   hex.EncodeToString(s.Bytes),
		Alias:   (*Alias)(s),
	})
	if err != nil {
		return nil, err
	}
	return j, nil
}

// UnmarshalJSON overrides the default JSON unmarshaler
// and decodes bytes from hex instead of base64. It also
// reads the deprecated "address" field from the response.
func (s *SigningPayload) UnmarshalJSON(b []byte) error {
	type Alias SigningPayload
	r := struct {
		// [DEPRECATED by `account_identifier` in `v1.4.4`] Address is the network-specific
		// address of the account that should sign the payload.
		Address string `json:"address,omitempty"`
		Bytes   string `json:"hex_bytes"`
		*Alias
	}{
		Alias: (*Alias)(s),
	}
	err := json.Unmarshal(b, &r)
	if err != nil {
		return err
	}

	bytes, err := hex.DecodeString(r.Bytes)
	if err != nil {
		return err
	}
	s.Bytes = bytes

	if s.AccountIdentifier == nil && len(r.Address) > 0 {
		s.AccountIdentifier = &AccountIdentifier{
			Address: r.Address,
		}
	}

	return nil
}
