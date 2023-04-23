/*******************************************************************************
*   (c) 2018 - 2022 ZondaX AG
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

package ledger_cosmos_go

import (
	"errors"
	"math"

	"github.com/zondax/ledger-go"
)

const (
	validatorCLA = 0x56

	validatorINSGetVersion       = 0
	validatorINSPublicKeyED25519 = 1
	validatorINSSignED25519      = 2

	validatorMessageChunkSize = 250
)

// Validator app
type LedgerTendermintValidator struct {
	// Add support for this app
	api ledger_go.LedgerDevice
}

// RequiredCosmosUserAppVersion indicates the minimum required version of the Tendermint app
func RequiredTendermintValidatorAppVersion() VersionInfo {
	return VersionInfo{0, 0, 5, 0}
}

// FindLedgerCosmosValidatorApp finds a Cosmos validator app running in a ledger device
func FindLedgerTendermintValidatorApp() (_ *LedgerTendermintValidator, rerr error) {
	ledgerAdmin := ledger_go.NewLedgerAdmin()
	ledgerAPI, err := ledgerAdmin.Connect(0)
	if err != nil {
		return nil, err
	}

	defer func() {
		if rerr != nil {
			defer ledgerAPI.Close()
		}
	}()

	ledgerCosmosValidatorApp := &LedgerTendermintValidator{ledgerAPI}
	appVersion, err := ledgerCosmosValidatorApp.GetVersion()
	if err != nil {
		if err.Error() == "[APDU_CODE_CLA_NOT_SUPPORTED] Class not supported" {
			err = errors.New("are you sure the Tendermint Validator app is open?")
		}
		return nil, err
	}

	req := RequiredTendermintValidatorAppVersion()
	if err := CheckVersion(*appVersion, req); err != nil {
		return nil, err
	}

	return ledgerCosmosValidatorApp, err
}

// Close closes a connection with the Cosmos user app
func (ledger *LedgerTendermintValidator) Close() error {
	return ledger.api.Close()
}

// GetVersion returns the current version of the Cosmos user app
func (ledger *LedgerTendermintValidator) GetVersion() (*VersionInfo, error) {
	message := []byte{validatorCLA, validatorINSGetVersion, 0, 0, 0}
	response, err := ledger.api.Exchange(message)

	if err != nil {
		return nil, err
	}

	if len(response) < 4 {
		return nil, errors.New("invalid response")
	}

	return &VersionInfo{
		AppMode: response[0],
		Major:   response[1],
		Minor:   response[2],
		Patch:   response[3],
	}, nil
}

// GetPublicKeyED25519 retrieves the public key for the corresponding bip32 derivation path
func (ledger *LedgerTendermintValidator) GetPublicKeyED25519(bip32Path []uint32) ([]byte, error) {
	pathBytes, err := GetBip32bytesv1(bip32Path, 10)
	if err != nil {
		return nil, err
	}

	header := []byte{validatorCLA, validatorINSPublicKeyED25519, 0, 0, byte(len(pathBytes))}
	message := append(header, pathBytes...)

	response, err := ledger.api.Exchange(message)

	if err != nil {
		return nil, err
	}

	if len(response) < 4 {
		return nil, errors.New("invalid response. Too short")
	}

	return response, nil
}

// SignSECP256K1 signs a message/vote using the Tendermint validator app
func (ledger *LedgerTendermintValidator) SignED25519(bip32Path []uint32, message []byte) ([]byte, error) {
	var packetIndex byte = 1
	var packetCount = 1 + byte(math.Ceil(float64(len(message))/float64(validatorMessageChunkSize)))

	var finalResponse []byte

	var apduMessage []byte

	for packetIndex <= packetCount {
		chunk := validatorMessageChunkSize
		if packetIndex == 1 {
			pathBytes, err := GetBip32bytesv1(bip32Path, 10)
			if err != nil {
				return nil, err
			}
			header := []byte{
				validatorCLA,
				validatorINSSignED25519,
				packetIndex,
				packetCount,
				byte(len(pathBytes))}

			apduMessage = append(header, pathBytes...)
		} else {
			if len(message) < validatorMessageChunkSize {
				chunk = len(message)
			}
			header := []byte{
				validatorCLA,
				validatorINSSignED25519,
				packetIndex,
				packetCount,
				byte(chunk)}

			apduMessage = append(header, message[:chunk]...)
		}

		response, err := ledger.api.Exchange(apduMessage)
		if err != nil {
			return nil, err
		}

		finalResponse = response
		if packetIndex > 1 {
			message = message[chunk:]
		}
		packetIndex++

	}
	return finalResponse, nil
}
