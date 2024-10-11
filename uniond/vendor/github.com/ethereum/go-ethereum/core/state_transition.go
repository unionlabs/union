// Copyright 2014 The go-ethereum Authors
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

package core

import (
	"errors"
	"fmt"
	"math"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	cmath "github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/params"
)

// ExecutionResult includes all output after executing given evm
// message no matter the execution itself is successful or not.
type ExecutionResult struct {
	UsedGas    uint64 // Total used gas but include the refunded gas
	Err        error  // Any error encountered during the execution(listed in core/vm/errors.go)
	ReturnData []byte // Returned data from evm(function result or data supplied with revert opcode)
}

// Unwrap returns the internal evm error which allows us for further
// analysis outside.
func (result *ExecutionResult) Unwrap() error {
	return result.Err
}

// Failed returns the indicator whether the execution is successful or not
func (result *ExecutionResult) Failed() bool { return result.Err != nil }

// Return is a helper function to help caller distinguish between revert reason
// and function return. Return returns the data after execution if no error occurs.
func (result *ExecutionResult) Return() []byte {
	if result.Err != nil {
		return nil
	}
	return common.CopyBytes(result.ReturnData)
}

// Revert returns the concrete revert reason if the execution is aborted by `REVERT`
// opcode. Note the reason can be nil if no data supplied with revert opcode.
func (result *ExecutionResult) Revert() []byte {
	if result.Err != vm.ErrExecutionReverted {
		return nil
	}
	return common.CopyBytes(result.ReturnData)
}

// IntrinsicGas computes the 'intrinsic gas' for a message with the given data.
func IntrinsicGas(data []byte, accessList types.AccessList, isContractCreation bool, isHomestead, isEIP2028 bool, isEIP3860 bool) (uint64, error) {
	// Set the starting gas for the raw transaction
	var gas uint64
	if isContractCreation && isHomestead {
		gas = params.TxGasContractCreation
	} else {
		gas = params.TxGas
	}
	dataLen := uint64(len(data))
	// Bump the required gas by the amount of transactional data
	if dataLen > 0 {
		// Zero and non-zero bytes are priced differently
		var nz uint64
		for _, byt := range data {
			if byt != 0 {
				nz++
			}
		}
		// Make sure we don't exceed uint64 for all data combinations
		nonZeroGas := params.TxDataNonZeroGasFrontier
		if isEIP2028 {
			nonZeroGas = params.TxDataNonZeroGasEIP2028
		}
		if (math.MaxUint64-gas)/nonZeroGas < nz {
			return 0, ErrGasUintOverflow
		}
		gas += nz * nonZeroGas

		z := dataLen - nz
		if (math.MaxUint64-gas)/params.TxDataZeroGas < z {
			return 0, ErrGasUintOverflow
		}
		gas += z * params.TxDataZeroGas

		if isContractCreation && isEIP3860 {
			lenWords := toWordSize(dataLen)
			if (math.MaxUint64-gas)/params.InitCodeWordGas < lenWords {
				return 0, ErrGasUintOverflow
			}
			gas += lenWords * params.InitCodeWordGas
		}
	}
	if accessList != nil {
		gas += uint64(len(accessList)) * params.TxAccessListAddressGas
		gas += uint64(accessList.StorageKeys()) * params.TxAccessListStorageKeyGas
	}
	return gas, nil
}

// toWordSize returns the ceiled word size required for init code payment calculation.
func toWordSize(size uint64) uint64 {
	if size > math.MaxUint64-31 {
		return math.MaxUint64/32 + 1
	}

	return (size + 31) / 32
}

// A Message contains the data derived from a single transaction that is relevant to state
// processing.
type Message struct {
	To            *common.Address
	From          common.Address
	Nonce         uint64
	Value         *big.Int
	GasLimit      uint64
	GasPrice      *big.Int
	GasFeeCap     *big.Int
	GasTipCap     *big.Int
	Data          []byte
	AccessList    types.AccessList
	BlobGasFeeCap *big.Int
	BlobHashes    []common.Hash

	// When SkipAccountChecks is true, the message nonce is not checked against the
	// account nonce in state. It also disables checking that the sender is an EOA.
	// This field will be set to true for operations like RPC eth_call.
	SkipAccountChecks bool
}

