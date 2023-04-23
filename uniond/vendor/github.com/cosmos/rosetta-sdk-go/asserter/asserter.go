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

package asserter

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"path"

	"github.com/coinbase/rosetta-sdk-go/types"
)

// Asserter contains all logic to perform static
// validation on Rosetta Server responses.
type Asserter struct {
	// These variables are used for response assertion.
	network             *types.NetworkIdentifier
	operationTypes      []string
	operationStatusMap  map[string]bool
	errorTypeMap        map[int32]*types.Error
	genesisBlock        *types.BlockIdentifier
	timestampStartIndex int64

	// These variables are used for request assertion.
	historicalBalanceLookup bool
	supportedNetworks       []*types.NetworkIdentifier
	callMethods             map[string]struct{}
	mempoolCoins            bool
	validations             *Validations
}

// Validations is used to define stricter validations
// on the transaction. Fore more details please refer to
// https://github.com/cosmos/rosetta-sdk-go/tree/master/asserter#readme
type Validations struct {
	Enabled          bool                 `json:"enabled"`
	RelatedOpsExists bool                 `json:"related_ops_exists"`
	ChainType        ChainType            `json:"chain_type"`
	Payment          *ValidationOperation `json:"payment"`
	Fee              *ValidationOperation `json:"fee"`
}

type ValidationOperation struct {
	Name      string     `json:"name"`
	Operation *Operation `json:"operation"`
}

type Operation struct {
	Count         int  `json:"count"`
	ShouldBalance bool `json:"should_balance"`
}

type ChainType string

const (
	Account ChainType = "account"
	UTXO    ChainType = "utxo"
)

// NewServer constructs a new Asserter for use in the
// server package.
func NewServer(
	supportedOperationTypes []string,
	historicalBalanceLookup bool,
	supportedNetworks []*types.NetworkIdentifier,
	callMethods []string,
	mempoolCoins bool,
	validationFilePath string,
) (*Asserter, error) {
	if err := OperationTypes(supportedOperationTypes); err != nil {
		return nil, fmt.Errorf("operation types %v are invalid: %w", supportedOperationTypes, err)
	}

	if err := SupportedNetworks(supportedNetworks); err != nil {
		return nil, fmt.Errorf(
			"network identifiers %s are invalid: %w",
			types.PrintStruct(supportedNetworks),
			err,
		)
	}

	validationConfig, err := getValidationConfig(validationFilePath)
	if err != nil {
		return nil, fmt.Errorf("config %s is invalid: %w", validationFilePath, err)
	}

	callMap := map[string]struct{}{}
	for _, method := range callMethods {
		if len(method) == 0 {
			return nil, ErrCallMethodEmpty
		}

		if _, ok := callMap[method]; ok {
			return nil, fmt.Errorf("failed to call method %s: %w", method, ErrCallMethodDuplicate)
		}

		callMap[method] = struct{}{}
	}

	return &Asserter{
		operationTypes:          supportedOperationTypes,
		historicalBalanceLookup: historicalBalanceLookup,
		supportedNetworks:       supportedNetworks,
		callMethods:             callMap,
		mempoolCoins:            mempoolCoins,
		validations:             validationConfig,
	}, nil
}

// NewClientWithResponses constructs a new Asserter
// from a NetworkStatusResponse and
// NetworkOptionsResponse.
func NewClientWithResponses(
	network *types.NetworkIdentifier,
	networkStatus *types.NetworkStatusResponse,
	networkOptions *types.NetworkOptionsResponse,
	validationFilePath string,
) (*Asserter, error) {
	if err := NetworkIdentifier(network); err != nil {
		return nil, fmt.Errorf(
			"network identifier %s is invalid: %w",
			types.PrintStruct(network),
			err,
		)
	}

	if err := NetworkStatusResponse(networkStatus); err != nil {
		return nil, fmt.Errorf(
			"network status response %s is invalid: %w",
			types.PrintStruct(networkStatus),
			err,
		)
	}

	if err := NetworkOptionsResponse(networkOptions); err != nil {
		return nil, fmt.Errorf(
			"network options response %s is invalid: %w",
			types.PrintStruct(networkOptions),
			err,
		)
	}

	validationConfig, err := getValidationConfig(validationFilePath)
	if err != nil {
		return nil, fmt.Errorf("config %s is invalid: %w", validationFilePath, err)
	}

	return NewClientWithOptions(
		network,
		networkStatus.GenesisBlockIdentifier,
		networkOptions.Allow.OperationTypes,
		networkOptions.Allow.OperationStatuses,
		networkOptions.Allow.Errors,
		networkOptions.Allow.TimestampStartIndex,
		validationConfig,
	)
}

// Configuration is the static configuration of an Asserter. This
// configuration can be exported by the Asserter and used to instantiate an
// Asserter.
type Configuration struct {
	NetworkIdentifier          *types.NetworkIdentifier `json:"network_identifier"`
	GenesisBlockIdentifier     *types.BlockIdentifier   `json:"genesis_block_identifier"`
	AllowedOperationTypes      []string                 `json:"allowed_operation_types"`
	AllowedOperationStatuses   []*types.OperationStatus `json:"allowed_operation_statuses"`
	AllowedErrors              []*types.Error           `json:"allowed_errors"`
	AllowedTimestampStartIndex int64                    `json:"allowed_timestamp_start_index"`
}

