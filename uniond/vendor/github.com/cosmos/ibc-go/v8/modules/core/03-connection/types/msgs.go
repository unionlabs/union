package types

import (
	errorsmod "cosmossdk.io/errors"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ sdk.Msg = (*MsgConnectionOpenInit)(nil)
	_ sdk.Msg = (*MsgConnectionOpenConfirm)(nil)
	_ sdk.Msg = (*MsgConnectionOpenAck)(nil)
	_ sdk.Msg = (*MsgConnectionOpenTry)(nil)
	_ sdk.Msg = (*MsgUpdateParams)(nil)

	_ sdk.HasValidateBasic = (*MsgConnectionOpenInit)(nil)
	_ sdk.HasValidateBasic = (*MsgConnectionOpenConfirm)(nil)
	_ sdk.HasValidateBasic = (*MsgConnectionOpenAck)(nil)
	_ sdk.HasValidateBasic = (*MsgConnectionOpenTry)(nil)
	_ sdk.HasValidateBasic = (*MsgUpdateParams)(nil)

	_ codectypes.UnpackInterfacesMessage = (*MsgConnectionOpenTry)(nil)
	_ codectypes.UnpackInterfacesMessage = (*MsgConnectionOpenAck)(nil)
)

// NewMsgConnectionOpenInit creates a new MsgConnectionOpenInit instance. It sets the
// counterparty connection identifier to be empty.
func NewMsgConnectionOpenInit(
	clientID, counterpartyClientID string,
	counterpartyPrefix commitmenttypes.MerklePrefix,
	version *Version, delayPeriod uint64, signer string,
) *MsgConnectionOpenInit {
	// counterparty must have the same delay period
	counterparty := NewCounterparty(counterpartyClientID, "", counterpartyPrefix)
	return &MsgConnectionOpenInit{
		ClientId:     clientID,
		Counterparty: counterparty,
		Version:      version,
		DelayPeriod:  delayPeriod,
		Signer:       signer,
	}
}

// ValidateBasic implements sdk.Msg.
func (msg MsgConnectionOpenInit) ValidateBasic() error {
	if msg.ClientId == exported.LocalhostClientID {
		return errorsmod.Wrap(clienttypes.ErrInvalidClientType, "localhost connection handshakes are disallowed")
	}

	if err := host.ClientIdentifierValidator(msg.ClientId); err != nil {
		return errorsmod.Wrap(err, "invalid client ID")
	}
	if msg.Counterparty.ConnectionId != "" {
		return errorsmod.Wrap(ErrInvalidCounterparty, "counterparty connection identifier must be empty")
	}

	// NOTE: Version can be nil on MsgConnectionOpenInit
	if msg.Version != nil {
		if err := ValidateVersion(msg.Version); err != nil {
			return errorsmod.Wrap(err, "basic validation of the provided version failed")
		}
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Counterparty.ValidateBasic()
}

// GetSigners implements sdk.Msg
func (msg MsgConnectionOpenInit) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// NewMsgConnectionOpenTry creates a new MsgConnectionOpenTry instance
func NewMsgConnectionOpenTry(
	clientID, counterpartyConnectionID, counterpartyClientID string,
	counterpartyClient exported.ClientState,
	counterpartyPrefix commitmenttypes.MerklePrefix,
	counterpartyVersions []*Version, delayPeriod uint64,
	initProof, clientProof, consensusProof []byte,
	proofHeight, consensusHeight clienttypes.Height, signer string,
) *MsgConnectionOpenTry {
	counterparty := NewCounterparty(counterpartyClientID, counterpartyConnectionID, counterpartyPrefix)
	protoAny, _ := clienttypes.PackClientState(counterpartyClient)
	return &MsgConnectionOpenTry{
		ClientId:             clientID,
		ClientState:          protoAny,
		Counterparty:         counterparty,
		CounterpartyVersions: counterpartyVersions,
		DelayPeriod:          delayPeriod,
		ProofInit:            initProof,
		ProofClient:          clientProof,
		ProofConsensus:       consensusProof,
		ProofHeight:          proofHeight,
		ConsensusHeight:      consensusHeight,
		Signer:               signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgConnectionOpenTry) ValidateBasic() error {
	if msg.ClientId == exported.LocalhostClientID {
		return errorsmod.Wrap(clienttypes.ErrInvalidClientType, "localhost connection handshakes are disallowed")
	}

	if msg.PreviousConnectionId != "" {
		return errorsmod.Wrap(ErrInvalidConnectionIdentifier, "previous connection identifier must be empty, this field has been deprecated as crossing hellos are no longer supported")
	}
	if err := host.ClientIdentifierValidator(msg.ClientId); err != nil {
		return errorsmod.Wrap(err, "invalid client ID")
	}
	// counterparty validate basic allows empty counterparty connection identifiers
	if err := host.ConnectionIdentifierValidator(msg.Counterparty.ConnectionId); err != nil {
		return errorsmod.Wrap(err, "invalid counterparty connection ID")
	}
	if msg.ClientState == nil {
		return errorsmod.Wrap(clienttypes.ErrInvalidClient, "counterparty client is nil")
	}
	clientState, err := clienttypes.UnpackClientState(msg.ClientState)
	if err != nil {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClient, "unpack err: %v", err)
	}
	if err := clientState.Validate(); err != nil {
		return errorsmod.Wrap(err, "counterparty client is invalid")
	}
	if len(msg.CounterpartyVersions) == 0 {
		return errorsmod.Wrap(ibcerrors.ErrInvalidVersion, "empty counterparty versions")
	}
	for i, version := range msg.CounterpartyVersions {
		if err := ValidateVersion(version); err != nil {
			return errorsmod.Wrapf(err, "basic validation failed on version with index %d", i)
		}
	}
	if len(msg.ProofInit) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty proof init")
	}
	if len(msg.ProofClient) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit empty proof client")
	}
	if len(msg.ProofConsensus) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty proof of consensus state")
	}
	if msg.ConsensusHeight.IsZero() {
		return errorsmod.Wrap(ibcerrors.ErrInvalidHeight, "consensus height must be non-zero")
	}
	_, err = sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Counterparty.ValidateBasic()
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgConnectionOpenTry) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(msg.ClientState, new(exported.ClientState))
}

