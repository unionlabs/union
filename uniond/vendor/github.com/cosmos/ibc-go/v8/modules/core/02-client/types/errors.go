package types

import (
	errorsmod "cosmossdk.io/errors"
)

// IBC client sentinel errors
var (
	ErrClientExists                           = errorsmod.Register(SubModuleName, 2, "light client already exists")
	ErrInvalidClient                          = errorsmod.Register(SubModuleName, 3, "light client is invalid")
	ErrClientNotFound                         = errorsmod.Register(SubModuleName, 4, "light client not found")
	ErrClientFrozen                           = errorsmod.Register(SubModuleName, 5, "light client is frozen due to misbehaviour")
	ErrInvalidClientMetadata                  = errorsmod.Register(SubModuleName, 6, "invalid client metadata")
	ErrConsensusStateNotFound                 = errorsmod.Register(SubModuleName, 7, "consensus state not found")
	ErrInvalidConsensus                       = errorsmod.Register(SubModuleName, 8, "invalid consensus state")
	ErrClientTypeNotFound                     = errorsmod.Register(SubModuleName, 9, "client type not found")
	ErrInvalidClientType                      = errorsmod.Register(SubModuleName, 10, "invalid client type")
	ErrRootNotFound                           = errorsmod.Register(SubModuleName, 11, "commitment root not found")
	ErrInvalidHeader                          = errorsmod.Register(SubModuleName, 12, "invalid client header")
	ErrInvalidMisbehaviour                    = errorsmod.Register(SubModuleName, 13, "invalid light client misbehaviour")
	ErrFailedClientStateVerification          = errorsmod.Register(SubModuleName, 14, "client state verification failed")
	ErrFailedClientConsensusStateVerification = errorsmod.Register(SubModuleName, 15, "client consensus state verification failed")
	ErrFailedConnectionStateVerification      = errorsmod.Register(SubModuleName, 16, "connection state verification failed")
	ErrFailedChannelStateVerification         = errorsmod.Register(SubModuleName, 17, "channel state verification failed")
	ErrFailedPacketCommitmentVerification     = errorsmod.Register(SubModuleName, 18, "packet commitment verification failed")
	ErrFailedPacketAckVerification            = errorsmod.Register(SubModuleName, 19, "packet acknowledgement verification failed")
	ErrFailedPacketReceiptVerification        = errorsmod.Register(SubModuleName, 20, "packet receipt verification failed")
	ErrFailedNextSeqRecvVerification          = errorsmod.Register(SubModuleName, 21, "next sequence receive verification failed")
	ErrSelfConsensusStateNotFound             = errorsmod.Register(SubModuleName, 22, "self consensus state not found")
	ErrUpdateClientFailed                     = errorsmod.Register(SubModuleName, 23, "unable to update light client")
	ErrInvalidRecoveryClient                  = errorsmod.Register(SubModuleName, 24, "invalid recovery client")
	ErrInvalidUpgradeClient                   = errorsmod.Register(SubModuleName, 25, "invalid client upgrade")
	ErrInvalidHeight                          = errorsmod.Register(SubModuleName, 26, "invalid height")
	ErrInvalidSubstitute                      = errorsmod.Register(SubModuleName, 27, "invalid client state substitute")
	ErrInvalidUpgradeProposal                 = errorsmod.Register(SubModuleName, 28, "invalid upgrade proposal")
	ErrClientNotActive                        = errorsmod.Register(SubModuleName, 29, "client state is not active")
	ErrFailedMembershipVerification           = errorsmod.Register(SubModuleName, 30, "membership verification failed")
	ErrFailedNonMembershipVerification        = errorsmod.Register(SubModuleName, 31, "non-membership verification failed")
)
