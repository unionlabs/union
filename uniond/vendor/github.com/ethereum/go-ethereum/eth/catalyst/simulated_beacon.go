// Copyright 2023 The go-ethereum Authors
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

package catalyst

import (
	"crypto/rand"
	"errors"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/beacon/engine"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/eth"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/rpc"
)

const devEpochLength = 32

// withdrawalQueue implements a FIFO queue which holds withdrawals that are
// pending inclusion.
type withdrawalQueue struct {
	pending chan *types.Withdrawal
}

// add queues a withdrawal for future inclusion.
func (w *withdrawalQueue) add(withdrawal *types.Withdrawal) error {
	select {
	case w.pending <- withdrawal:
		break
	default:
		return errors.New("withdrawal queue full")
	}
	return nil
}

// gatherPending returns a number of queued withdrawals up to a maximum count.
func (w *withdrawalQueue) gatherPending(maxCount int) []*types.Withdrawal {
	withdrawals := []*types.Withdrawal{}
	for {
		select {
		case withdrawal := <-w.pending:
			withdrawals = append(withdrawals, withdrawal)
			if len(withdrawals) == maxCount {
				break
			}
		default:
			return withdrawals
		}
	}
}

type SimulatedBeacon struct {
	shutdownCh  chan struct{}
	eth         *eth.Ethereum
	period      uint64
	withdrawals withdrawalQueue

	feeRecipient     common.Address
	feeRecipientLock sync.Mutex // lock gates concurrent access to the feeRecipient

	engineAPI          *ConsensusAPI
	curForkchoiceState engine.ForkchoiceStateV1
	lastBlockTime      uint64
}

func NewSimulatedBeacon(period uint64, eth *eth.Ethereum) (*SimulatedBeacon, error) {
	chainConfig := eth.APIBackend.ChainConfig()
	if !chainConfig.IsDevMode {
		return nil, errors.New("incompatible pre-existing chain configuration")
	}
	block := eth.BlockChain().CurrentBlock()
	current := engine.ForkchoiceStateV1{
		HeadBlockHash:      block.Hash(),
		SafeBlockHash:      block.Hash(),
		FinalizedBlockHash: block.Hash(),
	}
	engineAPI := newConsensusAPIWithoutHeartbeat(eth)

	// if genesis block, send forkchoiceUpdated to trigger transition to PoS
	if block.Number.Sign() == 0 {
		if _, err := engineAPI.ForkchoiceUpdatedV2(current, nil); err != nil {
			return nil, err
		}
	}
	return &SimulatedBeacon{
		eth:                eth,
		period:             period,
		shutdownCh:         make(chan struct{}),
		engineAPI:          engineAPI,
		lastBlockTime:      block.Time,
		curForkchoiceState: current,
		withdrawals:        withdrawalQueue{make(chan *types.Withdrawal, 20)},
	}, nil
}

func (c *SimulatedBeacon) setFeeRecipient(feeRecipient common.Address) {
	c.feeRecipientLock.Lock()
	c.feeRecipient = feeRecipient
	c.feeRecipientLock.Unlock()
}

// Start invokes the SimulatedBeacon life-cycle function in a goroutine.
func (c *SimulatedBeacon) Start() error {
	if c.period == 0 {
		go c.loopOnDemand()
	} else {
		go c.loop()
	}
	return nil
}

// Stop halts the SimulatedBeacon service.
func (c *SimulatedBeacon) Stop() error {
	close(c.shutdownCh)
	return nil
}

