// Copyright 2021 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package t8ntool

import (
	"crypto/ecdsa"
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"os"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/consensus/clique"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/urfave/cli/v2"
)

//go:generate go run github.com/fjl/gencodec -type header -field-override headerMarshaling -out gen_header.go
type header struct {
	ParentHash            common.Hash       `json:"parentHash"`
	OmmerHash             *common.Hash      `json:"sha3Uncles"`
	Coinbase              *common.Address   `json:"miner"`
	Root                  common.Hash       `json:"stateRoot"        gencodec:"required"`
	TxHash                *common.Hash      `json:"transactionsRoot"`
	ReceiptHash           *common.Hash      `json:"receiptsRoot"`
	Bloom                 types.Bloom       `json:"logsBloom"`
	Difficulty            *big.Int          `json:"difficulty"`
	Number                *big.Int          `json:"number"           gencodec:"required"`
	GasLimit              uint64            `json:"gasLimit"         gencodec:"required"`
	GasUsed               uint64            `json:"gasUsed"`
	Time                  uint64            `json:"timestamp"        gencodec:"required"`
	Extra                 []byte            `json:"extraData"`
	MixDigest             common.Hash       `json:"mixHash"`
	Nonce                 *types.BlockNonce `json:"nonce"`
	BaseFee               *big.Int          `json:"baseFeePerGas" rlp:"optional"`
	WithdrawalsHash       *common.Hash      `json:"withdrawalsRoot" rlp:"optional"`
	BlobGasUsed           *uint64           `json:"blobGasUsed"   rlp:"optional"`
	ExcessBlobGas         *uint64           `json:"excessBlobGas"   rlp:"optional"`
	ParentBeaconBlockRoot *common.Hash      `json:"parentBeaconBlockRoot" rlp:"optional"`
}

type headerMarshaling struct {
	Difficulty    *math.HexOrDecimal256
	Number        *math.HexOrDecimal256
	GasLimit      math.HexOrDecimal64
	GasUsed       math.HexOrDecimal64
	Time          math.HexOrDecimal64
	Extra         hexutil.Bytes
	BaseFee       *math.HexOrDecimal256
	BlobGasUsed   *math.HexOrDecimal64
	ExcessBlobGas *math.HexOrDecimal64
}

type bbInput struct {
	Header      *header             `json:"header,omitempty"`
	OmmersRlp   []string            `json:"ommers,omitempty"`
	TxRlp       string              `json:"txs,omitempty"`
	Withdrawals []*types.Withdrawal `json:"withdrawals,omitempty"`
	Clique      *cliqueInput        `json:"clique,omitempty"`

	Ethash bool                 `json:"-"`
	Txs    []*types.Transaction `json:"-"`
	Ommers []*types.Header      `json:"-"`
}

type cliqueInput struct {
	Key       *ecdsa.PrivateKey
	Voted     *common.Address
	Authorize *bool
	Vanity    common.Hash
}

// UnmarshalJSON implements json.Unmarshaler interface.
func (c *cliqueInput) UnmarshalJSON(input []byte) error {
	var x struct {
		Key       *common.Hash    `json:"secretKey"`
		Voted     *common.Address `json:"voted"`
		Authorize *bool           `json:"authorize"`
		Vanity    common.Hash     `json:"vanity"`
	}
	if err := json.Unmarshal(input, &x); err != nil {
		return err
	}
	if x.Key == nil {
		return errors.New("missing required field 'secretKey' for cliqueInput")
	}
	if ecdsaKey, err := crypto.ToECDSA(x.Key[:]); err != nil {
		return err
	} else {
		c.Key = ecdsaKey
	}
	c.Voted = x.Voted
	c.Authorize = x.Authorize
	c.Vanity = x.Vanity
	return nil
}