// NewClientWithFile constructs a new Asserter using a specification
// file instead of responses. This can be useful for running reliable
// systems that error when updates to the server (more error types,
// more operations, etc.) significantly change how to parse the chain.
// The filePath provided is parsed relative to the current directory.
func NewClientWithFile(
	filePath string,
) (*Asserter, error) {
	content, err := ioutil.ReadFile(path.Clean(filePath))
	if err != nil {
		return nil, fmt.Errorf("failed to read file %s: %w", filePath, err)
	}

	config := &Configuration{}
	if err := json.Unmarshal(content, config); err != nil {
		return nil, fmt.Errorf("failed to unmarshal content of file %s: %w", filePath, err)
	}

	return NewClientWithOptions(
		config.NetworkIdentifier,
		config.GenesisBlockIdentifier,
		config.AllowedOperationTypes,
		config.AllowedOperationStatuses,
		config.AllowedErrors,
		&config.AllowedTimestampStartIndex,
		&Validations{
			Enabled: false,
		},
	)
}

// NewClientWithOptions constructs a new Asserter using the provided
// arguments instead of using a NetworkStatusResponse and a
// NetworkOptionsResponse.
func NewClientWithOptions(
	network *types.NetworkIdentifier,
	genesisBlockIdentifier *types.BlockIdentifier,
	operationTypes []string,
	operationStatuses []*types.OperationStatus,
	errors []*types.Error,
	timestampStartIndex *int64,
	validationConfig *Validations,
) (*Asserter, error) {
	if err := NetworkIdentifier(network); err != nil {
		return nil, fmt.Errorf(
			"network identifier %s is invalid: %w",
			types.PrintStruct(network),
			err,
		)
	}

	if err := BlockIdentifier(genesisBlockIdentifier); err != nil {
		return nil, fmt.Errorf(
			"genesis block identifier %s is invalid: %w",
			types.PrintStruct(genesisBlockIdentifier),
			err,
		)
	}

	if err := OperationStatuses(operationStatuses); err != nil {
		return nil, fmt.Errorf(
			"operation statuses %s are invalid: %w",
			types.PrintStruct(operationStatuses),
			err,
		)
	}

	if err := OperationTypes(operationTypes); err != nil {
		return nil, fmt.Errorf(
			"operation types %s are invalid: %w",
			types.PrintStruct(operationTypes),
			err,
		)
	}

	// TimestampStartIndex defaults to genesisIndex + 1 (this
	// avoid breaking existing clients using < v1.4.6).
	parsedTimestampStartIndex := genesisBlockIdentifier.Index + 1
	if timestampStartIndex != nil {
		if *timestampStartIndex < 0 {
			return nil, fmt.Errorf(
				"failed to validate index %d: %w",
				*timestampStartIndex,
				ErrTimestampStartIndexInvalid,
			)
		}

		parsedTimestampStartIndex = *timestampStartIndex
	}

	asserter := &Asserter{
		network:             network,
		operationTypes:      operationTypes,
		genesisBlock:        genesisBlockIdentifier,
		timestampStartIndex: parsedTimestampStartIndex,
		validations:         validationConfig,
	}

	asserter.operationStatusMap = map[string]bool{}
	for _, status := range operationStatuses {
		asserter.operationStatusMap[status.Status] = status.Successful
	}

	asserter.errorTypeMap = map[int32]*types.Error{}
	for _, err := range errors {
		asserter.errorTypeMap[err.Code] = err
	}

	return asserter, nil
}

// ClientConfiguration returns all variables currently set in an Asserter.
// This function will error if it is called on an uninitialized asserter.
func (a *Asserter) ClientConfiguration() (*Configuration, error) {
	if a == nil {
		return nil, ErrAsserterNotInitialized
	}

	operationStatuses := []*types.OperationStatus{}
	for k, v := range a.operationStatusMap {
		operationStatuses = append(operationStatuses, &types.OperationStatus{
			Status:     k,
			Successful: v,
		})
	}

	errors := []*types.Error{}
	for _, v := range a.errorTypeMap {
		errors = append(errors, v)
	}

	return &Configuration{
		NetworkIdentifier:          a.network,
		GenesisBlockIdentifier:     a.genesisBlock,
		AllowedOperationTypes:      a.operationTypes,
		AllowedOperationStatuses:   operationStatuses,
		AllowedErrors:              errors,
		AllowedTimestampStartIndex: a.timestampStartIndex,
	}, nil
}

// OperationSuccessful returns a boolean indicating if a types.Operation is
// successful and should be applied in a transaction. This should only be called
// AFTER an operation has been validated.
func (a *Asserter) OperationSuccessful(operation *types.Operation) (bool, error) {
	if a == nil {
		return false, ErrAsserterNotInitialized
	}

	if operation.Status == nil || len(*operation.Status) == 0 {
		return false, ErrOperationStatusMissing
	}

	val, ok := a.operationStatusMap[*operation.Status]
	if !ok {
		return false, fmt.Errorf("operation status %s is not found", *operation.Status)
	}

	return val, nil
}

func getValidationConfig(validationFilePath string) (*Validations, error) {
	validationConfig := &Validations{
		Enabled: false,
	}
	if validationFilePath != "" {
		content, err := ioutil.ReadFile(path.Clean(validationFilePath))
		if err != nil {
			return nil, fmt.Errorf("failed to read file %s: %w", validationFilePath, err)
		}

		if err := json.Unmarshal(content, validationConfig); err != nil {
			return nil, fmt.Errorf(
				"failed to unmarshal content of file %s: %w",
				validationFilePath,
				err,
			)
		}
	}
	return validationConfig, nil
}