// TransactionToMessage converts a transaction into a Message.
func TransactionToMessage(tx *types.Transaction, s types.Signer, baseFee *big.Int) (*Message, error) {
	msg := &Message{
		Nonce:             tx.Nonce(),
		GasLimit:          tx.Gas(),
		GasPrice:          new(big.Int).Set(tx.GasPrice()),
		GasFeeCap:         new(big.Int).Set(tx.GasFeeCap()),
		GasTipCap:         new(big.Int).Set(tx.GasTipCap()),
		To:                tx.To(),
		Value:             tx.Value(),
		Data:              tx.Data(),
		AccessList:        tx.AccessList(),
		SkipAccountChecks: false,
		BlobHashes:        tx.BlobHashes(),
		BlobGasFeeCap:     tx.BlobGasFeeCap(),
	}
	// If baseFee provided, set gasPrice to effectiveGasPrice.
	if baseFee != nil {
		msg.GasPrice = cmath.BigMin(msg.GasPrice.Add(msg.GasTipCap, baseFee), msg.GasFeeCap)
	}
	var err error
	msg.From, err = types.Sender(s, tx)
	return msg, err
}

// ApplyMessage computes the new state by applying the given message
// against the old state within the environment.
//
// ApplyMessage returns the bytes returned by any EVM execution (if it took place),
// the gas used (which includes gas refunds) and an error if it failed. An error always
// indicates a core error meaning that the message would always fail for that particular
// state and would never be accepted within a block.
func ApplyMessage(evm *vm.EVM, msg *Message, gp *GasPool) (*ExecutionResult, error) {
	return NewStateTransition(evm, msg, gp).TransitionDb()
}

// StateTransition represents a state transition.
//
// == The State Transitioning Model
//
// A state transition is a change made when a transaction is applied to the current world
// state. The state transitioning model does all the necessary work to work out a valid new
// state root.
//
//  1. Nonce handling
//  2. Pre pay gas
//  3. Create a new state object if the recipient is nil
//  4. Value transfer
//
// == If contract creation ==
//
//	4a. Attempt to run transaction data
//	4b. If valid, use result as code for the new state object
//
// == end ==
//
//  5. Run Script section
//  6. Derive new state root
type StateTransition struct {
	gp           *GasPool
	msg          *Message
	gasRemaining uint64
	initialGas   uint64
	state        vm.StateDB
	evm          *vm.EVM
}

// NewStateTransition initialises and returns a new state transition object.
func NewStateTransition(evm *vm.EVM, msg *Message, gp *GasPool) *StateTransition {
	return &StateTransition{
		gp:    gp,
		evm:   evm,
		msg:   msg,
		state: evm.StateDB,
	}
}

// to returns the recipient of the message.
func (st *StateTransition) to() common.Address {
	if st.msg == nil || st.msg.To == nil /* contract creation */ {
		return common.Address{}
	}
	return *st.msg.To
}

func (st *StateTransition) buyGas() error {
	mgval := new(big.Int).SetUint64(st.msg.GasLimit)
	mgval = mgval.Mul(mgval, st.msg.GasPrice)
	balanceCheck := new(big.Int).Set(mgval)
	if st.msg.GasFeeCap != nil {
		balanceCheck.SetUint64(st.msg.GasLimit)
		balanceCheck = balanceCheck.Mul(balanceCheck, st.msg.GasFeeCap)
		balanceCheck.Add(balanceCheck, st.msg.Value)
	}
	if st.evm.ChainConfig().IsCancun(st.evm.Context.BlockNumber, st.evm.Context.Time) {
		if blobGas := st.blobGasUsed(); blobGas > 0 {
			// Check that the user has enough funds to cover blobGasUsed * tx.BlobGasFeeCap
			blobBalanceCheck := new(big.Int).SetUint64(blobGas)
			blobBalanceCheck.Mul(blobBalanceCheck, st.msg.BlobGasFeeCap)
			balanceCheck.Add(balanceCheck, blobBalanceCheck)
			// Pay for blobGasUsed * actual blob fee
			blobFee := new(big.Int).SetUint64(blobGas)
			blobFee.Mul(blobFee, st.evm.Context.BlobBaseFee)
			mgval.Add(mgval, blobFee)
		}
	}
	if have, want := st.state.GetBalance(st.msg.From), balanceCheck; have.Cmp(want) < 0 {
		return fmt.Errorf("%w: address %v have %v want %v", ErrInsufficientFunds, st.msg.From.Hex(), have, want)
	}
	if err := st.gp.SubGas(st.msg.GasLimit); err != nil {
		return err
	}
	st.gasRemaining += st.msg.GasLimit

	st.initialGas = st.msg.GasLimit
	st.state.SubBalance(st.msg.From, mgval)
	return nil
}

