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
	"encoding/binary"
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

var (
	errInvalidMessageType  = errors.New("invalid message type")
	errInvalidEntryCount   = errors.New("invalid number of response entries")
	errHeaderUnavailable   = errors.New("header unavailable")
	errTxHashMismatch      = errors.New("transaction hash mismatch")
	errUncleHashMismatch   = errors.New("uncle hash mismatch")
	errReceiptHashMismatch = errors.New("receipt hash mismatch")
	errDataHashMismatch    = errors.New("data hash mismatch")
	errCHTHashMismatch     = errors.New("cht hash mismatch")
	errCHTNumberMismatch   = errors.New("cht number mismatch")
	errUselessNodes        = errors.New("useless nodes in merkle proof nodeset")
)

type LesOdrRequest interface {
	GetCost(*serverPeer) uint64
	CanSend(*serverPeer) bool
	Request(uint64, *serverPeer) error
	Validate(ethdb.Database, *Msg) error
}

func LesRequest(req light.OdrRequest) LesOdrRequest {
	switch r := req.(type) {
	case *light.BlockRequest:
		return (*BlockRequest)(r)
	case *light.ReceiptsRequest:
		return (*ReceiptsRequest)(r)
	case *light.TrieRequest:
		return (*TrieRequest)(r)
	case *light.CodeRequest:
		return (*CodeRequest)(r)
	case *light.ChtRequest:
		return (*ChtRequest)(r)
	case *light.BloomRequest:
		return (*BloomRequest)(r)
	case *light.TxStatusRequest:
		return (*TxStatusRequest)(r)
	default:
		return nil
	}
}

// BlockRequest is the ODR request type for block bodies
type BlockRequest light.BlockRequest

// GetCost returns the cost of the given ODR request according to the serving
// peer's cost table (implementation of LesOdrRequest)
func (r *BlockRequest) GetCost(peer *serverPeer) uint64 {
	return peer.getRequestCost(GetBlockBodiesMsg, 1)
}

// CanSend tells if a certain peer is suitable for serving the given request
func (r *BlockRequest) CanSend(peer *serverPeer) bool {
	return peer.HasBlock(r.Hash, r.Number, false)
}

// Request sends an ODR request to the LES network (implementation of LesOdrRequest)
func (r *BlockRequest) Request(reqID uint64, peer *serverPeer) error {
	peer.Log().Debug("Requesting block body", "hash", r.Hash)
	return peer.requestBodies(reqID, []common.Hash{r.Hash})
}

// Validate processes an ODR request reply message from the LES network
// returns true and stores results in memory if the message was a valid reply
// to the request (implementation of LesOdrRequest)
func (r *BlockRequest) Validate(db ethdb.Database, msg *Msg) error {
	log.Debug("Validating block body", "hash", r.Hash)

	// Ensure we have a correct message with a single block body
	if msg.MsgType != MsgBlockBodies {
		return errInvalidMessageType
	}
	bodies := msg.Obj.([]*types.Body)
	if len(bodies) != 1 {
		return errInvalidEntryCount
	}
	body := bodies[0]

	// Retrieve our stored header and validate block content against it
	if r.Header == nil {
		r.Header = rawdb.ReadHeader(db, r.Hash, r.Number)
	}
	if r.Header == nil {
		return errHeaderUnavailable
	}
	if r.Header.TxHash != types.DeriveSha(types.Transactions(body.Transactions), trie.NewStackTrie(nil)) {
		return errTxHashMismatch
	}
	if r.Header.UncleHash != types.CalcUncleHash(body.Uncles) {
		return errUncleHashMismatch
	}
	// Validations passed, encode and store RLP
	data, err := rlp.EncodeToBytes(body)
	if err != nil {
		return err
	}
	r.Rlp = data
	return nil
}

// ReceiptsRequest is the ODR request type for block receipts by block hash
type ReceiptsRequest light.ReceiptsRequest

// GetCost returns the cost of the given ODR request according to the serving
// peer's cost table (implementation of LesOdrRequest)
func (r *ReceiptsRequest) GetCost(peer *serverPeer) uint64 {
	return peer.getRequestCost(GetReceiptsMsg, 1)
}

// CanSend tells if a certain peer is suitable for serving the given request
func (r *ReceiptsRequest) CanSend(peer *serverPeer) bool {
	return peer.HasBlock(r.Hash, r.Number, false)
}

// Request sends an ODR request to the LES network (implementation of LesOdrRequest)
func (r *ReceiptsRequest) Request(reqID uint64, peer *serverPeer) error {
	peer.Log().Debug("Requesting block receipts", "hash", r.Hash)
	return peer.requestReceipts(reqID, []common.Hash{r.Hash})
}

