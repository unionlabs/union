package types

import (
	errorsmod "cosmossdk.io/errors"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ sdk.Msg = (*MsgCreateClient)(nil)
	_ sdk.Msg = (*MsgUpdateClient)(nil)
	_ sdk.Msg = (*MsgSubmitMisbehaviour)(nil)
	_ sdk.Msg = (*MsgUpgradeClient)(nil)
	_ sdk.Msg = (*MsgUpdateParams)(nil)
	_ sdk.Msg = (*MsgIBCSoftwareUpgrade)(nil)
	_ sdk.Msg = (*MsgRecoverClient)(nil)

	_ sdk.HasValidateBasic = (*MsgCreateClient)(nil)
	_ sdk.HasValidateBasic = (*MsgUpdateClient)(nil)
	_ sdk.HasValidateBasic = (*MsgSubmitMisbehaviour)(nil)
	_ sdk.HasValidateBasic = (*MsgUpgradeClient)(nil)
	_ sdk.HasValidateBasic = (*MsgUpdateParams)(nil)
	_ sdk.HasValidateBasic = (*MsgIBCSoftwareUpgrade)(nil)
	_ sdk.HasValidateBasic = (*MsgRecoverClient)(nil)

	_ codectypes.UnpackInterfacesMessage = (*MsgCreateClient)(nil)
	_ codectypes.UnpackInterfacesMessage = (*MsgUpdateClient)(nil)
	_ codectypes.UnpackInterfacesMessage = (*MsgSubmitMisbehaviour)(nil)
	_ codectypes.UnpackInterfacesMessage = (*MsgUpgradeClient)(nil)
	_ codectypes.UnpackInterfacesMessage = (*MsgIBCSoftwareUpgrade)(nil)
)

// NewMsgCreateClient creates a new MsgCreateClient instance
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
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
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
		return errorsmod.Wrap(ErrInvalidClientType, "client type for client state and consensus state do not match")
	}
	if err := ValidateClientType(clientState.ClientType()); err != nil {
		return errorsmod.Wrap(err, "client type does not meet naming constraints")
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
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
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
func NewMsgUpgradeClient(clientID string, clientState exported.ClientState, consState exported.ConsensusState,
	upgradeClientProof, upgradeConsensusStateProof []byte, signer string,
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
		ProofUpgradeClient:         upgradeClientProof,
		ProofUpgradeConsensusState: upgradeConsensusStateProof,
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
		return errorsmod.Wrapf(ErrInvalidUpgradeClient, "consensus state's client-type does not match client. expected: %s, got: %s",
			clientState.ClientType(), consensusState.ClientType())
	}
	if len(msg.ProofUpgradeClient) == 0 {
		return errorsmod.Wrap(ErrInvalidUpgradeClient, "proof of upgrade client cannot be empty")
	}
	if len(msg.ProofUpgradeConsensusState) == 0 {
		return errorsmod.Wrap(ErrInvalidUpgradeClient, "proof of upgrade consensus state cannot be empty")
	}
	_, err = sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
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
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
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

// NewMsgRecoverClient creates a new MsgRecoverClient instance
func NewMsgRecoverClient(signer, subjectClientID, substituteClientID string) *MsgRecoverClient {
	return &MsgRecoverClient{
		Signer:             signer,
		SubjectClientId:    subjectClientID,
		SubstituteClientId: substituteClientID,
	}
}

// ValidateBasic performs basic checks on a MsgRecoverClient.
func (msg *MsgRecoverClient) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Signer); err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	if err := host.ClientIdentifierValidator(msg.SubjectClientId); err != nil {
		return err
	}

	if err := host.ClientIdentifierValidator(msg.SubstituteClientId); err != nil {
		return err
	}

	if msg.SubjectClientId == msg.SubstituteClientId {
		return errorsmod.Wrapf(ErrInvalidSubstitute, "subject and substitute clients must be different")
	}

	return nil
}

// GetSigners returns the expected signers for a MsgRecoverClient message.
func (msg *MsgRecoverClient) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// NewMsgIBCSoftwareUpgrade creates a new MsgIBCSoftwareUpgrade instance
func NewMsgIBCSoftwareUpgrade(signer string, plan upgradetypes.Plan, upgradedClientState exported.ClientState) (*MsgIBCSoftwareUpgrade, error) {
	anyClient, err := PackClientState(upgradedClientState)
	if err != nil {
		return nil, err
	}

	return &MsgIBCSoftwareUpgrade{
		Signer:              signer,
		Plan:                plan,
		UpgradedClientState: anyClient,
	}, nil
}

// ValidateBasic performs basic checks on a MsgIBCSoftwareUpgrade.
func (msg *MsgIBCSoftwareUpgrade) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Signer); err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	clientState, err := UnpackClientState(msg.UpgradedClientState)
	if err != nil {
		return err
	}

	// for the time being, we should implicitly be on tendermint when using ibc-go
	if clientState.ClientType() != exported.Tendermint {
		return errorsmod.Wrapf(ErrInvalidUpgradeClient, "upgraded client state must be a Tendermint client")
	}

	return msg.Plan.ValidateBasic()
}

// GetSigners returns the expected signers for a MsgIBCSoftwareUpgrade message.
func (msg *MsgIBCSoftwareUpgrade) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}
	return []sdk.AccAddress{accAddr}
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg *MsgIBCSoftwareUpgrade) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return unpacker.UnpackAny(msg.UpgradedClientState, new(exported.ClientState))
}

// NewMsgUpdateParams creates a new instance of MsgUpdateParams.
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
