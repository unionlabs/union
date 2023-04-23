package types

import (
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// IBC client sentinel errors
var (
	ErrClientExists                           = sdkerrors.Register(SubModuleName, 2, "light client already exists")
	ErrInvalidClient                          = sdkerrors.Register(SubModuleName, 3, "light client is invalid")
	ErrClientNotFound                         = sdkerrors.Register(SubModuleName, 4, "light client not found")
	ErrClientFrozen                           = sdkerrors.Register(SubModuleName, 5, "light client is frozen due to misbehaviour")
	ErrInvalidClientMetadata                  = sdkerrors.Register(SubModuleName, 6, "invalid client metadata")
	ErrConsensusStateNotFound                 = sdkerrors.Register(SubModuleName, 7, "consensus state not found")
	ErrInvalidConsensus                       = sdkerrors.Register(SubModuleName, 8, "invalid consensus state")
	ErrClientTypeNotFound                     = sdkerrors.Register(SubModuleName, 9, "client type not found")
	ErrInvalidClientType                      = sdkerrors.Register(SubModuleName, 10, "invalid client type")
	ErrRootNotFound                           = sdkerrors.Register(SubModuleName, 11, "commitment root not found")
	ErrInvalidHeader                          = sdkerrors.Register(SubModuleName, 12, "invalid client header")
	ErrInvalidMisbehaviour                    = sdkerrors.Register(SubModuleName, 13, "invalid light client misbehaviour")
	ErrFailedClientStateVerification          = sdkerrors.Register(SubModuleName, 14, "client state verification failed")
	ErrFailedClientConsensusStateVerification = sdkerrors.Register(SubModuleName, 15, "client consensus state verification failed")
	ErrFailedConnectionStateVerification      = sdkerrors.Register(SubModuleName, 16, "connection state verification failed")
	ErrFailedChannelStateVerification         = sdkerrors.Register(SubModuleName, 17, "channel state verification failed")
	ErrFailedPacketCommitmentVerification     = sdkerrors.Register(SubModuleName, 18, "packet commitment verification failed")
	ErrFailedPacketAckVerification            = sdkerrors.Register(SubModuleName, 19, "packet acknowledgement verification failed")
	ErrFailedPacketReceiptVerification        = sdkerrors.Register(SubModuleName, 20, "packet receipt verification failed")
	ErrFailedNextSeqRecvVerification          = sdkerrors.Register(SubModuleName, 21, "next sequence receive verification failed")
	ErrSelfConsensusStateNotFound             = sdkerrors.Register(SubModuleName, 22, "self consensus state not found")
	ErrUpdateClientFailed                     = sdkerrors.Register(SubModuleName, 23, "unable to update light client")
	ErrInvalidUpdateClientProposal            = sdkerrors.Register(SubModuleName, 24, "invalid update client proposal")
	ErrInvalidUpgradeClient                   = sdkerrors.Register(SubModuleName, 25, "invalid client upgrade")
	ErrInvalidHeight                          = sdkerrors.Register(SubModuleName, 26, "invalid height")
	ErrInvalidSubstitute                      = sdkerrors.Register(SubModuleName, 27, "invalid client state substitute")
	ErrInvalidUpgradeProposal                 = sdkerrors.Register(SubModuleName, 28, "invalid upgrade proposal")
	ErrClientNotActive                        = sdkerrors.Register(SubModuleName, 29, "client state is not active")
)
