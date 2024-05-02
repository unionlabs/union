package types

import (
	"fmt"
	"reflect"
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
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

// NewErrorAcknowledgementWithCodespace returns a new instance of Acknowledgement using an Acknowledgement_Error
// type in the Response field.
// NOTE: The error includes the ABCI codespace and code in the error string to provide more information about the module
// that generated the error. This is useful for debugging but can potentially introduce non-determinism if care is
// not taken to ensure the codespace doesn't change in non state-machine breaking versions.
func NewErrorAcknowledgementWithCodespace(err error) Acknowledgement {
	// The ABCI code is included in the abcitypes.ResponseDeliverTx hash
	// constructed in Tendermint and is therefore deterministic.
	// However, a code without codespace is incomplete information (e.g. sdk/5 and wasm/5 are
	// different errors). We add this codespace here, in oder to provide a meaningful error
	// identifier which means changing the codespace of an error becomes a consensus breaking change.
	codespace, code, _ := errorsmod.ABCIInfo(err, false)

	return Acknowledgement{
		Response: &Acknowledgement_Error{
			Error: fmt.Sprintf("ABCI error: %s/%d: %s", codespace, code, ackErrorString),
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
	_, code, _ := errorsmod.ABCIInfo(err, false) // discard non-determinstic codespace and log values

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
			return errorsmod.Wrap(ErrInvalidAcknowledgement, "acknowledgement result cannot be empty")
		}
	case *Acknowledgement_Error:
		if strings.TrimSpace(resp.Error) == "" {
			return errorsmod.Wrap(ErrInvalidAcknowledgement, "acknowledgement error cannot be empty")
		}

	default:
		return errorsmod.Wrapf(ErrInvalidAcknowledgement, "unsupported acknowledgement response field type %T", resp)
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