// ToBlock converts i into a *types.Block
func (i *bbInput) ToBlock() *types.Block {
	header := &types.Header{
		ParentHash:       i.Header.ParentHash,
		UncleHash:        types.EmptyUncleHash,
		Coinbase:         common.Address{},
		Root:             i.Header.Root,
		TxHash:           types.EmptyTxsHash,
		ReceiptHash:      types.EmptyReceiptsHash,
		Bloom:            i.Header.Bloom,
		Difficulty:       common.Big0,
		Number:           i.Header.Number,
		GasLimit:         i.Header.GasLimit,
		GasUsed:          i.Header.GasUsed,
		Time:             i.Header.Time,
		Extra:            i.Header.Extra,
		MixDigest:        i.Header.MixDigest,
		BaseFee:          i.Header.BaseFee,
		WithdrawalsHash:  i.Header.WithdrawalsHash,
		BlobGasUsed:      i.Header.BlobGasUsed,
		ExcessBlobGas:    i.Header.ExcessBlobGas,
		ParentBeaconRoot: i.Header.ParentBeaconBlockRoot,
	}

	// Fill optional values.
	if i.Header.OmmerHash != nil {
		header.UncleHash = *i.Header.OmmerHash
	} else if len(i.Ommers) != 0 {
		// Calculate the ommer hash if none is provided and there are ommers to hash
		header.UncleHash = types.CalcUncleHash(i.Ommers)
	}
	if i.Header.Coinbase != nil {
		header.Coinbase = *i.Header.Coinbase
	}
	if i.Header.TxHash != nil {
		header.TxHash = *i.Header.TxHash
	}
	if i.Header.ReceiptHash != nil {
		header.ReceiptHash = *i.Header.ReceiptHash
	}
	if i.Header.Nonce != nil {
		header.Nonce = *i.Header.Nonce
	}
	if i.Header.Difficulty != nil {
		header.Difficulty = i.Header.Difficulty
	}
	return types.NewBlockWithHeader(header).WithBody(i.Txs, i.Ommers).WithWithdrawals(i.Withdrawals)
}

// SealBlock seals the given block using the configured engine.
func (i *bbInput) SealBlock(block *types.Block) (*types.Block, error) {
	switch {
	case i.Clique != nil:
		return i.sealClique(block)
	default:
		return block, nil
	}
}

// sealClique seals the given block using clique.
func (i *bbInput) sealClique(block *types.Block) (*types.Block, error) {
	// If any clique value overwrites an explicit header value, fail
	// to avoid silently building a block with unexpected values.
	if i.Header.Extra != nil {
		return nil, NewError(ErrorConfig, errors.New("sealing with clique will overwrite provided extra data"))
	}
	header := block.Header()
	if i.Clique.Voted != nil {
		if i.Header.Coinbase != nil {
			return nil, NewError(ErrorConfig, errors.New("sealing with clique and voting will overwrite provided coinbase"))
		}
		header.Coinbase = *i.Clique.Voted
	}
	if i.Clique.Authorize != nil {
		if i.Header.Nonce != nil {
			return nil, NewError(ErrorConfig, errors.New("sealing with clique and voting will overwrite provided nonce"))
		}
		if *i.Clique.Authorize {
			header.Nonce = [8]byte{}
		} else {
			header.Nonce = [8]byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff}
		}
	}
	// Extra is fixed 32 byte vanity and 65 byte signature
	header.Extra = make([]byte, 32+65)
	copy(header.Extra[0:32], i.Clique.Vanity.Bytes()[:])

	// Sign the seal hash and fill in the rest of the extra data
	h := clique.SealHash(header)
	sighash, err := crypto.Sign(h[:], i.Clique.Key)
	if err != nil {
		return nil, err
	}
	copy(header.Extra[32:], sighash)
	block = block.WithSeal(header)
	return block, nil
}

// BuildBlock constructs a block from the given inputs.
func BuildBlock(ctx *cli.Context) error {
	// Configure the go-ethereum logger
	glogger := log.NewGlogHandler(log.StreamHandler(os.Stderr, log.TerminalFormat(false)))
	glogger.Verbosity(log.Lvl(ctx.Int(VerbosityFlag.Name)))
	log.Root().SetHandler(glogger)

	baseDir, err := createBasedir(ctx)
	if err != nil {
		return NewError(ErrorIO, fmt.Errorf("failed creating output basedir: %v", err))
	}
	inputData, err := readInput(ctx)
	if err != nil {
		return err
	}
	block := inputData.ToBlock()
	block, err = inputData.SealBlock(block)
	if err != nil {
		return err
	}
	return dispatchBlock(ctx, baseDir, block)
}

