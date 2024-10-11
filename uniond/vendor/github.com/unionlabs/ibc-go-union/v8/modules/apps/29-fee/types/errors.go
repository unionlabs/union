package types

import (
	errorsmod "cosmossdk.io/errors"
)

// 29-fee sentinel errors
var (
	ErrInvalidVersion                = errorsmod.Register(ModuleName, 2, "invalid ICS29 middleware version")
	ErrRefundAccNotFound             = errorsmod.Register(ModuleName, 3, "no account found for given refund address")
	ErrBalanceNotFound               = errorsmod.Register(ModuleName, 4, "balance not found for given account address")
	ErrFeeNotFound                   = errorsmod.Register(ModuleName, 5, "there is no fee escrowed for the given packetID")
	ErrRelayersNotEmpty              = errorsmod.Register(ModuleName, 6, "relayers must not be set. This feature is not supported")
	ErrCounterpartyPayeeEmpty        = errorsmod.Register(ModuleName, 7, "counterparty payee must not be empty")
	ErrForwardRelayerAddressNotFound = errorsmod.Register(ModuleName, 8, "forward relayer address not found")
	ErrFeeNotEnabled                 = errorsmod.Register(ModuleName, 9, "fee module is not enabled for this channel. If this error occurs after channel setup, fee module may not be enabled")
	ErrRelayerNotFoundForAsyncAck    = errorsmod.Register(ModuleName, 10, "relayer address must be stored for async WriteAcknowledgement")
	ErrFeeModuleLocked               = errorsmod.Register(ModuleName, 11, "the fee module is currently locked, a severe bug has been detected")
	ErrUnsupportedAction             = errorsmod.Register(ModuleName, 12, "unsupported action")
)
