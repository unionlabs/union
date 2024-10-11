// Copyright 2019 The go-ethereum Authors
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
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core/forkid"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

// clientHandler is responsible for receiving and processing all incoming server
// responses.
type clientHandler struct {
	forkFilter forkid.Filter
	backend    *LightEthereum

	closeCh chan struct{}
	wg      sync.WaitGroup // WaitGroup used to track all connected peers.
}

func newClientHandler(backend *LightEthereum) *clientHandler {
	handler := &clientHandler{
		forkFilter: forkid.NewFilter(backend.blockchain),
		backend:    backend,
		closeCh:    make(chan struct{}),
	}
	return handler
}

func (h *clientHandler) stop() {
	close(h.closeCh)
	h.wg.Wait()
}

// runPeer is the p2p protocol run function for the given version.
func (h *clientHandler) runPeer(version uint, p *p2p.Peer, rw p2p.MsgReadWriter) error {
	peer := newServerPeer(int(version), h.backend.config.NetworkId, false, p, newMeteredMsgWriter(rw, int(version)))
	defer peer.close()
	h.wg.Add(1)
	defer h.wg.Done()
	err := h.handle(peer, false)
	return err
}

func (h *clientHandler) handle(p *serverPeer, noInitAnnounce bool) error {
	if h.backend.peers.len() >= h.backend.config.LightPeers && !p.Peer.Info().Network.Trusted {
		return p2p.DiscTooManyPeers
	}
	p.Log().Debug("Light Ethereum peer connected", "name", p.Name())

	// Execute the LES handshake
	forkid := forkid.NewID(h.backend.blockchain.Config(), h.backend.BlockChain().Genesis(), h.backend.blockchain.CurrentHeader().Number.Uint64(), h.backend.blockchain.CurrentHeader().Time)
	if err := p.Handshake(h.backend.blockchain.Genesis().Hash(), forkid, h.forkFilter); err != nil {
		p.Log().Debug("Light Ethereum handshake failed", "err", err)
		return err
	}
	// Register peer with the server pool
	if h.backend.serverPool != nil {
		if nvt, err := h.backend.serverPool.RegisterNode(p.Node()); err == nil {
			p.setValueTracker(nvt)
			p.updateVtParams()
			defer func() {
				p.setValueTracker(nil)
				h.backend.serverPool.UnregisterNode(p.Node())
			}()
		} else {
			return err
		}
	}
	// Register the peer locally
	if err := h.backend.peers.register(p); err != nil {
		p.Log().Error("Light Ethereum peer registration failed", "err", err)
		return err
	}

	serverConnectionGauge.Update(int64(h.backend.peers.len()))

	connectedAt := mclock.Now()
	defer func() {
		h.backend.peers.unregister(p.id)
		connectionTimer.Update(time.Duration(mclock.Now() - connectedAt))
		serverConnectionGauge.Update(int64(h.backend.peers.len()))
	}()

	// Mark the peer starts to be served.
	p.serving.Store(true)
	defer p.serving.Store(false)

	// Spawn a main loop to handle all incoming messages.
	for {
		if err := h.handleMsg(p); err != nil {
			p.Log().Debug("Light Ethereum message handling failed", "err", err)
			p.fcServer.DumpLogs()
			return err
		}
	}
}

