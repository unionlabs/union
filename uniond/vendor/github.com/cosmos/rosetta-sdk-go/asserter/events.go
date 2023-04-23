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

// BlockEvent ensures a *types.BlockEvent
// is valid.
func BlockEvent(
	event *types.BlockEvent,
) error {
	if event.Sequence < 0 {
		return ErrSequenceInvalid
	}

	if err := BlockIdentifier(event.BlockIdentifier); err != nil {
		return fmt.Errorf(
			"block identifier %s is invalid: %w",
			types.PrintStruct(event.BlockIdentifier),
			err,
		)
	}

	switch event.Type {
	case types.ADDED, types.REMOVED:
	default:
		return ErrBlockEventTypeInvalid
	}

	return nil
}

// EventsBlocksResponse ensures a *types.EventsBlocksResponse
// is valid.
func EventsBlocksResponse(
	response *types.EventsBlocksResponse,
) error {
	if response.MaxSequence < 0 {
		return ErrMaxSequenceInvalid
	}

	seq := int64(-1)
	for i, event := range response.Events {
		if err := BlockEvent(event); err != nil {
			return fmt.Errorf("block event %s is invalid: %w", types.PrintStruct(event), err)
		}

		if seq == -1 {
			seq = event.Sequence
		}

		if event.Sequence != seq+int64(i) {
			return ErrSequenceOutOfOrder
		}
	}

	return nil
}
