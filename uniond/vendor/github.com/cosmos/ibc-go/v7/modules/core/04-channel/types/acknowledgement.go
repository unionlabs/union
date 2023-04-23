package types

import (
	"fmt"
	"reflect"
	"strings"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	// ackErrorString defines a string constant included in error acknowledgements
	// NOTE: Changing this const is state machine breaking as acknowledgements are written into state.
	ackErrorString = "error handling packet: see events for details"
)

// NewResultAcknowledgement returns a new instance of Acknowledgement using an Acknowledgement_Result
// type in the Response field.
func NewResultAcknowledgement(result []byte) Acknowledgement {
	return Acknowledgement{
		Response: &Acknowledgement_Result{
			Result: result,
		},
	}
}

// NewErrorAcknowledgement returns a new instance of Acknowledgement using an Acknowledgement_Error
// type in the Response field.
// NOTE: Acknowledgements are written into state and thus, changes made to error strings included in packet acknowledgements
// risk an app hash divergence when nodes in a network are running different patch versions of software.
func NewErrorAcknowledgement(err error) Acknowledgement {
	// the ABCI code is included in the abcitypes.ResponseDeliverTx hash
	// constructed in Tendermint and is therefore deterministic
	_, code, _ := sdkerrors.ABCIInfo(err, false) // discard non-determinstic codespace and log values

	return Acknowledgement{
		Response: &Acknowledgement_Error{
			Error: fmt.Sprintf("ABCI code: %d: %s", code, ackErrorString),
		},
	}
}

// ValidateBasic performs a basic validation of the acknowledgement
func (ack Acknowledgement) ValidateBasic() error {
	switch resp := ack.Response.(type) {
	case *Acknowledgement_Result:
		if len(resp.Result) == 0 {
			return sdkerrors.Wrap(ErrInvalidAcknowledgement, "acknowledgement result cannot be empty")
		}
	case *Acknowledgement_Error:
		if strings.TrimSpace(resp.Error) == "" {
			return sdkerrors.Wrap(ErrInvalidAcknowledgement, "acknowledgement error cannot be empty")
		}

	default:
		return sdkerrors.Wrapf(ErrInvalidAcknowledgement, "unsupported acknowledgement response field type %T", resp)
	}
	return nil
}

// Success implements the Acknowledgement interface. The acknowledgement is
// considered successful if it is a ResultAcknowledgement. Otherwise it is
// considered a failed acknowledgement.
func (ack Acknowledgement) Success() bool {
	return reflect.TypeOf(ack.Response) == reflect.TypeOf(((*Acknowledgement_Result)(nil)))
}

// Acknowledgement implements the Acknowledgement interface. It returns the
// acknowledgement serialised using JSON.
func (ack Acknowledgement) Acknowledgement() []byte {
	return sdk.MustSortJSON(SubModuleCdc.MustMarshalJSON(&ack))
}