// handleMsg is invoked whenever an inbound message is received from a remote
// peer. The remote connection is torn down upon returning any error.
func (h *clientHandler) handleMsg(p *serverPeer) error {
	// Read the next message from the remote peer, and ensure it's fully consumed
	msg, err := p.rw.ReadMsg()
	if err != nil {
		return err
	}
	p.Log().Trace("Light Ethereum message arrived", "code", msg.Code, "bytes", msg.Size)

	if msg.Size > ProtocolMaxMsgSize {
		return errResp(ErrMsgTooLarge, "%v > %v", msg.Size, ProtocolMaxMsgSize)
	}
	defer msg.Discard()

	var deliverMsg *Msg

	// Handle the message depending on its contents
	switch {
	case msg.Code == AnnounceMsg:
		p.Log().Trace("Received announce message")
		var req announceData
		if err := msg.Decode(&req); err != nil {
			return errResp(ErrDecode, "%v: %v", msg, err)
		}
		if err := req.sanityCheck(); err != nil {
			return err
		}
		update, size := req.Update.decode()
		if p.rejectUpdate(size) {
			return errResp(ErrRequestRejected, "")
		}
		p.updateFlowControl(update)
		p.updateVtParams()

		if req.Hash != (common.Hash{}) {
			if p.announceType == announceTypeNone {
				return errResp(ErrUnexpectedResponse, "")
			}
			if p.announceType == announceTypeSigned {
				if err := req.checkSignature(p.ID(), update); err != nil {
					p.Log().Trace("Invalid announcement signature", "err", err)
					return err
				}
				p.Log().Trace("Valid announcement signature")
			}
			p.Log().Trace("Announce message content", "number", req.Number, "hash", req.Hash, "td", req.Td, "reorg", req.ReorgDepth)

			// Update peer head information first and then notify the announcement
			p.updateHead(req.Hash, req.Number, req.Td)
		}
	case msg.Code == BlockHeadersMsg:
		p.Log().Trace("Received block header response message")
		var resp struct {
			ReqID, BV uint64
			Headers   []*types.Header
		}
		if err := msg.Decode(&resp); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ReceivedReply(resp.ReqID, resp.BV)
		p.answeredRequest(resp.ReqID)

		deliverMsg = &Msg{
			MsgType: MsgBlockHeaders,
			ReqID:   resp.ReqID,
			Obj:     resp.Headers,
		}
	case msg.Code == BlockBodiesMsg:
		p.Log().Trace("Received block bodies response")
		var resp struct {
			ReqID, BV uint64
			Data      []*types.Body
		}
		if err := msg.Decode(&resp); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ReceivedReply(resp.ReqID, resp.BV)
		p.answeredRequest(resp.ReqID)
		deliverMsg = &Msg{
			MsgType: MsgBlockBodies,
			ReqID:   resp.ReqID,
			Obj:     resp.Data,
		}
	case msg.Code == CodeMsg:
		p.Log().Trace("Received code response")
		var resp struct {
			ReqID, BV uint64
			Data      [][]byte
		}
		if err := msg.Decode(&resp); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ReceivedReply(resp.ReqID, resp.BV)
		p.answeredRequest(resp.ReqID)
		deliverMsg = &Msg{
			MsgType: MsgCode,
			ReqID:   resp.ReqID,
			Obj:     resp.Data,
		}
	case msg.Code == ReceiptsMsg:
		p.Log().Trace("Received receipts response")
		var resp struct {
			ReqID, BV uint64
			Receipts  []types.Receipts
		}
		if err := msg.Decode(&resp); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ReceivedReply(resp.ReqID, resp.BV)
		p.answeredRequest(resp.ReqID)
		deliverMsg = &Msg{
			MsgType: MsgReceipts,
			ReqID:   resp.ReqID,
			Obj:     resp.Receipts,
		}
	case msg.Code == ProofsV2Msg:
		p.Log().Trace("Received les/2 proofs response")
		var resp struct {
			ReqID, BV uint64
			Data      trienode.ProofList
		}
		if err := msg.Decode(&resp); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ReceivedReply(resp.ReqID, resp.BV)
		p.answeredRequest(resp.ReqID)
		deliverMsg = &Msg{
			MsgType: MsgProofsV2,
			ReqID:   resp.ReqID,
			Obj:     resp.Data,
		}
	case msg.Code == HelperTrieProofsMsg:
		p.Log().Trace("Received helper trie proof response")
		var resp struct {
			ReqID, BV uint64
			Data      HelperTrieResps
		}
		if err := msg.Decode(&resp); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ReceivedReply(resp.ReqID, resp.BV)
		p.answeredRequest(resp.ReqID)
		deliverMsg = &Msg{
			MsgType: MsgHelperTrieProofs,
			ReqID:   resp.ReqID,
			Obj:     resp.Data,
		}
	case msg.Code == TxStatusMsg:
		p.Log().Trace("Received tx status response")
		var resp struct {
			ReqID, BV uint64
			Status    []light.TxStatus
		}
		if err := msg.Decode(&resp); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ReceivedReply(resp.ReqID, resp.BV)
		p.answeredRequest(resp.ReqID)
		deliverMsg = &Msg{
			MsgType: MsgTxStatus,
			ReqID:   resp.ReqID,
			Obj:     resp.Status,
		}
	case msg.Code == StopMsg && p.version >= lpv3:
		p.freeze()
		h.backend.retriever.frozen(p)
		p.Log().Debug("Service stopped")
	case msg.Code == ResumeMsg && p.version >= lpv3:
		var bv uint64
		if err := msg.Decode(&bv); err != nil {
			return errResp(ErrDecode, "msg %v: %v", msg, err)
		}
		p.fcServer.ResumeFreeze(bv)
		p.unfreeze()
		p.Log().Debug("Service resumed")
	default:
		p.Log().Trace("Received invalid message", "code", msg.Code)
		return errResp(ErrInvalidMsgCode, "%v", msg.Code)
	}
	// Deliver the received response to retriever.
	if deliverMsg != nil {
		if err := h.backend.retriever.deliver(p, deliverMsg); err != nil {
			if val := p.errCount.Add(1, mclock.Now()); val > maxResponseErrors {
				return err
			}
		}
	}
	return nil
}