func (st *StateTransition) preCheck() error {
	// Only check transactions that are not fake
	msg := st.msg
	if !msg.SkipAccountChecks {
		// Make sure this transaction's nonce is correct.
		stNonce := st.state.GetNonce(msg.From)
		if msgNonce := msg.Nonce; stNonce < msgNonce {
			return fmt.Errorf("%w: address %v, tx: %d state: %d", ErrNonceTooHigh,
				msg.From.Hex(), msgNonce, stNonce)
		} else if stNonce > msgNonce {
			return fmt.Errorf("%w: address %v, tx: %d state: %d", ErrNonceTooLow,
				msg.From.Hex(), msgNonce, stNonce)
		} else if stNonce+1 < stNonce {
			return fmt.Errorf("%w: address %v, nonce: %d", ErrNonceMax,
				msg.From.Hex(), stNonce)
		}
		// Make sure the sender is an EOA
		codeHash := st.state.GetCodeHash(msg.From)
		if codeHash != (common.Hash{}) && codeHash != types.EmptyCodeHash {
			return fmt.Errorf("%w: address %v, codehash: %s", ErrSenderNoEOA,
				msg.From.Hex(), codeHash)
		}
	}

	// Make sure that transaction gasFeeCap is greater than the baseFee (post london)
	if st.evm.ChainConfig().IsLondon(st.evm.Context.BlockNumber) {
		// Skip the checks if gas fields are zero and baseFee was explicitly disabled (eth_call)
		if !st.evm.Config.NoBaseFee || msg.GasFeeCap.BitLen() > 0 || msg.GasTipCap.BitLen() > 0 {
			if l := msg.GasFeeCap.BitLen(); l > 256 {
				return fmt.Errorf("%w: address %v, maxFeePerGas bit length: %d", ErrFeeCapVeryHigh,
					msg.From.Hex(), l)
			}
			if l := msg.GasTipCap.BitLen(); l > 256 {
				return fmt.Errorf("%w: address %v, maxPriorityFeePerGas bit length: %d", ErrTipVeryHigh,
					msg.From.Hex(), l)
			}
			if msg.GasFeeCap.Cmp(msg.GasTipCap) < 0 {
				return fmt.Errorf("%w: address %v, maxPriorityFeePerGas: %s, maxFeePerGas: %s", ErrTipAboveFeeCap,
					msg.From.Hex(), msg.GasTipCap, msg.GasFeeCap)
			}
			// This will panic if baseFee is nil, but basefee presence is verified
			// as part of header validation.
			if msg.GasFeeCap.Cmp(st.evm.Context.BaseFee) < 0 {
				return fmt.Errorf("%w: address %v, maxFeePerGas: %s baseFee: %s", ErrFeeCapTooLow,
					msg.From.Hex(), msg.GasFeeCap, st.evm.Context.BaseFee)
			}
		}
	}
	// Check the blob version validity
	if msg.BlobHashes != nil {
		if len(msg.BlobHashes) == 0 {
			return errors.New("blob transaction missing blob hashes")
		}
		for i, hash := range msg.BlobHashes {
			if hash[0] != params.BlobTxHashVersion {
				return fmt.Errorf("blob %d hash version mismatch (have %d, supported %d)",
					i, hash[0], params.BlobTxHashVersion)
			}
		}
	}

	if st.evm.ChainConfig().IsCancun(st.evm.Context.BlockNumber, st.evm.Context.Time) {
		if st.blobGasUsed() > 0 {
			// Check that the user is paying at least the current blob fee
			blobFee := st.evm.Context.BlobBaseFee
			if st.msg.BlobGasFeeCap.Cmp(blobFee) < 0 {
				return fmt.Errorf("%w: address %v have %v want %v", ErrBlobFeeCapTooLow, st.msg.From.Hex(), st.msg.BlobGasFeeCap, blobFee)
			}
		}
	}

	return st.buyGas()
}