// Validate processes an ODR request reply message from the LES network
// returns true and stores results in memory if the message was a valid reply
// to the request (implementation of LesOdrRequest)
func (r *ReceiptsRequest) Validate(db ethdb.Database, msg *Msg) error {
	log.Debug("Validating block receipts", "hash", r.Hash)

	// Ensure we have a correct message with a single block receipt
	if msg.MsgType != MsgReceipts {
		return errInvalidMessageType
	}
	receipts := msg.Obj.([]types.Receipts)
	if len(receipts) != 1 {
		return errInvalidEntryCount
	}
	receipt := receipts[0]

	// Retrieve our stored header and validate receipt content against it
	if r.Header == nil {
		r.Header = rawdb.ReadHeader(db, r.Hash, r.Number)
	}
	if r.Header == nil {
		return errHeaderUnavailable
	}
	if r.Header.ReceiptHash != types.DeriveSha(receipt, trie.NewStackTrie(nil)) {
		return errReceiptHashMismatch
	}
	// Validations passed, store and return
	r.Receipts = receipt
	return nil
}

type ProofReq struct {
	BHash               common.Hash
	AccountAddress, Key []byte
	FromLevel           uint
}

// ODR request type for state/storage trie entries, see LesOdrRequest interface
type TrieRequest light.TrieRequest

// GetCost returns the cost of the given ODR request according to the serving
// peer's cost table (implementation of LesOdrRequest)
func (r *TrieRequest) GetCost(peer *serverPeer) uint64 {
	return peer.getRequestCost(GetProofsV2Msg, 1)
}

// CanSend tells if a certain peer is suitable for serving the given request
func (r *TrieRequest) CanSend(peer *serverPeer) bool {
	return peer.HasBlock(r.Id.BlockHash, r.Id.BlockNumber, true)
}

// Request sends an ODR request to the LES network (implementation of LesOdrRequest)
func (r *TrieRequest) Request(reqID uint64, peer *serverPeer) error {
	peer.Log().Debug("Requesting trie proof", "root", r.Id.Root, "key", r.Key)
	req := ProofReq{
		BHash:          r.Id.BlockHash,
		AccountAddress: r.Id.AccountAddress,
		Key:            r.Key,
	}
	return peer.requestProofs(reqID, []ProofReq{req})
}

// Validate processes an ODR request reply message from the LES network
// returns true and stores results in memory if the message was a valid reply
// to the request (implementation of LesOdrRequest)
func (r *TrieRequest) Validate(db ethdb.Database, msg *Msg) error {
	log.Debug("Validating trie proof", "root", r.Id.Root, "key", r.Key)

	if msg.MsgType != MsgProofsV2 {
		return errInvalidMessageType
	}
	proofs := msg.Obj.(trienode.ProofList)
	// Verify the proof and store if checks out
	nodeSet := proofs.Set()
	reads := &readTraceDB{db: nodeSet}
	if _, err := trie.VerifyProof(r.Id.Root, r.Key, reads); err != nil {
		return fmt.Errorf("merkle proof verification failed: %v", err)
	}
	// check if all nodes have been read by VerifyProof
	if len(reads.reads) != nodeSet.KeyCount() {
		return errUselessNodes
	}
	r.Proof = nodeSet
	return nil
}

type CodeReq struct {
	BHash          common.Hash
	AccountAddress []byte
}

// CodeRequest is the ODR request type for node data (used for retrieving contract code), see LesOdrRequest interface
type CodeRequest light.CodeRequest

// GetCost returns the cost of the given ODR request according to the serving
// peer's cost table (implementation of LesOdrRequest)
func (r *CodeRequest) GetCost(peer *serverPeer) uint64 {
	return peer.getRequestCost(GetCodeMsg, 1)
}

// CanSend tells if a certain peer is suitable for serving the given request
func (r *CodeRequest) CanSend(peer *serverPeer) bool {
	return peer.HasBlock(r.Id.BlockHash, r.Id.BlockNumber, true)
}

// Request sends an ODR request to the LES network (implementation of LesOdrRequest)
func (r *CodeRequest) Request(reqID uint64, peer *serverPeer) error {
	peer.Log().Debug("Requesting code data", "hash", r.Hash)
	req := CodeReq{
		BHash:          r.Id.BlockHash,
		AccountAddress: r.Id.AccountAddress,
	}
	return peer.requestCode(reqID, []CodeReq{req})
}

// Validate processes an ODR request reply message from the LES network
// returns true and stores results in memory if the message was a valid reply
// to the request (implementation of LesOdrRequest)
func (r *CodeRequest) Validate(db ethdb.Database, msg *Msg) error {
	log.Debug("Validating code data", "hash", r.Hash)

	// Ensure we have a correct message with a single code element
	if msg.MsgType != MsgCode {
		return errInvalidMessageType
	}
	reply := msg.Obj.([][]byte)
	if len(reply) != 1 {
		return errInvalidEntryCount
	}
	data := reply[0]

	// Verify the data and store if checks out
	if hash := crypto.Keccak256Hash(data); r.Hash != hash {
		return errDataHashMismatch
	}
	r.Data = data
	return nil
}

