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

package les

import (
	"bytes"
	"encoding/binary"
	"io"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/txpool/legacypool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/crypto"
	l "github.com/ethereum/go-ethereum/les"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

var (
	bankKey, _ = crypto.HexToECDSA("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")
	bankAddr   = crypto.PubkeyToAddress(bankKey.PublicKey)
	bankFunds  = new(big.Int).Mul(big.NewInt(100), big.NewInt(params.Ether))

	testChainLen     = 256
	testContractCode = common.Hex2Bytes("606060405260cc8060106000396000f360606040526000357c01000000000000000000000000000000000000000000000000000000009004806360cd2685146041578063c16431b914606b57603f565b005b6055600480803590602001909190505060a9565b6040518082815260200191505060405180910390f35b60886004808035906020019091908035906020019091905050608a565b005b80600060005083606481101560025790900160005b50819055505b5050565b6000600060005082606481101560025790900160005b5054905060c7565b91905056")

	chain     *core.BlockChain
	addresses []common.Address
	txHashes  []common.Hash

	chtTrie   *trie.Trie
	bloomTrie *trie.Trie
	chtKeys   [][]byte
	bloomKeys [][]byte
)

func makechain() (bc *core.BlockChain, addresses []common.Address, txHashes []common.Hash) {
	gspec := &core.Genesis{
		Config:   params.TestChainConfig,
		Alloc:    core.GenesisAlloc{bankAddr: {Balance: bankFunds}},
		GasLimit: 100000000,
	}
	signer := types.HomesteadSigner{}
	_, blocks, _ := core.GenerateChainWithGenesis(gspec, ethash.NewFaker(), testChainLen,
		func(i int, gen *core.BlockGen) {
			var (
				tx   *types.Transaction
				addr common.Address
			)
			nonce := uint64(i)
			if i%4 == 0 {
				tx, _ = types.SignTx(types.NewContractCreation(nonce, big.NewInt(0), 200000, big.NewInt(params.GWei), testContractCode), signer, bankKey)
				addr = crypto.CreateAddress(bankAddr, nonce)
			} else {
				addr = common.BigToAddress(big.NewInt(int64(i)))
				tx, _ = types.SignTx(types.NewTransaction(nonce, addr, big.NewInt(10000), params.TxGas, big.NewInt(params.GWei), nil), signer, bankKey)
			}
			gen.AddTx(tx)
			addresses = append(addresses, addr)
			txHashes = append(txHashes, tx.Hash())
		})
	bc, _ = core.NewBlockChain(rawdb.NewMemoryDatabase(), nil, gspec, nil, ethash.NewFaker(), vm.Config{}, nil, nil)
	if _, err := bc.InsertChain(blocks); err != nil {
		panic(err)
	}
	return
}

func makeTries() (chtTrie *trie.Trie, bloomTrie *trie.Trie, chtKeys, bloomKeys [][]byte) {
	chtTrie = trie.NewEmpty(trie.NewDatabase(rawdb.NewMemoryDatabase(), trie.HashDefaults))
	bloomTrie = trie.NewEmpty(trie.NewDatabase(rawdb.NewMemoryDatabase(), trie.HashDefaults))
	for i := 0; i < testChainLen; i++ {
		// The element in CHT is <big-endian block number> -> <block hash>
		key := make([]byte, 8)
		binary.BigEndian.PutUint64(key, uint64(i+1))
		chtTrie.MustUpdate(key, []byte{0x1, 0xf})
		chtKeys = append(chtKeys, key)

		// The element in Bloom trie is <2 byte bit index> + <big-endian block number> -> bloom
		key2 := make([]byte, 10)
		binary.BigEndian.PutUint64(key2[2:], uint64(i+1))
		bloomTrie.MustUpdate(key2, []byte{0x2, 0xe})
		bloomKeys = append(bloomKeys, key2)
	}
	return
}

func init() {
	chain, addresses, txHashes = makechain()
	chtTrie, bloomTrie, chtKeys, bloomKeys = makeTries()
}

type fuzzer struct {
	chain *core.BlockChain
	pool  *txpool.TxPool

	chainLen  int
	addresses []common.Address
	txs       []common.Hash
	nonce     uint64

	chtKeys   [][]byte
	bloomKeys [][]byte
	chtTrie   *trie.Trie
	bloomTrie *trie.Trie

	input     io.Reader
	exhausted bool
}

func newFuzzer(input []byte) *fuzzer {
	pool := legacypool.New(legacypool.DefaultConfig, chain)
	txpool, _ := txpool.New(new(big.Int).SetUint64(legacypool.DefaultConfig.PriceLimit), chain, []txpool.SubPool{pool})

	return &fuzzer{
		chain:     chain,
		chainLen:  testChainLen,
		addresses: addresses,
		txs:       txHashes,
		chtTrie:   chtTrie,
		bloomTrie: bloomTrie,
		chtKeys:   chtKeys,
		bloomKeys: bloomKeys,
		nonce:     uint64(len(txHashes)),
		pool:      txpool,
		input:     bytes.NewReader(input),
	}
}

func (f *fuzzer) read(size int) []byte {
	out := make([]byte, size)
	if _, err := f.input.Read(out); err != nil {
		f.exhausted = true
	}
	return out
}

func (f *fuzzer) randomByte() byte {
	d := f.read(1)
	return d[0]
}

func (f *fuzzer) randomBool() bool {
	d := f.read(1)
	return d[0]&1 == 1
}

func (f *fuzzer) randomInt(max int) int {
	if max == 0 {
		return 0
	}
	if max <= 256 {
		return int(f.randomByte()) % max
	}
	var a uint16
	if err := binary.Read(f.input, binary.LittleEndian, &a); err != nil {
		f.exhausted = true
	}
	return int(a % uint16(max))
}

func (f *fuzzer) randomX(max int) uint64 {
	var a uint16
	if err := binary.Read(f.input, binary.LittleEndian, &a); err != nil {
		f.exhausted = true
	}
	if a < 0x8000 {
		return uint64(a%uint16(max+1)) - 1
	}
	return (uint64(1)<<(a%64+1) - 1) & (uint64(a) * 343897772345826595)
}

func (f *fuzzer) randomBlockHash() common.Hash {
	h := f.chain.GetCanonicalHash(uint64(f.randomInt(3 * f.chainLen)))
	if h != (common.Hash{}) {
		return h
	}
	return common.BytesToHash(f.read(common.HashLength))
}

func (f *fuzzer) randomAddress() []byte {
	i := f.randomInt(3 * len(f.addresses))
	if i < len(f.addresses) {
		return f.addresses[i].Bytes()
	}
	return f.read(common.AddressLength)
}

func (f *fuzzer) randomCHTTrieKey() []byte {
	i := f.randomInt(3 * len(f.chtKeys))
	if i < len(f.chtKeys) {
		return f.chtKeys[i]
	}
	return f.read(8)
}

func (f *fuzzer) randomBloomTrieKey() []byte {
	i := f.randomInt(3 * len(f.bloomKeys))
	if i < len(f.bloomKeys) {
		return f.bloomKeys[i]
	}
	return f.read(10)
}

func (f *fuzzer) randomTxHash() common.Hash {
	i := f.randomInt(3 * len(f.txs))
	if i < len(f.txs) {
		return f.txs[i]
	}
	return common.BytesToHash(f.read(common.HashLength))
}

func (f *fuzzer) BlockChain() *core.BlockChain {
	return f.chain
}

func (f *fuzzer) TxPool() *txpool.TxPool {
	return f.pool
}

func (f *fuzzer) ArchiveMode() bool {
	return false
}

func (f *fuzzer) AddTxsSync() bool {
	return false
}

func (f *fuzzer) GetHelperTrie(typ uint, index uint64) *trie.Trie {
	if typ == 0 {
		return f.chtTrie
	} else if typ == 1 {
		return f.bloomTrie
	}
	return nil
}

type dummyMsg struct {
	data []byte
}

func (d dummyMsg) Decode(val interface{}) error {
	return rlp.DecodeBytes(d.data, val)
}

func (f *fuzzer) doFuzz(msgCode uint64, packet interface{}) {
	enc, err := rlp.EncodeToBytes(packet)
	if err != nil {
		panic(err)
	}
	version := f.randomInt(3) + 2 // [LES2, LES3, LES4]
	peer, closeFn := l.NewFuzzerPeer(version)
	defer closeFn()
	fn, _, _, err := l.Les3[msgCode].Handle(dummyMsg{enc})
	if err != nil {
		panic(err)
	}
	fn(f, peer, func() bool { return true })
}

func fuzz(input []byte) int {
	// We expect some large inputs
	if len(input) < 100 {
		return -1
	}
	f := newFuzzer(input)
	if f.exhausted {
		return -1
	}
	for !f.exhausted {
		switch f.randomInt(8) {
		case 0:
			req := &l.GetBlockHeadersPacket{
				Query: l.GetBlockHeadersData{
					Amount:  f.randomX(l.MaxHeaderFetch + 1),
					Skip:    f.randomX(10),
					Reverse: f.randomBool(),
				},
			}
			if f.randomBool() {
				req.Query.Origin.Hash = f.randomBlockHash()
			} else {
				req.Query.Origin.Number = uint64(f.randomInt(f.chainLen * 2))
			}
			f.doFuzz(l.GetBlockHeadersMsg, req)

		case 1:
			req := &l.GetBlockBodiesPacket{Hashes: make([]common.Hash, f.randomInt(l.MaxBodyFetch+1))}
			for i := range req.Hashes {
				req.Hashes[i] = f.randomBlockHash()
			}
			f.doFuzz(l.GetBlockBodiesMsg, req)

		case 2:
			req := &l.GetCodePacket{Reqs: make([]l.CodeReq, f.randomInt(l.MaxCodeFetch+1))}
			for i := range req.Reqs {
				req.Reqs[i] = l.CodeReq{
					BHash:          f.randomBlockHash(),
					AccountAddress: f.randomAddress(),
				}
			}
			f.doFuzz(l.GetCodeMsg, req)

		case 3:
			req := &l.GetReceiptsPacket{Hashes: make([]common.Hash, f.randomInt(l.MaxReceiptFetch+1))}
			for i := range req.Hashes {
				req.Hashes[i] = f.randomBlockHash()
			}
			f.doFuzz(l.GetReceiptsMsg, req)

		case 4:
			req := &l.GetProofsPacket{Reqs: make([]l.ProofReq, f.randomInt(l.MaxProofsFetch+1))}
			for i := range req.Reqs {
				if f.randomBool() {
					req.Reqs[i] = l.ProofReq{
						BHash:          f.randomBlockHash(),
						AccountAddress: f.randomAddress(),
						Key:            f.randomAddress(),
						FromLevel:      uint(f.randomX(3)),
					}
				} else {
					req.Reqs[i] = l.ProofReq{
						BHash:     f.randomBlockHash(),
						Key:       f.randomAddress(),
						FromLevel: uint(f.randomX(3)),
					}
				}
			}
			f.doFuzz(l.GetProofsV2Msg, req)

		case 5:
			req := &l.GetHelperTrieProofsPacket{Reqs: make([]l.HelperTrieReq, f.randomInt(l.MaxHelperTrieProofsFetch+1))}
			for i := range req.Reqs {
				switch f.randomInt(3) {
				case 0:
					// Canonical hash trie
					req.Reqs[i] = l.HelperTrieReq{
						Type:      0,
						TrieIdx:   f.randomX(3),
						Key:       f.randomCHTTrieKey(),
						FromLevel: uint(f.randomX(3)),
						AuxReq:    uint(2),
					}
				case 1:
					// Bloom trie
					req.Reqs[i] = l.HelperTrieReq{
						Type:      1,
						TrieIdx:   f.randomX(3),
						Key:       f.randomBloomTrieKey(),
						FromLevel: uint(f.randomX(3)),
						AuxReq:    0,
					}
				default:
					// Random trie
					req.Reqs[i] = l.HelperTrieReq{
						Type:      2,
						TrieIdx:   f.randomX(3),
						Key:       f.randomCHTTrieKey(),
						FromLevel: uint(f.randomX(3)),
						AuxReq:    0,
					}
				}
			}
			f.doFuzz(l.GetHelperTrieProofsMsg, req)

		case 6:
			req := &l.SendTxPacket{Txs: make([]*types.Transaction, f.randomInt(l.MaxTxSend+1))}
			signer := types.HomesteadSigner{}
			for i := range req.Txs {
				var nonce uint64
				if f.randomBool() {
					nonce = uint64(f.randomByte())
				} else {
					nonce = f.nonce
					f.nonce += 1
				}
				req.Txs[i], _ = types.SignTx(types.NewTransaction(nonce, common.Address{}, big.NewInt(10000), params.TxGas, big.NewInt(1000000000*int64(f.randomByte())), nil), signer, bankKey)
			}
			f.doFuzz(l.SendTxV2Msg, req)

		case 7:
			req := &l.GetTxStatusPacket{Hashes: make([]common.Hash, f.randomInt(l.MaxTxStatus+1))}
			for i := range req.Hashes {
				req.Hashes[i] = f.randomTxHash()
			}
			f.doFuzz(l.GetTxStatusMsg, req)
		}
	}
	return 0
}