// TransitionDb will transition the state by applying the current message and
// returning the evm execution result with following fields.
//
//   - used gas: total gas used (including gas being refunded)
//   - returndata: the returned data from evm
//   - concrete execution error: various EVM errors which abort the execution, e.g.
//     ErrOutOfGas, ErrExecutionReverted
//
// However if any consensus issue encountered, return the error directly with
// nil evm execution result.
func (st *StateTransition) TransitionDb() (*ExecutionResult, error) {
	// First check this message satisfies all consensus rules before
	// applying the message. The rules include these clauses
	//
	// 1. the nonce of the message caller is correct
	// 2. caller has enough balance to cover transaction fee(gaslimit * gasprice)
	// 3. the amount of gas required is available in the block
	// 4. the purchased gas is enough to cover intrinsic usage
	// 5. there is no overflow when calculating intrinsic gas
	// 6. caller has enough balance to cover asset transfer for **topmost** call

	// Check clauses 1-3, buy gas if everything is correct
	if err := st.preCheck(); err != nil {
		return nil, err
	}

	if tracer := st.evm.Config.Tracer; tracer != nil {
		tracer.CaptureTxStart(st.initialGas)
		defer func() {
			tracer.CaptureTxEnd(st.gasRemaining)
		}()
	}

	var (
		msg              = st.msg
		sender           = vm.AccountRef(msg.From)
		rules            = st.evm.ChainConfig().Rules(st.evm.Context.BlockNumber, st.evm.Context.Random != nil, st.evm.Context.Time)
		contractCreation = msg.To == nil
	)

	// Check clauses 4-5, subtract intrinsic gas if everything is correct
	gas, err := IntrinsicGas(msg.Data, msg.AccessList, contractCreation, rules.IsHomestead, rules.IsIstanbul, rules.IsShanghai)
	if err != nil {
		return nil, err
	}
	if st.gasRemaining < gas {
		return nil, fmt.Errorf("%w: have %d, want %d", ErrIntrinsicGas, st.gasRemaining, gas)
	}
	st.gasRemaining -= gas

	// Check clause 6
	if msg.Value.Sign() > 0 && !st.evm.Context.CanTransfer(st.state, msg.From, msg.Value) {
		return nil, fmt.Errorf("%w: address %v", ErrInsufficientFundsForTransfer, msg.From.Hex())
	}

	// Check whether the init code size has been exceeded.
	if rules.IsShanghai && contractCreation && len(msg.Data) > params.MaxInitCodeSize {
		return nil, fmt.Errorf("%w: code size %v limit %v", ErrMaxInitCodeSizeExceeded, len(msg.Data), params.MaxInitCodeSize)
	}

	// Execute the preparatory steps for state transition which includes:
	// - prepare accessList(post-berlin)
	// - reset transient storage(eip 1153)
	st.state.Prepare(rules, msg.From, st.evm.Context.Coinbase, msg.To, vm.ActivePrecompiles(rules), msg.AccessList)

	var (
		ret   []byte
		vmerr error // vm errors do not effect consensus and are therefore not assigned to err
	)
	if contractCreation {
		ret, _, st.gasRemaining, vmerr = st.evm.Create(sender, msg.Data, st.gasRemaining, msg.Value)
	} else {
		// Increment the nonce for the next transaction
		st.state.SetNonce(msg.From, st.state.GetNonce(sender.Address())+1)
		ret, st.gasRemaining, vmerr = st.evm.Call(sender, st.to(), msg.Data, st.gasRemaining, msg.Value)
	}

	if !rules.IsLondon {
		// Before EIP-3529: refunds were capped to gasUsed / 2
		st.refundGas(params.RefundQuotient)
	} else {
		// After EIP-3529: refunds are capped to gasUsed / 5
		st.refundGas(params.RefundQuotientEIP3529)
	}
	effectiveTip := msg.GasPrice
	if rules.IsLondon {
		effectiveTip = cmath.BigMin(msg.GasTipCap, new(big.Int).Sub(msg.GasFeeCap, st.evm.Context.BaseFee))
	}

	if st.evm.Config.NoBaseFee && msg.GasFeeCap.Sign() == 0 && msg.GasTipCap.Sign() == 0 {
		// Skip fee payment when NoBaseFee is set and the fee fields
		// are 0. This avoids a negative effectiveTip being applied to
		// the coinbase when simulating calls.
	} else {
		fee := new(big.Int).SetUint64(st.gasUsed())
		fee.Mul(fee, effectiveTip)
		st.state.AddBalance(st.evm.Context.Coinbase, fee)
	}

	return &ExecutionResult{
		UsedGas:    st.gasUsed(),
		Err:        vmerr,
		ReturnData: ret,
	}, nil
}

func (st *StateTransition) refundGas(refundQuotient uint64) {
	// Apply refund counter, capped to a refund quotient
	refund := st.gasUsed() / refundQuotient
	if refund > st.state.GetRefund() {
		refund = st.state.GetRefund()
	}
	st.gasRemaining += refund

	// Return ETH for remaining gas, exchanged at the original rate.
	remaining := new(big.Int).Mul(new(big.Int).SetUint64(st.gasRemaining), st.msg.GasPrice)
	st.state.AddBalance(st.msg.From, remaining)

	// Also return remaining gas to the block gas counter so it is
	// available for the next transaction.
	st.gp.AddGas(st.gasRemaining)
}

// gasUsed returns the amount of gas used up by the state transition.
func (st *StateTransition) gasUsed() uint64 {
	return st.initialGas - st.gasRemaining
}

// blobGasUsed returns the amount of blob gas used by the message.
func (st *StateTransition) blobGasUsed() uint64 {
	return uint64(len(st.msg.BlobHashes) * params.BlobTxBlobGasPerBlob)
}