const (
	// helper trie type constants
	htCanonical = iota // Canonical hash trie
	htBloomBits        // BloomBits trie

	// helper trie auxiliary types
	// htAuxNone = 1 ; deprecated number, used in les2/3 previously.
	htAuxHeader = 2 // applicable for htCanonical, requests for relevant headers
)

type HelperTrieReq struct {
	Type              uint
	TrieIdx           uint64
	Key               []byte
	FromLevel, AuxReq uint
}

type HelperTrieResps struct { // describes all responses, not just a single one
	Proofs  trienode.ProofList
	AuxData [][]byte
}

// ChtRequest is the ODR request type for requesting headers by Canonical Hash Trie, see LesOdrRequest interface
type ChtRequest light.ChtRequest

// GetCost returns the cost of the given ODR request according to the serving
// peer's cost table (implementation of LesOdrRequest)
func (r *ChtRequest) GetCost(peer *serverPeer) uint64 {
	return peer.getRequestCost(GetHelperTrieProofsMsg, 1)
}

// CanSend tells if a certain peer is suitable for serving the given request
func (r *ChtRequest) CanSend(peer *serverPeer) bool {
	peer.lock.RLock()
	defer peer.lock.RUnlock()

	return peer.headInfo.Number >= r.Config.ChtConfirms && r.ChtNum <= (peer.headInfo.Number-r.Config.ChtConfirms)/r.Config.ChtSize
}

// Request sends an ODR request to the LES network (implementation of LesOdrRequest)
func (r *ChtRequest) Request(reqID uint64, peer *serverPeer) error {
	peer.Log().Debug("Requesting CHT", "cht", r.ChtNum, "block", r.BlockNum)
	var encNum [8]byte
	binary.BigEndian.PutUint64(encNum[:], r.BlockNum)
	req := HelperTrieReq{
		Type:    htCanonical,
		TrieIdx: r.ChtNum,
		Key:     encNum[:],
		AuxReq:  htAuxHeader,
	}
	return peer.requestHelperTrieProofs(reqID, []HelperTrieReq{req})
}

// Validate processes an ODR request reply message from the LES network
// returns true and stores results in memory if the message was a valid reply
// to the request (implementation of LesOdrRequest)
func (r *ChtRequest) Validate(db ethdb.Database, msg *Msg) error {
	log.Debug("Validating CHT", "cht", r.ChtNum, "block", r.BlockNum)

	if msg.MsgType != MsgHelperTrieProofs {
		return errInvalidMessageType
	}
	resp := msg.Obj.(HelperTrieResps)
	if len(resp.AuxData) != 1 {
		return errInvalidEntryCount
	}
	nodeSet := resp.Proofs.Set()
	headerEnc := resp.AuxData[0]
	if len(headerEnc) == 0 {
		return errHeaderUnavailable
	}
	header := new(types.Header)
	if err := rlp.DecodeBytes(headerEnc, header); err != nil {
		return errHeaderUnavailable
	}
	// Verify the CHT
	var (
		node      light.ChtNode
		encNumber [8]byte
	)
	binary.BigEndian.PutUint64(encNumber[:], r.BlockNum)

	reads := &readTraceDB{db: nodeSet}
	value, err := trie.VerifyProof(r.ChtRoot, encNumber[:], reads)
	if err != nil {
		return fmt.Errorf("merkle proof verification failed: %v", err)
	}
	if len(reads.reads) != nodeSet.KeyCount() {
		return errUselessNodes
	}
	if err := rlp.DecodeBytes(value, &node); err != nil {
		return err
	}
	if node.Hash != header.Hash() {
		return errCHTHashMismatch
	}
	if r.BlockNum != header.Number.Uint64() {
		return errCHTNumberMismatch
	}
	// Verifications passed, store and return
	r.Header = header
	r.Proof = nodeSet
	r.Td = node.Td
	return nil
}

type BloomReq struct {
	BloomTrieNum, BitIdx, SectionIndex, FromLevel uint64
}

// BloomRequest is the ODR request type for requesting headers by Canonical Hash Trie, see LesOdrRequest interface
type BloomRequest light.BloomRequest

// GetCost returns the cost of the given ODR request according to the serving
// peer's cost table (implementation of LesOdrRequest)
func (r *BloomRequest) GetCost(peer *serverPeer) uint64 {
	return peer.getRequestCost(GetHelperTrieProofsMsg, len(r.SectionIndexList))
}

