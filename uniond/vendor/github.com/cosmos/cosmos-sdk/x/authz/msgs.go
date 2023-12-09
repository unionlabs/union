package authz

import (
	"time"

	"github.com/cosmos/gogoproto/proto"

	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var (
	_ sdk.Msg = &MsgGrant{}
	_ sdk.Msg = &MsgRevoke{}
	_ sdk.Msg = &MsgExec{}

	_ cdctypes.UnpackInterfacesMessage = &MsgGrant{}
	_ cdctypes.UnpackInterfacesMessage = &MsgExec{}
)

// NewMsgGrant creates a new MsgGrant
func NewMsgGrant(granter, grantee sdk.AccAddress, a Authorization, expiration *time.Time) (*MsgGrant, error) {
	m := &MsgGrant{
		Granter: granter.String(),
		Grantee: grantee.String(),
		Grant:   Grant{Expiration: expiration},
	}
	err := m.SetAuthorization(a)
	if err != nil {
		return nil, err
	}
	return m, nil
}

// GetAuthorization returns the cache value from the MsgGrant.Authorization if present.
func (msg *MsgGrant) GetAuthorization() (Authorization, error) {
	return msg.Grant.GetAuthorization()
}

// SetAuthorization converts Authorization to any and adds it to MsgGrant.Authorization.
func (msg *MsgGrant) SetAuthorization(a Authorization) error {
	m, ok := a.(proto.Message)
	if !ok {
		return sdkerrors.ErrPackAny.Wrapf("can't proto marshal %T", m)
	}
	any, err := cdctypes.NewAnyWithValue(m)
	if err != nil {
		return err
	}
	msg.Grant.Authorization = any
	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgExec) UnpackInterfaces(unpacker cdctypes.AnyUnpacker) error {
	for _, x := range msg.Msgs {
		var msgExecAuthorized sdk.Msg
		err := unpacker.UnpackAny(x, &msgExecAuthorized)
		if err != nil {
			return err
		}
	}

	return nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgGrant) UnpackInterfaces(unpacker cdctypes.AnyUnpacker) error {
	return msg.Grant.UnpackInterfaces(unpacker)
}

// NewMsgRevoke creates a new MsgRevoke
func NewMsgRevoke(granter, grantee sdk.AccAddress, msgTypeURL string) MsgRevoke {
	return MsgRevoke{
		Granter:    granter.String(),
		Grantee:    grantee.String(),
		MsgTypeUrl: msgTypeURL,
	}
}

// NewMsgExec creates a new MsgExecAuthorized
func NewMsgExec(grantee sdk.AccAddress, msgs []sdk.Msg) MsgExec {
	msgsAny := make([]*cdctypes.Any, len(msgs))
	for i, msg := range msgs {
		any, err := cdctypes.NewAnyWithValue(msg)
		if err != nil {
			panic(err)
		}

		msgsAny[i] = any
	}

	return MsgExec{
		Grantee: grantee.String(),
		Msgs:    msgsAny,
	}
}

// GetMessages returns the cache values from the MsgExecAuthorized.Msgs if present.
func (msg MsgExec) GetMessages() ([]sdk.Msg, error) {
	msgs := make([]sdk.Msg, len(msg.Msgs))
	for i, msgAny := range msg.Msgs {
		msg, ok := msgAny.GetCachedValue().(sdk.Msg)
		if !ok {
			return nil, sdkerrors.ErrInvalidRequest.Wrapf("messages contains %T which is not a sdk.MsgRequest", msgAny)
		}
		msgs[i] = msg
	}

	return msgs, nil
}
