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

// ConstructionDeriveResponse ConstructionDeriveResponse is returned by the `/construction/derive`
// endpoint.
type ConstructionDeriveResponse struct {
	AccountIdentifier *AccountIdentifier     `json:"account_identifier,omitempty"`
	Metadata          map[string]interface{} `json:"metadata,omitempty"`
}

// MarshalJSON overrides the default JSON marshaler
// and adds the deprecated "address" field to the response.
func (c *ConstructionDeriveResponse) MarshalJSON() ([]byte, error) {
	type Alias ConstructionDeriveResponse
	addressString := ""
	if c.AccountIdentifier != nil {
		addressString = c.AccountIdentifier.Address
	}

	j, err := json.Marshal(struct {
		// [DEPRECATED by `account_identifier` in `v1.4.4`] Address is the network-specific
		// address of the account that should sign the payload.
		Address string `json:"address,omitempty"`
		*Alias
	}{
		Address: addressString,
		Alias:   (*Alias)(c),
	})
	if err != nil {
		return nil, err
	}
	return j, nil
}

// UnmarshalJSON overrides the default JSON unmarshaler
// and reads the deprecated "address" field from the response.
func (c *ConstructionDeriveResponse) UnmarshalJSON(b []byte) error {
	type Alias ConstructionDeriveResponse
	r := struct {
		// [DEPRECATED by `account_identifier` in `v1.4.4`] Address is the network-specific
		// address of the account that should sign the payload.
		Address string `json:"address,omitempty"`
		*Alias
	}{
		Alias: (*Alias)(c),
	}
	err := json.Unmarshal(b, &r)
	if err != nil {
		return err
	}

	if c.AccountIdentifier == nil && len(r.Address) > 0 {
		c.AccountIdentifier = &AccountIdentifier{
			Address: r.Address,
		}
	}

	return nil
}