func readInput(ctx *cli.Context) (*bbInput, error) {
	var (
		headerStr      = ctx.String(InputHeaderFlag.Name)
		ommersStr      = ctx.String(InputOmmersFlag.Name)
		withdrawalsStr = ctx.String(InputWithdrawalsFlag.Name)
		txsStr         = ctx.String(InputTxsRlpFlag.Name)
		cliqueStr      = ctx.String(SealCliqueFlag.Name)
		inputData      = &bbInput{}
	)
	if headerStr == stdinSelector || ommersStr == stdinSelector || txsStr == stdinSelector || cliqueStr == stdinSelector {
		decoder := json.NewDecoder(os.Stdin)
		if err := decoder.Decode(inputData); err != nil {
			return nil, NewError(ErrorJson, fmt.Errorf("failed unmarshaling stdin: %v", err))
		}
	}
	if cliqueStr != stdinSelector && cliqueStr != "" {
		var clique cliqueInput
		if err := readFile(cliqueStr, "clique", &clique); err != nil {
			return nil, err
		}
		inputData.Clique = &clique
	}
	if headerStr != stdinSelector {
		var env header
		if err := readFile(headerStr, "header", &env); err != nil {
			return nil, err
		}
		inputData.Header = &env
	}
	if ommersStr != stdinSelector && ommersStr != "" {
		var ommers []string
		if err := readFile(ommersStr, "ommers", &ommers); err != nil {
			return nil, err
		}
		inputData.OmmersRlp = ommers
	}
	if withdrawalsStr != stdinSelector && withdrawalsStr != "" {
		var withdrawals []*types.Withdrawal
		if err := readFile(withdrawalsStr, "withdrawals", &withdrawals); err != nil {
			return nil, err
		}
		inputData.Withdrawals = withdrawals
	}
	if txsStr != stdinSelector {
		var txs string
		if err := readFile(txsStr, "txs", &txs); err != nil {
			return nil, err
		}
		inputData.TxRlp = txs
	}
	// Deserialize rlp txs and ommers
	var (
		ommers = []*types.Header{}
		txs    = []*types.Transaction{}
	)
	if inputData.TxRlp != "" {
		if err := rlp.DecodeBytes(common.FromHex(inputData.TxRlp), &txs); err != nil {
			return nil, NewError(ErrorRlp, fmt.Errorf("unable to decode transaction from rlp data: %v", err))
		}
		inputData.Txs = txs
	}
	for _, str := range inputData.OmmersRlp {
		type extblock struct {
			Header *types.Header
			Txs    []*types.Transaction
			Ommers []*types.Header
		}
		var ommer *extblock
		if err := rlp.DecodeBytes(common.FromHex(str), &ommer); err != nil {
			return nil, NewError(ErrorRlp, fmt.Errorf("unable to decode ommer from rlp data: %v", err))
		}
		ommers = append(ommers, ommer.Header)
	}
	inputData.Ommers = ommers

	return inputData, nil
}

// dispatchBlock writes the output data to either stderr or stdout, or to the specified
// files
func dispatchBlock(ctx *cli.Context, baseDir string, block *types.Block) error {
	raw, _ := rlp.EncodeToBytes(block)
	type blockInfo struct {
		Rlp  hexutil.Bytes `json:"rlp"`
		Hash common.Hash   `json:"hash"`
	}
	enc := blockInfo{
		Rlp:  raw,
		Hash: block.Hash(),
	}
	b, err := json.MarshalIndent(enc, "", "  ")
	if err != nil {
		return NewError(ErrorJson, fmt.Errorf("failed marshalling output: %v", err))
	}
	switch dest := ctx.String(OutputBlockFlag.Name); dest {
	case "stdout":
		os.Stdout.Write(b)
		os.Stdout.WriteString("\n")
	case "stderr":
		os.Stderr.Write(b)
		os.Stderr.WriteString("\n")
	default:
		if err := saveFile(baseDir, dest, enc); err != nil {
			return err
		}
	}
	return nil
}
