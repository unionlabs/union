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

package params

const (
	EpochLength      = 32
	SyncPeriodLength = 8192

	BLSSignatureSize = 96
	BLSPubkeySize    = 48

	SyncCommitteeSize          = 512
	SyncCommitteeBitmaskSize   = SyncCommitteeSize / 8
	SyncCommitteeSupermajority = (SyncCommitteeSize*2 + 2) / 3
)

const (
	StateIndexGenesisTime       = 32
	StateIndexGenesisValidators = 33
	StateIndexForkVersion       = 141
	StateIndexLatestHeader      = 36
	StateIndexBlockRoots        = 37
	StateIndexStateRoots        = 38
	StateIndexHistoricRoots     = 39
	StateIndexFinalBlock        = 105
	StateIndexSyncCommittee     = 54
	StateIndexNextSyncCommittee = 55
	StateIndexExecPayload       = 56
	StateIndexExecHead          = 908
)
