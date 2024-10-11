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
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>

package pathdb

import "github.com/ethereum/go-ethereum/metrics"

var (
	cleanHitMeter   = metrics.NewRegisteredMeter("pathdb/clean/hit", nil)
	cleanMissMeter  = metrics.NewRegisteredMeter("pathdb/clean/miss", nil)
	cleanReadMeter  = metrics.NewRegisteredMeter("pathdb/clean/read", nil)
	cleanWriteMeter = metrics.NewRegisteredMeter("pathdb/clean/write", nil)

	dirtyHitMeter         = metrics.NewRegisteredMeter("pathdb/dirty/hit", nil)
	dirtyMissMeter        = metrics.NewRegisteredMeter("pathdb/dirty/miss", nil)
	dirtyReadMeter        = metrics.NewRegisteredMeter("pathdb/dirty/read", nil)
	dirtyWriteMeter       = metrics.NewRegisteredMeter("pathdb/dirty/write", nil)
	dirtyNodeHitDepthHist = metrics.NewRegisteredHistogram("pathdb/dirty/depth", nil, metrics.NewExpDecaySample(1028, 0.015))

	cleanFalseMeter = metrics.NewRegisteredMeter("pathdb/clean/false", nil)
	dirtyFalseMeter = metrics.NewRegisteredMeter("pathdb/dirty/false", nil)
	diskFalseMeter  = metrics.NewRegisteredMeter("pathdb/disk/false", nil)

	commitTimeTimer  = metrics.NewRegisteredTimer("pathdb/commit/time", nil)
	commitNodesMeter = metrics.NewRegisteredMeter("pathdb/commit/nodes", nil)
	commitBytesMeter = metrics.NewRegisteredMeter("pathdb/commit/bytes", nil)

	gcNodesMeter = metrics.NewRegisteredMeter("pathdb/gc/nodes", nil)
	gcBytesMeter = metrics.NewRegisteredMeter("pathdb/gc/bytes", nil)

	diffLayerBytesMeter = metrics.NewRegisteredMeter("pathdb/diff/bytes", nil)
	diffLayerNodesMeter = metrics.NewRegisteredMeter("pathdb/diff/nodes", nil)

	historyBuildTimeMeter  = metrics.NewRegisteredTimer("pathdb/history/time", nil)
	historyDataBytesMeter  = metrics.NewRegisteredMeter("pathdb/history/bytes/data", nil)
	historyIndexBytesMeter = metrics.NewRegisteredMeter("pathdb/history/bytes/index", nil)
)
