package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewIncentivizedAcknowledgement creates a new instance of IncentivizedAcknowledgement
func NewIncentivizedAcknowledgement(relayer string, ack []byte, success bool) IncentivizedAcknowledgement {
	return IncentivizedAcknowledgement{
		AppAcknowledgement:    ack,
		ForwardRelayerAddress: relayer,
		UnderlyingAppSuccess:  success,
	}
}

// Success implements the Acknowledgement interface. The acknowledgement is
// considered successful if the forward relayer address is empty. Otherwise it is
// considered a failed acknowledgement.
func (ack IncentivizedAcknowledgement) Success() bool {
	return ack.UnderlyingAppSuccess
}

// Acknowledgement implements the Acknowledgement interface. It returns the
// acknowledgement serialised using JSON.
func (ack IncentivizedAcknowledgement) Acknowledgement() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&ack))
}
