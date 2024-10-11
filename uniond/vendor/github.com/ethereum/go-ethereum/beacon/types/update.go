// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package types

import (
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/beacon/merkle"
	"github.com/ethereum/go-ethereum/beacon/params"
	"github.com/ethereum/go-ethereum/common"
)

// LightClientUpdate is a proof of the next sync committee root based on a header
// signed by the sync committee of the given period. Optionally, the update can
// prove quasi-finality by the signed header referring to a previous, finalized
// header from the same period, and the finalized header referring to the next
// sync committee root.
//
// See data structure definition here:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientupdate
type LightClientUpdate struct {
	AttestedHeader          SignedHeader  // Arbitrary header out of the period signed by the sync committee
	NextSyncCommitteeRoot   common.Hash   // Sync committee of the next period advertised in the current one
	NextSyncCommitteeBranch merkle.Values // Proof for the next period's sync committee

	FinalizedHeader *Header       `rlp:"nil"` // Optional header to announce a point of finality
	FinalityBranch  merkle.Values // Proof for the announced finality

	score *UpdateScore // Weight of the update to compare between competing ones
}

// Validate verifies the validity of the update.
func (update *LightClientUpdate) Validate() error {
	period := update.AttestedHeader.Header.SyncPeriod()
	if SyncPeriod(update.AttestedHeader.SignatureSlot) != period {
		return errors.New("signature slot and signed header are from different periods")
	}
	if update.FinalizedHeader != nil {
		if update.FinalizedHeader.SyncPeriod() != period {
			return errors.New("finalized header is from different period")
		}
		if err := merkle.VerifyProof(update.AttestedHeader.Header.StateRoot, params.StateIndexFinalBlock, update.FinalityBranch, merkle.Value(update.FinalizedHeader.Hash())); err != nil {
			return fmt.Errorf("invalid finalized header proof: %w", err)
		}
	}
	if err := merkle.VerifyProof(update.AttestedHeader.Header.StateRoot, params.StateIndexNextSyncCommittee, update.NextSyncCommitteeBranch, merkle.Value(update.NextSyncCommitteeRoot)); err != nil {
		return fmt.Errorf("invalid next sync committee proof: %w", err)
	}
	return nil
}

// Score returns the UpdateScore describing the proof strength of the update
// Note: thread safety can be ensured by always calling Score on a newly received
// or decoded update before making it potentially available for other threads
func (update *LightClientUpdate) Score() UpdateScore {
	if update.score == nil {
		update.score = &UpdateScore{
			SignerCount:     uint32(update.AttestedHeader.Signature.SignerCount()),
			SubPeriodIndex:  uint32(update.AttestedHeader.Header.Slot & 0x1fff),
			FinalizedHeader: update.FinalizedHeader != nil,
		}
	}
	return *update.score
}

// UpdateScore allows the comparison between updates at the same period in order
// to find the best update chain that provides the strongest proof of being canonical.
//
// UpdateScores have a tightly packed binary encoding format for efficient p2p
// protocol transmission. Each UpdateScore is encoded in 3 bytes.
// When interpreted as a 24 bit little indian unsigned integer:
//   - the lowest 10 bits contain the number of signers in the header signature aggregate
//   - the next 13 bits contain the "sub-period index" which is he signed header's
//     slot modulo params.SyncPeriodLength (which is correlated with the risk of the chain being
//     re-orged before the previous period boundary in case of non-finalized updates)
//   - the highest bit is set when the update is finalized (meaning that the finality
//     header referenced by the signed header is in the same period as the signed
//     header, making reorgs before the period boundary impossible
type UpdateScore struct {
	SignerCount     uint32 // number of signers in the header signature aggregate
	SubPeriodIndex  uint32 // signed header's slot modulo params.SyncPeriodLength
	FinalizedHeader bool   // update is considered finalized if has finalized header from the same period and 2/3 signatures
}

// finalized returns true if the update has a header signed by at least 2/3 of
// the committee, referring to a finalized header that refers to the next sync
// committee. This condition is a close approximation of the actual finality
// condition that can only be verified by full beacon nodes.
func (u *UpdateScore) finalized() bool {
	return u.FinalizedHeader && u.SignerCount >= params.SyncCommitteeSupermajority
}

// BetterThan returns true if update u is considered better than w.
func (u UpdateScore) BetterThan(w UpdateScore) bool {
	var (
		uFinalized = u.finalized()
		wFinalized = w.finalized()
	)
	if uFinalized != wFinalized {
		return uFinalized
	}
	return u.SignerCount > w.SignerCount
}
