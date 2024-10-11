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

// Package types implements a few types of the beacon chain for light client usage.
package types

import (
	"crypto/sha256"
	"encoding/binary"

	"github.com/ethereum/go-ethereum/beacon/merkle"
	"github.com/ethereum/go-ethereum/beacon/params"
	"github.com/ethereum/go-ethereum/common"
)

//go:generate go run github.com/fjl/gencodec -type Header -field-override headerMarshaling -out gen_header_json.go

const (
	headerIndexSlot          = 8
	headerIndexProposerIndex = 9
	headerIndexParentRoot    = 10
	headerIndexStateRoot     = 11
	headerIndexBodyRoot      = 12
)

// Header defines a beacon header.
//
// See data structure definition here:
// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblockheader
type Header struct {
	// Monotonically increasing slot number for the beacon block (may be gapped)
	Slot uint64 `gencodec:"required" json:"slot"`

	// Index into the validator table who created the beacon block
	ProposerIndex uint64 `gencodec:"required" json:"proposer_index"`

	// SSZ hash of the parent beacon header
	ParentRoot common.Hash `gencodec:"required" json:"parent_root"`

	// SSZ hash of the beacon state (https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beacon-state)
	StateRoot common.Hash `gencodec:"required" json:"state_root"`

	// SSZ hash of the beacon block body (https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beaconblockbody)
	BodyRoot common.Hash `gencodec:"required" json:"body_root"`
}

// headerMarshaling is a field type overrides for gencodec.
type headerMarshaling struct {
	Slot          common.Decimal
	ProposerIndex common.Decimal
}

// Hash calculates the block root of the header.
//
// TODO(zsfelfoldi): Remove this when an SSZ encoder lands.
func (h *Header) Hash() common.Hash {
	var values [16]merkle.Value // values corresponding to indices 8 to 15 of the beacon header tree
	binary.LittleEndian.PutUint64(values[headerIndexSlot][:8], h.Slot)
	binary.LittleEndian.PutUint64(values[headerIndexProposerIndex][:8], h.ProposerIndex)
	values[headerIndexParentRoot] = merkle.Value(h.ParentRoot)
	values[headerIndexStateRoot] = merkle.Value(h.StateRoot)
	values[headerIndexBodyRoot] = merkle.Value(h.BodyRoot)
	hasher := sha256.New()
	for i := 7; i > 0; i-- {
		hasher.Reset()
		hasher.Write(values[i*2][:])
		hasher.Write(values[i*2+1][:])
		hasher.Sum(values[i][:0])
	}
	return common.Hash(values[1])
}

// Epoch returns the epoch the header belongs to.
func (h *Header) Epoch() uint64 {
	return h.Slot / params.EpochLength
}

// SyncPeriod returns the sync period the header belongs to.
func (h *Header) SyncPeriod() uint64 {
	return SyncPeriod(h.Slot)
}

// SyncPeriodStart returns the first slot of the given period.
func SyncPeriodStart(period uint64) uint64 {
	return period * params.SyncPeriodLength
}

// SyncPeriod returns the sync period that the given slot belongs to.
func SyncPeriod(slot uint64) uint64 {
	return slot / params.SyncPeriodLength
}

// SignedHeader represents a beacon header signed by a sync committee.
//
// This structure is created from either an optimistic update or an instant update:
//   - https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientoptimisticupdate
//   - https://github.com/zsfelfoldi/beacon-APIs/blob/instant_update/apis/beacon/light_client/instant_update.yaml
type SignedHeader struct {
	// Beacon header being signed
	Header Header

	// Sync committee BLS signature aggregate
	Signature SyncAggregate

	// Slot in which the signature has been created (newer than Header.Slot,
	// determines the signing sync committee)
	SignatureSlot uint64
}
