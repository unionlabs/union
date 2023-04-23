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
	"encoding/json"
)

// ConstructionParseResponse ConstructionParseResponse contains an array of operations that occur in
// a transaction blob. This should match the array of operations provided to
// `/construction/preprocess` and `/construction/payloads`.
type ConstructionParseResponse struct {
	Operations               []*Operation           `json:"operations"`
	AccountIdentifierSigners []*AccountIdentifier   `json:"account_identifier_signers,omitempty"`
	Metadata                 map[string]interface{} `json:"metadata,omitempty"`
}

// MarshalJSON overrides the default JSON marshaler
// and adds the deprecated "signers" field to the response.
func (c *ConstructionParseResponse) MarshalJSON() ([]byte, error) {
	type Alias ConstructionParseResponse

	// Do not create a signers array if the AccountIdentifierSigners
	// array is nil.
	var signers []string
	if c.AccountIdentifierSigners != nil {
		signers = make([]string, len(c.AccountIdentifierSigners))
		for i, signer := range c.AccountIdentifierSigners {
			signers[i] = signer.Address
		}
	}

	j, err := json.Marshal(struct {
		// [DEPRECATED by `account_identifier_signers` in `v1.4.4`] All signers (addresses) of a
		// particular transaction. If the transaction is unsigned, it should be empty.
		Signers []string `json:"signers,omitempty"`
		*Alias
	}{
		Signers: signers,
		Alias:   (*Alias)(c),
	})
	if err != nil {
		return nil, err
	}
	return j, nil
}

// UnmarshalJSON overrides the default JSON unmarshaler
// and reads the deprecated "signers" field from the response.
func (c *ConstructionParseResponse) UnmarshalJSON(b []byte) error {
	type Alias ConstructionParseResponse
	r := struct {
		// [DEPRECATED by `account_identifier_signers` in `v1.4.4`] All signers (addresses) of a
		// particular transaction. If the transaction is unsigned, it should be empty.
		Signers []string `json:"signers,omitempty"`
		*Alias
	}{
		Alias: (*Alias)(c),
	}
	err := json.Unmarshal(b, &r)
	if err != nil {
		return err
	}

	if len(c.AccountIdentifierSigners) == 0 && len(r.Signers) > 0 {
		c.AccountIdentifierSigners = make([]*AccountIdentifier, len(r.Signers))
		for i, signer := range r.Signers {
			c.AccountIdentifierSigners[i] = &AccountIdentifier{Address: signer}
		}
	}

	return nil
}