// GetSigners implements sdk.Msg
func (msg MsgConnectionOpenTry) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// NewMsgConnectionOpenAck creates a new MsgConnectionOpenAck instance
func NewMsgConnectionOpenAck(
	connectionID, counterpartyConnectionID string, counterpartyClient exported.ClientState,
	tryProof, clientProof, consensusProof []byte,
	proofHeight, consensusHeight clienttypes.Height,
	version *Version,
	signer string,
) *MsgConnectionOpenAck {
	protoAny, _ := clienttypes.PackClientState(counterpartyClient)
	return &MsgConnectionOpenAck{
		ConnectionId:             connectionID,
		CounterpartyConnectionId: counterpartyConnectionID,
		ClientState:              protoAny,
		ProofTry:                 tryProof,
		ProofClient:              clientProof,
		ProofConsensus:           consensusProof,
		ProofHeight:              proofHeight,
		ConsensusHeight:          consensusHeight,
		Version:                  version,
		Signer:                   signer,
	}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgConnectionOpenAck) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(msg.ClientState, new(exported.ClientState))
}

// ValidateBasic implements sdk.Msg
func (msg MsgConnectionOpenAck) ValidateBasic() error {
	if !IsValidConnectionID(msg.ConnectionId) {
		return ErrInvalidConnectionIdentifier
	}
	if err := host.ConnectionIdentifierValidator(msg.CounterpartyConnectionId); err != nil {
		return errorsmod.Wrap(err, "invalid counterparty connection ID")
	}
	if err := ValidateVersion(msg.Version); err != nil {
		return err
	}
	if msg.ClientState == nil {
		return errorsmod.Wrap(clienttypes.ErrInvalidClient, "counterparty client is nil")
	}
	clientState, err := clienttypes.UnpackClientState(msg.ClientState)
	if err != nil {
		return errorsmod.Wrapf(clienttypes.ErrInvalidClient, "unpack err: %v", err)
	}
	if err := clientState.Validate(); err != nil {
		return errorsmod.Wrap(err, "counterparty client is invalid")
	}
	if len(msg.ProofTry) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty proof try")
	}
	if len(msg.ProofClient) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit empty proof client")
	}
	if len(msg.ProofConsensus) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty proof of consensus state")
	}
	if msg.ConsensusHeight.IsZero() {
		return errorsmod.Wrap(ibcerrors.ErrInvalidHeight, "consensus height must be non-zero")
	}
	_, err = sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgConnectionOpenAck) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// NewMsgConnectionOpenConfirm creates a new MsgConnectionOpenConfirm instance
func NewMsgConnectionOpenConfirm(
	connectionID string, ackProof []byte, proofHeight clienttypes.Height,
	signer string,
) *MsgConnectionOpenConfirm {
	return &MsgConnectionOpenConfirm{
		ConnectionId: connectionID,
		ProofAck:     ackProof,
		ProofHeight:  proofHeight,
		Signer:       signer,
	}
}

// ValidateBasic implements sdk.Msg
func (msg MsgConnectionOpenConfirm) ValidateBasic() error {
	if !IsValidConnectionID(msg.ConnectionId) {
		return ErrInvalidConnectionIdentifier
	}
	if len(msg.ProofAck) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty proof ack")
	}
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgConnectionOpenConfirm) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// NewMsgUpdateParams creates a new MsgUpdateParams instance
func NewMsgUpdateParams(signer string, params Params) *MsgUpdateParams {
	return &MsgUpdateParams{
		Signer: signer,
		Params: params,
	}
}

// GetSigners returns the expected signers for a MsgUpdateParams message.
func (msg *MsgUpdateParams) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// ValidateBasic performs basic checks on a MsgUpdateParams.
func (msg *MsgUpdateParams) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Signer); err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return msg.Params.Validate()
}
