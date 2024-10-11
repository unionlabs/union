// Copyright 2016 The go-ethereum Authors
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

package les

import (
	"github.com/ethereum/go-ethereum/metrics"
	"github.com/ethereum/go-ethereum/p2p"
)

var (
	miscInPacketsMeter           = metrics.NewRegisteredMeter("les/misc/in/packets/total", nil)
	miscInTrafficMeter           = metrics.NewRegisteredMeter("les/misc/in/traffic/total", nil)
	miscInHeaderPacketsMeter     = metrics.NewRegisteredMeter("les/misc/in/packets/header", nil)
	miscInHeaderTrafficMeter     = metrics.NewRegisteredMeter("les/misc/in/traffic/header", nil)
	miscInBodyPacketsMeter       = metrics.NewRegisteredMeter("les/misc/in/packets/body", nil)
	miscInBodyTrafficMeter       = metrics.NewRegisteredMeter("les/misc/in/traffic/body", nil)
	miscInCodePacketsMeter       = metrics.NewRegisteredMeter("les/misc/in/packets/code", nil)
	miscInCodeTrafficMeter       = metrics.NewRegisteredMeter("les/misc/in/traffic/code", nil)
	miscInReceiptPacketsMeter    = metrics.NewRegisteredMeter("les/misc/in/packets/receipt", nil)
	miscInReceiptTrafficMeter    = metrics.NewRegisteredMeter("les/misc/in/traffic/receipt", nil)
	miscInTrieProofPacketsMeter  = metrics.NewRegisteredMeter("les/misc/in/packets/proof", nil)
	miscInTrieProofTrafficMeter  = metrics.NewRegisteredMeter("les/misc/in/traffic/proof", nil)
	miscInHelperTriePacketsMeter = metrics.NewRegisteredMeter("les/misc/in/packets/helperTrie", nil)
	miscInHelperTrieTrafficMeter = metrics.NewRegisteredMeter("les/misc/in/traffic/helperTrie", nil)
	miscInTxsPacketsMeter        = metrics.NewRegisteredMeter("les/misc/in/packets/txs", nil)
	miscInTxsTrafficMeter        = metrics.NewRegisteredMeter("les/misc/in/traffic/txs", nil)
	miscInTxStatusPacketsMeter   = metrics.NewRegisteredMeter("les/misc/in/packets/txStatus", nil)
	miscInTxStatusTrafficMeter   = metrics.NewRegisteredMeter("les/misc/in/traffic/txStatus", nil)

	miscOutPacketsMeter           = metrics.NewRegisteredMeter("les/misc/out/packets/total", nil)
	miscOutTrafficMeter           = metrics.NewRegisteredMeter("les/misc/out/traffic/total", nil)
	miscOutHeaderPacketsMeter     = metrics.NewRegisteredMeter("les/misc/out/packets/header", nil)
	miscOutHeaderTrafficMeter     = metrics.NewRegisteredMeter("les/misc/out/traffic/header", nil)
	miscOutBodyPacketsMeter       = metrics.NewRegisteredMeter("les/misc/out/packets/body", nil)
	miscOutBodyTrafficMeter       = metrics.NewRegisteredMeter("les/misc/out/traffic/body", nil)
	miscOutCodePacketsMeter       = metrics.NewRegisteredMeter("les/misc/out/packets/code", nil)
	miscOutCodeTrafficMeter       = metrics.NewRegisteredMeter("les/misc/out/traffic/code", nil)
	miscOutReceiptPacketsMeter    = metrics.NewRegisteredMeter("les/misc/out/packets/receipt", nil)
	miscOutReceiptTrafficMeter    = metrics.NewRegisteredMeter("les/misc/out/traffic/receipt", nil)
	miscOutTrieProofPacketsMeter  = metrics.NewRegisteredMeter("les/misc/out/packets/proof", nil)
	miscOutTrieProofTrafficMeter  = metrics.NewRegisteredMeter("les/misc/out/traffic/proof", nil)
	miscOutHelperTriePacketsMeter = metrics.NewRegisteredMeter("les/misc/out/packets/helperTrie", nil)
	miscOutHelperTrieTrafficMeter = metrics.NewRegisteredMeter("les/misc/out/traffic/helperTrie", nil)
	miscOutTxsPacketsMeter        = metrics.NewRegisteredMeter("les/misc/out/packets/txs", nil)
	miscOutTxsTrafficMeter        = metrics.NewRegisteredMeter("les/misc/out/traffic/txs", nil)
	miscOutTxStatusPacketsMeter   = metrics.NewRegisteredMeter("les/misc/out/packets/txStatus", nil)
	miscOutTxStatusTrafficMeter   = metrics.NewRegisteredMeter("les/misc/out/traffic/txStatus", nil)

	miscServingTimeHeaderTimer     = metrics.NewRegisteredTimer("les/misc/serve/header", nil)
	miscServingTimeBodyTimer       = metrics.NewRegisteredTimer("les/misc/serve/body", nil)
	miscServingTimeCodeTimer       = metrics.NewRegisteredTimer("les/misc/serve/code", nil)
	miscServingTimeReceiptTimer    = metrics.NewRegisteredTimer("les/misc/serve/receipt", nil)
	miscServingTimeTrieProofTimer  = metrics.NewRegisteredTimer("les/misc/serve/proof", nil)
	miscServingTimeHelperTrieTimer = metrics.NewRegisteredTimer("les/misc/serve/helperTrie", nil)
	miscServingTimeTxTimer         = metrics.NewRegisteredTimer("les/misc/serve/txs", nil)
	miscServingTimeTxStatusTimer   = metrics.NewRegisteredTimer("les/misc/serve/txStatus", nil)

	connectionTimer       = metrics.NewRegisteredTimer("les/connection/duration", nil)
	serverConnectionGauge = metrics.NewRegisteredGauge("les/connection/server", nil)

	totalCapacityGauge   = metrics.NewRegisteredGauge("les/server/totalCapacity", nil)
	totalRechargeGauge   = metrics.NewRegisteredGauge("les/server/totalRecharge", nil)
	blockProcessingTimer = metrics.NewRegisteredTimer("les/server/blockProcessingTime", nil)

	requestServedMeter               = metrics.NewRegisteredMeter("les/server/req/avgServedTime", nil)
	requestServedTimer               = metrics.NewRegisteredTimer("les/server/req/servedTime", nil)
	requestEstimatedMeter            = metrics.NewRegisteredMeter("les/server/req/avgEstimatedTime", nil)
	requestEstimatedTimer            = metrics.NewRegisteredTimer("les/server/req/estimatedTime", nil)
	relativeCostHistogram            = metrics.NewRegisteredHistogram("les/server/req/relative", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostHeaderHistogram      = metrics.NewRegisteredHistogram("les/server/req/relative/header", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostBodyHistogram        = metrics.NewRegisteredHistogram("les/server/req/relative/body", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostReceiptHistogram     = metrics.NewRegisteredHistogram("les/server/req/relative/receipt", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostCodeHistogram        = metrics.NewRegisteredHistogram("les/server/req/relative/code", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostProofHistogram       = metrics.NewRegisteredHistogram("les/server/req/relative/proof", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostHelperProofHistogram = metrics.NewRegisteredHistogram("les/server/req/relative/helperTrie", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostSendTxHistogram      = metrics.NewRegisteredHistogram("les/server/req/relative/txs", nil, metrics.NewExpDecaySample(1028, 0.015))
	relativeCostTxStatusHistogram    = metrics.NewRegisteredHistogram("les/server/req/relative/txStatus", nil, metrics.NewExpDecaySample(1028, 0.015))

	globalFactorGauge    = metrics.NewRegisteredGauge("les/server/globalFactor", nil)
	recentServedGauge    = metrics.NewRegisteredGauge("les/server/recentRequestServed", nil)
	recentEstimatedGauge = metrics.NewRegisteredGauge("les/server/recentRequestEstimated", nil)
	sqServedGauge        = metrics.NewRegisteredGauge("les/server/servingQueue/served", nil)
	sqQueuedGauge        = metrics.NewRegisteredGauge("les/server/servingQueue/queued", nil)

	clientFreezeMeter = metrics.NewRegisteredMeter("les/server/clientEvent/freeze", nil)
	clientErrorMeter  = metrics.NewRegisteredMeter("les/server/clientEvent/error", nil)

	requestRTT       = metrics.NewRegisteredTimer("les/client/req/rtt", nil)
	requestSendDelay = metrics.NewRegisteredTimer("les/client/req/sendDelay", nil)

	serverSelectableGauge = metrics.NewRegisteredGauge("les/client/serverPool/selectable", nil)
	serverDialedMeter     = metrics.NewRegisteredMeter("les/client/serverPool/dialed", nil)
	serverConnectedGauge  = metrics.NewRegisteredGauge("les/client/serverPool/connected", nil)
	sessionValueMeter     = metrics.NewRegisteredMeter("les/client/serverPool/sessionValue", nil)
	totalValueGauge       = metrics.NewRegisteredGauge("les/client/serverPool/totalValue", nil)
	suggestedTimeoutGauge = metrics.NewRegisteredGauge("les/client/serverPool/timeout", nil)
)

// meteredMsgReadWriter is a wrapper around a p2p.MsgReadWriter, capable of
// accumulating the above defined metrics based on the data stream contents.
type meteredMsgReadWriter struct {
	p2p.MsgReadWriter     // Wrapped message stream to meter
	version           int // Protocol version to select correct meters
}

// newMeteredMsgWriter wraps a p2p MsgReadWriter with metering support. If the
// metrics system is disabled, this function returns the original object.
func newMeteredMsgWriter(rw p2p.MsgReadWriter, version int) p2p.MsgReadWriter {
	if !metrics.Enabled {
		return rw
	}
	return &meteredMsgReadWriter{MsgReadWriter: rw, version: version}
}

func (rw *meteredMsgReadWriter) ReadMsg() (p2p.Msg, error) {
	// Read the message and short circuit in case of an error
	msg, err := rw.MsgReadWriter.ReadMsg()
	if err != nil {
		return msg, err
	}
	// Account for the data traffic
	packets, traffic := miscInPacketsMeter, miscInTrafficMeter
	packets.Mark(1)
	traffic.Mark(int64(msg.Size))

	return msg, err
}

func (rw *meteredMsgReadWriter) WriteMsg(msg p2p.Msg) error {
	// Account for the data traffic
	packets, traffic := miscOutPacketsMeter, miscOutTrafficMeter
	packets.Mark(1)
	traffic.Mark(int64(msg.Size))

	// Send the packet to the p2p layer
	return rw.MsgReadWriter.WriteMsg(msg)
}
