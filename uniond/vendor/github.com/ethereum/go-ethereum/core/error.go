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

	"github.com/ethereum/go-ethereum/core/types"
)

var (
	// ErrKnownBlock is returned when a block to import is already known locally.
	ErrKnownBlock = errors.New("block already known")

	// ErrBannedHash is returned if a block to import is on the banned list.
	ErrBannedHash = errors.New("banned hash")

	// ErrNoGenesis is returned when there is no Genesis Block.
	ErrNoGenesis = errors.New("genesis not found in chain")

	errSideChainReceipts = errors.New("side blocks can't be accepted as ancient chain data")
)

// List of evm-call-message pre-checking errors. All state transition messages will
// be pre-checked before execution. If any invalidation detected, the corresponding
// error should be returned which is defined here.
//
// - If the pre-checking happens in the miner, then the transaction won't be packed.
// - If the pre-checking happens in the block processing procedure, then a "BAD BLOCk"
// error should be emitted.
var (
	// ErrNonceTooLow is returned if the nonce of a transaction is lower than the
	// one present in the local chain.
	ErrNonceTooLow = errors.New("nonce too low")

	// ErrNonceTooHigh is returned if the nonce of a transaction is higher than the
	// next one expected based on the local chain.
	ErrNonceTooHigh = errors.New("nonce too high")

	// ErrNonceMax is returned if the nonce of a transaction sender account has
	// maximum allowed value and would become invalid if incremented.
	ErrNonceMax = errors.New("nonce has max value")

	// ErrGasLimitReached is returned by the gas pool if the amount of gas required
	// by a transaction is higher than what's left in the block.
	ErrGasLimitReached = errors.New("gas limit reached")

	// ErrInsufficientFundsForTransfer is returned if the transaction sender doesn't
	// have enough funds for transfer(topmost call only).
	ErrInsufficientFundsForTransfer = errors.New("insufficient funds for transfer")

	// ErrMaxInitCodeSizeExceeded is returned if creation transaction provides the init code bigger
	// than init code size limit.
	ErrMaxInitCodeSizeExceeded = errors.New("max initcode size exceeded")

	// ErrInsufficientFunds is returned if the total cost of executing a transaction
	// is higher than the balance of the user's account.
	ErrInsufficientFunds = errors.New("insufficient funds for gas * price + value")

	// ErrGasUintOverflow is returned when calculating gas usage.
	ErrGasUintOverflow = errors.New("gas uint64 overflow")

	// ErrIntrinsicGas is returned if the transaction is specified to use less gas
	// than required to start the invocation.
	ErrIntrinsicGas = errors.New("intrinsic gas too low")

	// ErrTxTypeNotSupported is returned if a transaction is not supported in the
	// current network configuration.
	ErrTxTypeNotSupported = types.ErrTxTypeNotSupported

	// ErrTipAboveFeeCap is a sanity error to ensure no one is able to specify a
	// transaction with a tip higher than the total fee cap.
	ErrTipAboveFeeCap = errors.New("max priority fee per gas higher than max fee per gas")

	// ErrTipVeryHigh is a sanity error to avoid extremely big numbers specified
	// in the tip field.
	ErrTipVeryHigh = errors.New("max priority fee per gas higher than 2^256-1")

	// ErrFeeCapVeryHigh is a sanity error to avoid extremely big numbers specified
	// in the fee cap field.
	ErrFeeCapVeryHigh = errors.New("max fee per gas higher than 2^256-1")

	// ErrFeeCapTooLow is returned if the transaction fee cap is less than the
	// base fee of the block.
	ErrFeeCapTooLow = errors.New("max fee per gas less than block base fee")

	// ErrSenderNoEOA is returned if the sender of a transaction is a contract.
	ErrSenderNoEOA = errors.New("sender not an eoa")

	// ErrBlobFeeCapTooLow is returned if the transaction fee cap is less than the
	// blob gas fee of the block.
	ErrBlobFeeCapTooLow = errors.New("max fee per blob gas less than block blob gas fee")
)
