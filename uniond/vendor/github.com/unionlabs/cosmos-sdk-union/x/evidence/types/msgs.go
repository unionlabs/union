package types

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"cosmossdk.io/x/evidence/exported"

	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	_ sdk.Msg                              = &MsgSubmitEvidence{}
	_ gogoprotoany.UnpackInterfacesMessage = MsgSubmitEvidence{}
	_ exported.MsgSubmitEvidenceI          = &MsgSubmitEvidence{}
)

// NewMsgSubmitEvidence returns a new MsgSubmitEvidence with a signer/submitter.
func NewMsgSubmitEvidence(s string, evi exported.Evidence) (*MsgSubmitEvidence, error) {
	msg, ok := evi.(proto.Message)
	if !ok {
		return nil, fmt.Errorf("cannot proto marshal %T", evi)
	}
	any, err := types.NewAnyWithValue(msg)
	if err != nil {
		return nil, err
	}
	return &MsgSubmitEvidence{Submitter: s, Evidence: any}, nil
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

func (m MsgSubmitEvidence) UnpackInterfaces(ctx gogoprotoany.AnyUnpacker) error {
	var evi exported.Evidence
	return ctx.UnpackAny(m.Evidence, &evi)
}