// CanSend tells if a certain peer is suitable for serving the given request
func (r *BloomRequest) CanSend(peer *serverPeer) bool {
	peer.lock.RLock()
	defer peer.lock.RUnlock()

	if peer.version < lpv2 {
		return false
	}
	return peer.headInfo.Number >= r.Config.BloomTrieConfirms && r.BloomTrieNum <= (peer.headInfo.Number-r.Config.BloomTrieConfirms)/r.Config.BloomTrieSize
}

// Request sends an ODR request to the LES network (implementation of LesOdrRequest)
func (r *BloomRequest) Request(reqID uint64, peer *serverPeer) error {
	peer.Log().Debug("Requesting BloomBits", "bloomTrie", r.BloomTrieNum, "bitIdx", r.BitIdx, "sections", r.SectionIndexList)
	reqs := make([]HelperTrieReq, len(r.SectionIndexList))

	var encNumber [10]byte
	binary.BigEndian.PutUint16(encNumber[:2], uint16(r.BitIdx))

	for i, sectionIdx := range r.SectionIndexList {
		binary.BigEndian.PutUint64(encNumber[2:], sectionIdx)
		reqs[i] = HelperTrieReq{
			Type:    htBloomBits,
			TrieIdx: r.BloomTrieNum,
			Key:     common.CopyBytes(encNumber[:]),
		}
	}
	return peer.requestHelperTrieProofs(reqID, reqs)
}

// Validate processes an ODR request reply message from the LES network
// returns true and stores results in memory if the message was a valid reply
// to the request (implementation of LesOdrRequest)
func (r *BloomRequest) Validate(db ethdb.Database, msg *Msg) error {
	log.Debug("Validating BloomBits", "bloomTrie", r.BloomTrieNum, "bitIdx", r.BitIdx, "sections", r.SectionIndexList)

	// Ensure we have a correct message with a single proof element
	if msg.MsgType != MsgHelperTrieProofs {
		return errInvalidMessageType
	}
	resps := msg.Obj.(HelperTrieResps)
	proofs := resps.Proofs
	nodeSet := proofs.Set()
	reads := &readTraceDB{db: nodeSet}

	r.BloomBits = make([][]byte, len(r.SectionIndexList))

	// Verify the proofs
	var encNumber [10]byte
	binary.BigEndian.PutUint16(encNumber[:2], uint16(r.BitIdx))

	for i, idx := range r.SectionIndexList {
		binary.BigEndian.PutUint64(encNumber[2:], idx)
		value, err := trie.VerifyProof(r.BloomTrieRoot, encNumber[:], reads)
		if err != nil {
			return err
		}
		r.BloomBits[i] = value
	}

	if len(reads.reads) != nodeSet.KeyCount() {
		return errUselessNodes
	}
	r.Proofs = nodeSet
	return nil
}

// TxStatusRequest is the ODR request type for transaction status
type TxStatusRequest light.TxStatusRequest

// GetCost returns the cost of the given ODR request according to the serving
// peer's cost table (implementation of LesOdrRequest)
func (r *TxStatusRequest) GetCost(peer *serverPeer) uint64 {
	return peer.getRequestCost(GetTxStatusMsg, len(r.Hashes))
}

// CanSend tells if a certain peer is suitable for serving the given request
func (r *TxStatusRequest) CanSend(peer *serverPeer) bool {
	return peer.txHistory != txIndexDisabled
}

// Request sends an ODR request to the LES network (implementation of LesOdrRequest)
func (r *TxStatusRequest) Request(reqID uint64, peer *serverPeer) error {
	peer.Log().Debug("Requesting transaction status", "count", len(r.Hashes))
	return peer.requestTxStatus(reqID, r.Hashes)
}

// Validate processes an ODR request reply message from the LES network
// returns true and stores results in memory if the message was a valid reply
// to the request (implementation of LesOdrRequest)
func (r *TxStatusRequest) Validate(db ethdb.Database, msg *Msg) error {
	log.Debug("Validating transaction status", "count", len(r.Hashes))

	if msg.MsgType != MsgTxStatus {
		return errInvalidMessageType
	}
	status := msg.Obj.([]light.TxStatus)
	if len(status) != len(r.Hashes) {
		return errInvalidEntryCount
	}
	r.Status = status
	return nil
}

// readTraceDB stores the keys of database reads. We use this to check that received node
// sets contain only the trie nodes necessary to make proofs pass.
type readTraceDB struct {
	db    ethdb.KeyValueReader
	reads map[string]struct{}
}

// Get returns a stored node
func (db *readTraceDB) Get(k []byte) ([]byte, error) {
	if db.reads == nil {
		db.reads = make(map[string]struct{})
	}
	db.reads[string(k)] = struct{}{}
	return db.db.Get(k)
}

// Has returns true if the node set contains the given key
func (db *readTraceDB) Has(key []byte) (bool, error) {
	_, err := db.Get(key)
	return err == nil, nil
}
