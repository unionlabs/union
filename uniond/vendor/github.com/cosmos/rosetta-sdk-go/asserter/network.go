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
	"fmt"

	"github.com/coinbase/rosetta-sdk-go/types"
)

// SubNetworkIdentifier asserts a types.SubNetworkIdentifier is valid (if not nil).
func SubNetworkIdentifier(subNetworkIdentifier *types.SubNetworkIdentifier) error {
	if subNetworkIdentifier == nil {
		return nil
	}

	if subNetworkIdentifier.Network == "" {
		return ErrSubNetworkIdentifierInvalid
	}

	return nil
}

// NetworkIdentifier ensures a types.NetworkIdentifier has
// a valid blockchain and network.
func NetworkIdentifier(network *types.NetworkIdentifier) error {
	if network == nil {
		return ErrNetworkIdentifierIsNil
	}

	if network.Blockchain == "" {
		return ErrNetworkIdentifierBlockchainMissing
	}

	if network.Network == "" {
		return ErrNetworkIdentifierNetworkMissing
	}

	return SubNetworkIdentifier(network.SubNetworkIdentifier)
}

// Peer ensures a types.Peer has a valid peer_id.
func Peer(peer *types.Peer) error {
	if peer == nil || peer.PeerID == "" {
		return ErrPeerIDMissing
	}

	return nil
}

// Version ensures the version of the node is
// returned.
func Version(version *types.Version) error {
	if version == nil {
		return ErrVersionIsNil
	}

	if version.NodeVersion == "" {
		return ErrVersionNodeVersionMissing
	}

	if version.MiddlewareVersion != nil && *version.MiddlewareVersion == "" {
		return ErrVersionMiddlewareVersionMissing
	}

	return nil
}

// SyncStatus ensures any types.SyncStatus is valid.
func SyncStatus(status *types.SyncStatus) error {
	if status == nil {
		return nil
	}

	if status.CurrentIndex != nil && *status.CurrentIndex < 0 {
		return ErrSyncStatusCurrentIndexNegative
	}

	if status.TargetIndex != nil && *status.TargetIndex < 0 {
		return ErrSyncStatusTargetIndexNegative
	}

	if status.Stage != nil && len(*status.Stage) == 0 {
		return ErrSyncStatusStageInvalid
	}

	return nil
}

// NetworkStatusResponse ensures any types.NetworkStatusResponse
// is valid.
func NetworkStatusResponse(response *types.NetworkStatusResponse) error {
	if response == nil {
		return ErrNetworkStatusResponseIsNil
	}

	if err := BlockIdentifier(response.CurrentBlockIdentifier); err != nil {
		return fmt.Errorf(
			"current block identifier %s is invalid: %w",
			types.PrintStruct(response.CurrentBlockIdentifier),
			err,
		)
	}

	if err := Timestamp(response.CurrentBlockTimestamp); err != nil {
		return fmt.Errorf(
			"current block timestamp %d is invalid: %w",
			response.CurrentBlockTimestamp,
			err,
		)
	}

	if err := BlockIdentifier(response.GenesisBlockIdentifier); err != nil {
		return fmt.Errorf(
			"genesis block identifier %s is invalid: %w",
			types.PrintStruct(response.GenesisBlockIdentifier),
			err,
		)
	}

	for _, peer := range response.Peers {
		if err := Peer(peer); err != nil {
			return fmt.Errorf("peer %s is invalid: %w", types.PrintStruct(peer), err)
		}
	}

	if err := SyncStatus(response.SyncStatus); err != nil {
		return fmt.Errorf(
			"sync status %s is invalid: %w",
			types.PrintStruct(response.SyncStatus),
			err,
		)
	}

	return nil
}

// OperationStatuses ensures all items in Options.Allow.OperationStatuses
// are valid and that there exists at least 1 successful status.
func OperationStatuses(statuses []*types.OperationStatus) error {
	if len(statuses) == 0 {
		return ErrNoAllowedOperationStatuses
	}

	statusStatuses := make([]string, len(statuses))
	foundSuccessful := false
	for i, status := range statuses {
		if status.Status == "" {
			return ErrOperationStatusMissing
		}

		if status.Successful {
			foundSuccessful = true
		}

		statusStatuses[i] = status.Status
	}

	if !foundSuccessful {
		return ErrNoSuccessfulAllowedOperationStatuses
	}

	return StringArray("Allow.OperationStatuses", statusStatuses)
}

// OperationTypes ensures all items in Options.Allow.OperationStatuses
// are valid and that there are no repeats.
func OperationTypes(types []string) error {
	return StringArray("Allow.OperationTypes", types)
}

// Error ensures a types.Error is valid.
func Error(err *types.Error) error {
	if err == nil {
		return ErrErrorIsNil
	}

	if err.Code < 0 {
		return ErrErrorCodeIsNeg
	}

	if len(err.Message) == 0 {
		return ErrErrorMessageMissing
	}

	if err.Description != nil && len(*err.Description) == 0 {
		return ErrErrorDescriptionEmpty
	}

	return nil
}

// Errors ensures each types.Error in a slice is valid
// and that there is no error code collision.
func Errors(rosettaErrors []*types.Error) error {
	statusCodes := map[int32]struct{}{}

	for _, rosettaError := range rosettaErrors {
		if err := Error(rosettaError); err != nil {
			return fmt.Errorf("error %s is invalid: %w", types.PrintStruct(rosettaError), err)
		}

		// Error.Details should not be populated
		// in the /network/options response.
		if len(rosettaError.Details) > 0 {
			return ErrErrorDetailsPopulated
		}

		_, exists := statusCodes[rosettaError.Code]
		if exists {
			return ErrErrorCodeUsedMultipleTimes
		}

		statusCodes[rosettaError.Code] = struct{}{}
	}

	return nil
}

