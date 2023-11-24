package types

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/x/evidence/exported"

	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	_ sdk.Msg                       = &MsgSubmitEvidence{}
	_ types.UnpackInterfacesMessage = MsgSubmitEvidence{}
	_ exported.MsgSubmitEvidenceI   = &MsgSubmitEvidence{}
)

// NewMsgSubmitEvidence returns a new MsgSubmitEvidence with a signer/submitter.
func NewMsgSubmitEvidence(s sdk.AccAddress, evi exported.Evidence) (*MsgSubmitEvidence, error) {
	msg, ok := evi.(proto.Message)
	if !ok {
		return nil, fmt.Errorf("cannot proto marshal %T", evi)
	}
	any, err := types.NewAnyWithValue(msg)
	if err != nil {
		return nil, err
	}
	return &MsgSubmitEvidence{Submitter: s.String(), Evidence: any}, nil
}

func (m MsgSubmitEvidence) GetEvidence() exported.Evidence {
	if m.Evidence == nil {
		return nil
	}

	evi, ok := m.Evidence.GetCachedValue().(exported.Evidence)
	if !ok {
		return nil
	}

	return evi
}

func (m MsgSubmitEvidence) GetSubmitter() sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(m.Submitter)
	if err != nil {
		return nil
	}
	return accAddr
}

func (m MsgSubmitEvidence) UnpackInterfaces(ctx types.AnyUnpacker) error {
	var evi exported.Evidence
	return ctx.UnpackAny(m.Evidence, &evi)
}
