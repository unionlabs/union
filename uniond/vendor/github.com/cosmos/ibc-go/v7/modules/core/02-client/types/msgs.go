package types

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

var (
	_ sdk.Msg = &MsgCreateClient{}
	_ sdk.Msg = &MsgUpdateClient{}
	_ sdk.Msg = &MsgSubmitMisbehaviour{}
	_ sdk.Msg = &MsgUpgradeClient{}

	_ codectypes.UnpackInterfacesMessage = MsgCreateClient{}
	_ codectypes.UnpackInterfacesMessage = MsgUpdateClient{}
	_ codectypes.UnpackInterfacesMessage = MsgSubmitMisbehaviour{}
	_ codectypes.UnpackInterfacesMessage = MsgUpgradeClient{}
)

// NewMsgCreateClient creates a new MsgCreateClient instance
//
//nolint:interfacer
func NewMsgCreateClient(
	clientState exported.ClientState, consensusState exported.ConsensusState, signer string,
) (*MsgCreateClient, error) {
	anyClientState, err := PackClientState(clientState)
	if err != nil {
		return nil, err
	}

	anyConsensusState, err := PackConsensusState(consensusState)
	if err != nil {
		return nil, err
	}

	return &MsgCreateClient{
		ClientState:    anyClientState,
		ConsensusState: anyConsensusState,
		Signer:         signer,
	}, nil
}

// ValidateBasic implements sdk.Msg
func (msg MsgCreateClient) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return sdkerrors.Wrapf(sdkerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	clientState, err := UnpackClientState(msg.ClientState)
	if err != nil {
		return err
	}
	if err := clientState.Validate(); err != nil {
		return err
	}
	consensusState, err := UnpackConsensusState(msg.ConsensusState)
	if err != nil {
		return err
	}
	if clientState.ClientType() != consensusState.ClientType() {
		return sdkerrors.Wrap(ErrInvalidClientType, "client type for client state and consensus state do not match")
	}
	if err := ValidateClientType(clientState.ClientType()); err != nil {
		return sdkerrors.Wrap(err, "client type does not meet naming constraints")
	}
	return consensusState.ValidateBasic()
}

// GetSigners implements sdk.Msg
func (msg MsgCreateClient) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgCreateClient) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	var clientState exported.ClientState
	err := unpacker.UnpackAny(msg.ClientState, &clientState)
	if err != nil {
		return err
	}

	var consensusState exported.ConsensusState
	return unpacker.UnpackAny(msg.ConsensusState, &consensusState)
}

// NewMsgUpdateClient creates a new MsgUpdateClient instance
//
//nolint:interfacer
func NewMsgUpdateClient(id string, clientMsg exported.ClientMessage, signer string) (*MsgUpdateClient, error) {
	anyClientMsg, err := PackClientMessage(clientMsg)
	if err != nil {
		return nil, err
	}

	return &MsgUpdateClient{
		ClientId:      id,
		ClientMessage: anyClientMsg,
		Signer:        signer,
	}, nil
}

// ValidateBasic implements sdk.Msg
func (msg MsgUpdateClient) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return sdkerrors.Wrapf(sdkerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	clientMsg, err := UnpackClientMessage(msg.ClientMessage)
	if err != nil {
		return err
	}
	if err := clientMsg.ValidateBasic(); err != nil {
		return err
	}
	return host.ClientIdentifierValidator(msg.ClientId)
}

// GetSigners implements sdk.Msg
func (msg MsgUpdateClient) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgUpdateClient) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	var clientMsg exported.ClientMessage
	return unpacker.UnpackAny(msg.ClientMessage, &clientMsg)
}

// NewMsgUpgradeClient creates a new MsgUpgradeClient instance
//
//nolint:interfacer
func NewMsgUpgradeClient(clientID string, clientState exported.ClientState, consState exported.ConsensusState,
	proofUpgradeClient, proofUpgradeConsState []byte, signer string,
) (*MsgUpgradeClient, error) {
	anyClient, err := PackClientState(clientState)
	if err != nil {
		return nil, err
	}
	anyConsState, err := PackConsensusState(consState)
	if err != nil {
		return nil, err
	}

	return &MsgUpgradeClient{
		ClientId:                   clientID,
		ClientState:                anyClient,
		ConsensusState:             anyConsState,
		ProofUpgradeClient:         proofUpgradeClient,
		ProofUpgradeConsensusState: proofUpgradeConsState,
		Signer:                     signer,
	}, nil
}

// ValidateBasic implements sdk.Msg
func (msg MsgUpgradeClient) ValidateBasic() error {
	// will not validate client state as committed client may not form a valid client state.
	// client implementations are responsible for ensuring final upgraded client is valid.
	clientState, err := UnpackClientState(msg.ClientState)
	if err != nil {
		return err
	}
	// will not validate consensus state here since the trusted kernel may not form a valid consenus state.
	// client implementations are responsible for ensuring client can submit new headers against this consensus state.
	consensusState, err := UnpackConsensusState(msg.ConsensusState)
	if err != nil {
		return err
	}

	if clientState.ClientType() != consensusState.ClientType() {
		return sdkerrors.Wrapf(ErrInvalidUpgradeClient, "consensus state's client-type does not match client. expected: %s, got: %s",
			clientState.ClientType(), consensusState.ClientType())
	}
	if len(msg.ProofUpgradeClient) == 0 {
		return sdkerrors.Wrap(ErrInvalidUpgradeClient, "proof of upgrade client cannot be empty")
	}
	if len(msg.ProofUpgradeConsensusState) == 0 {
		return sdkerrors.Wrap(ErrInvalidUpgradeClient, "proof of upgrade consensus state cannot be empty")
	}
	_, err = sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return sdkerrors.Wrapf(sdkerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	return host.ClientIdentifierValidator(msg.ClientId)
}

// GetSigners implements sdk.Msg
func (msg MsgUpgradeClient) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgUpgradeClient) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	var (
		clientState exported.ClientState
		consState   exported.ConsensusState
	)
	if err := unpacker.UnpackAny(msg.ClientState, &clientState); err != nil {
		return err
	}
	return unpacker.UnpackAny(msg.ConsensusState, &consState)
}

// NewMsgSubmitMisbehaviour creates a new MsgSubmitMisbehaviour instance.
//
//nolint:interfacer
func NewMsgSubmitMisbehaviour(clientID string, misbehaviour exported.ClientMessage, signer string) (*MsgSubmitMisbehaviour, error) {
	anyMisbehaviour, err := PackClientMessage(misbehaviour)
	if err != nil {
		return nil, err
	}

	return &MsgSubmitMisbehaviour{
		ClientId:     clientID,
		Misbehaviour: anyMisbehaviour,
		Signer:       signer,
	}, nil
}

// ValidateBasic performs basic (non-state-dependant) validation on a MsgSubmitMisbehaviour.
func (msg MsgSubmitMisbehaviour) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return sdkerrors.Wrapf(sdkerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}
	misbehaviour, err := UnpackClientMessage(msg.Misbehaviour)
	if err != nil {
		return err
	}
	if err := misbehaviour.ValidateBasic(); err != nil {
		return err
	}

	return host.ClientIdentifierValidator(msg.ClientId)
}

// GetSigners returns the single expected signer for a MsgSubmitMisbehaviour.
func (msg MsgSubmitMisbehaviour) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgSubmitMisbehaviour) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	var misbehaviour exported.ClientMessage
	return unpacker.UnpackAny(msg.Misbehaviour, &misbehaviour)
}