// BalanceExemptions ensures []*types.BalanceExemption is valid.
func BalanceExemptions(exemptions []*types.BalanceExemption) error {
	for _, exemption := range exemptions {
		if exemption == nil {
			return fmt.Errorf(
				"balance exemption %s is invalid: %w",
				types.PrintStruct(exemption),
				ErrBalanceExemptionIsNil,
			)
		}

		switch exemption.ExemptionType {
		case types.BalanceLessOrEqual, types.BalanceGreaterOrEqual, types.BalanceDynamic:
		default:
			return fmt.Errorf(
				"balance exemption type %s is invalid: %w",
				types.PrintStruct(exemption.ExemptionType),
				ErrBalanceExemptionTypeInvalid,
			)
		}

		if exemption.Currency == nil && exemption.SubAccountAddress == nil {
			return fmt.Errorf(
				"balance exemption %s is invalid: %w",
				types.PrintStruct(exemption),
				ErrBalanceExemptionMissingSubject,
			)
		}

		if exemption.Currency != nil {
			if err := Currency(exemption.Currency); err != nil {
				return fmt.Errorf(
					"balance exemption currency %s is invalid: %w",
					types.PrintStruct(exemption.Currency),
					err,
				)
			}
		}

		if exemption.SubAccountAddress != nil {
			if len(*exemption.SubAccountAddress) == 0 {
				return fmt.Errorf(
					"balance exemption sub account address %s is invalid: %w",
					types.PrintStruct(exemption.SubAccountAddress),
					ErrBalanceExemptionSubAccountAddressEmpty,
				)
			}
		}
	}

	return nil
}

// CallMethods ensures Allow.CallMethods are valid.
func CallMethods(methods []string) error {
	if len(methods) == 0 {
		return nil
	}

	if err := StringArray("Allow.CallMethods", methods); err != nil {
		return fmt.Errorf("methods %v are invalid: %w", methods, err)
	}

	return nil
}

// Allow ensures a types.Allow object is valid.
func Allow(allowed *types.Allow) error {
	if allowed == nil {
		return ErrAllowIsNil
	}

	if err := OperationStatuses(allowed.OperationStatuses); err != nil {
		return fmt.Errorf(
			"operation statuses %s are invalid: %w",
			types.PrintStruct(allowed.OperationStatuses),
			err,
		)
	}

	if err := OperationTypes(allowed.OperationTypes); err != nil {
		return fmt.Errorf("operation types %v are invalid: %w", allowed.OperationTypes, err)
	}

	if err := Errors(allowed.Errors); err != nil {
		return fmt.Errorf("errors are %s are invalid: %w", types.PrintStruct(allowed.Errors), err)
	}

	if err := CallMethods(allowed.CallMethods); err != nil {
		return fmt.Errorf("call methods %v are invalid: %w", allowed.CallMethods, err)
	}

	if err := BalanceExemptions(allowed.BalanceExemptions); err != nil {
		return fmt.Errorf(
			"balance exemptions %s are invalid: %w",
			types.PrintStruct(allowed.BalanceExemptions),
			err,
		)
	}

	if len(allowed.BalanceExemptions) > 0 && !allowed.HistoricalBalanceLookup {
		return ErrBalanceExemptionNoHistoricalLookup
	}

	if allowed.TimestampStartIndex != nil && *allowed.TimestampStartIndex < 0 {
		return fmt.Errorf(
			"timestamp start index %d is invalid: %w",
			*allowed.TimestampStartIndex,
			ErrTimestampStartIndexInvalid,
		)
	}

	return nil
}

// NetworkOptionsResponse ensures a types.NetworkOptionsResponse object is valid.
func NetworkOptionsResponse(options *types.NetworkOptionsResponse) error {
	if options == nil {
		return ErrNetworkOptionsResponseIsNil
	}

	if err := Version(options.Version); err != nil {
		return err
	}

	return Allow(options.Allow)
}

// containsNetworkIdentifier returns a boolean indicating if a
// *types.NetworkIdentifier is contained within a slice of
// *types.NetworkIdentifier. The check for equality takes
// into account everything within the types.NetworkIdentifier
// struct (including currency.Metadata).
func containsNetworkIdentifier(
	networks []*types.NetworkIdentifier,
	network *types.NetworkIdentifier,
) bool {
	for _, net := range networks {
		if types.Hash(net) == types.Hash(network) {
			return true
		}
	}

	return false
}

// NetworkListResponse ensures a types.NetworkListResponse object is valid.
func NetworkListResponse(response *types.NetworkListResponse) error {
	if response == nil {
		return ErrNetworkListResponseIsNil
	}

	seen := make([]*types.NetworkIdentifier, 0)
	for _, network := range response.NetworkIdentifiers {
		if err := NetworkIdentifier(network); err != nil {
			return fmt.Errorf(
				"network identifier %s is invalid: %w",
				types.PrintStruct(network),
				err,
			)
		}

		if containsNetworkIdentifier(seen, network) {
			return ErrNetworkListResponseNetworksContainsDuplicates
		}

		seen = append(seen, network)
	}

	return nil
}