// sealBlock initiates payload building for a new block and creates a new block
// with the completed payload.
func (c *SimulatedBeacon) sealBlock(withdrawals []*types.Withdrawal) error {
	tstamp := uint64(time.Now().Unix())
	if tstamp <= c.lastBlockTime {
		tstamp = c.lastBlockTime + 1
	}
	c.feeRecipientLock.Lock()
	feeRecipient := c.feeRecipient
	c.feeRecipientLock.Unlock()

	// Reset to CurrentBlock in case of the chain was rewound
	if header := c.eth.BlockChain().CurrentBlock(); c.curForkchoiceState.HeadBlockHash != header.Hash() {
		finalizedHash := c.finalizedBlockHash(header.Number.Uint64())
		c.setCurrentState(header.Hash(), *finalizedHash)
	}

	var random [32]byte
	rand.Read(random[:])
	fcResponse, err := c.engineAPI.ForkchoiceUpdatedV2(c.curForkchoiceState, &engine.PayloadAttributes{
		Timestamp:             tstamp,
		SuggestedFeeRecipient: feeRecipient,
		Withdrawals:           withdrawals,
		Random:                random,
	})
	if err != nil {
		return err
	}
	if fcResponse == engine.STATUS_SYNCING {
		return errors.New("chain rewind prevented invocation of payload creation")
	}

	envelope, err := c.engineAPI.getPayload(*fcResponse.PayloadID, true)
	if err != nil {
		return err
	}
	payload := envelope.ExecutionPayload

	var finalizedHash common.Hash
	if payload.Number%devEpochLength == 0 {
		finalizedHash = payload.BlockHash
	} else {
		if fh := c.finalizedBlockHash(payload.Number); fh == nil {
			return errors.New("chain rewind interrupted calculation of finalized block hash")
		} else {
			finalizedHash = *fh
		}
	}

	// Mark the payload as canon
	if _, err = c.engineAPI.NewPayloadV2(*payload); err != nil {
		return err
	}
	c.setCurrentState(payload.BlockHash, finalizedHash)
	// Mark the block containing the payload as canonical
	if _, err = c.engineAPI.ForkchoiceUpdatedV2(c.curForkchoiceState, nil); err != nil {
		return err
	}
	c.lastBlockTime = payload.Timestamp
	return nil
}

// loopOnDemand runs the block production loop for "on-demand" configuration (period = 0)
func (c *SimulatedBeacon) loopOnDemand() {
	var (
		newTxs = make(chan core.NewTxsEvent)
		sub    = c.eth.TxPool().SubscribeTransactions(newTxs, true)
	)
	defer sub.Unsubscribe()

	for {
		select {
		case <-c.shutdownCh:
			return
		case w := <-c.withdrawals.pending:
			withdrawals := append(c.withdrawals.gatherPending(9), w)
			if err := c.sealBlock(withdrawals); err != nil {
				log.Warn("Error performing sealing work", "err", err)
			}
		case <-newTxs:
			withdrawals := c.withdrawals.gatherPending(10)
			if err := c.sealBlock(withdrawals); err != nil {
				log.Warn("Error performing sealing work", "err", err)
			}
		}
	}
}

// loop runs the block production loop for non-zero period configuration
func (c *SimulatedBeacon) loop() {
	timer := time.NewTimer(0)
	for {
		select {
		case <-c.shutdownCh:
			return
		case <-timer.C:
			withdrawals := c.withdrawals.gatherPending(10)
			if err := c.sealBlock(withdrawals); err != nil {
				log.Warn("Error performing sealing work", "err", err)
			} else {
				timer.Reset(time.Second * time.Duration(c.period))
			}
		}
	}
}

// finalizedBlockHash returns the block hash of the finalized block corresponding to the given number
// or nil if doesn't exist in the chain.
func (c *SimulatedBeacon) finalizedBlockHash(number uint64) *common.Hash {
	var finalizedNumber uint64
	if number%devEpochLength == 0 {
		finalizedNumber = number
	} else {
		finalizedNumber = (number - 1) / devEpochLength * devEpochLength
	}

	if finalizedBlock := c.eth.BlockChain().GetBlockByNumber(finalizedNumber); finalizedBlock != nil {
		fh := finalizedBlock.Hash()
		return &fh
	}
	return nil
}

// setCurrentState sets the current forkchoice state
func (c *SimulatedBeacon) setCurrentState(headHash, finalizedHash common.Hash) {
	c.curForkchoiceState = engine.ForkchoiceStateV1{
		HeadBlockHash:      headHash,
		SafeBlockHash:      headHash,
		FinalizedBlockHash: finalizedHash,
	}
}

func RegisterSimulatedBeaconAPIs(stack *node.Node, sim *SimulatedBeacon) {
	stack.RegisterAPIs([]rpc.API{
		{
			Namespace: "dev",
			Service:   &api{sim},
			Version:   "1.0",
		},
	})
}
