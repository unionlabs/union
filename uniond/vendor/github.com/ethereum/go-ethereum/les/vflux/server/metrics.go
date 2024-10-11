// Copyright 2021 The go-ethereum Authors
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

package server

import (
	"github.com/ethereum/go-ethereum/metrics"
)

var (
	totalActiveCapacityGauge = metrics.NewRegisteredGauge("vflux/server/active/capacity", nil)
	totalActiveCountGauge    = metrics.NewRegisteredGauge("vflux/server/active/count", nil)
	totalInactiveCountGauge  = metrics.NewRegisteredGauge("vflux/server/inactive/count", nil)

	clientConnectedMeter    = metrics.NewRegisteredMeter("vflux/server/clientEvent/connected", nil)
	clientActivatedMeter    = metrics.NewRegisteredMeter("vflux/server/clientEvent/activated", nil)
	clientDeactivatedMeter  = metrics.NewRegisteredMeter("vflux/server/clientEvent/deactivated", nil)
	clientDisconnectedMeter = metrics.NewRegisteredMeter("vflux/server/clientEvent/disconnected", nil)

	capacityQueryZeroMeter    = metrics.NewRegisteredMeter("vflux/server/capQueryZero", nil)
	capacityQueryNonZeroMeter = metrics.NewRegisteredMeter("vflux/server/capQueryNonZero", nil)
)
