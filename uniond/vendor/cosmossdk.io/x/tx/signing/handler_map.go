package signing

import (
	"context"
	"fmt"

	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
)

// HandlerMap aggregates several sign mode handlers together for convenient generation of sign bytes
// based on sign mode.
type HandlerMap struct {
	signModeHandlers map[signingv1beta1.SignMode]SignModeHandler
	defaultMode      signingv1beta1.SignMode
	modes            []signingv1beta1.SignMode
}

// NewHandlerMap constructs a new sign mode handler map. The first handler is used as the default.
func NewHandlerMap(handlers ...SignModeHandler) *HandlerMap {
	if len(handlers) == 0 {
		panic("no handlers")
	}
	res := &HandlerMap{
		signModeHandlers: map[signingv1beta1.SignMode]SignModeHandler{},
	}

	for i, handler := range handlers {
		if handler == nil {
			panic("nil handler")
		}
		mode := handler.Mode()
		if i == 0 {
			res.defaultMode = mode
		}
		res.signModeHandlers[mode] = handler
		res.modes = append(res.modes, mode)
	}

	return res
}

// SupportedModes lists the modes supported by this handler map.
func (h *HandlerMap) SupportedModes() []signingv1beta1.SignMode {
	return h.modes
}

// DefaultMode returns the default mode for this handler map.
func (h *HandlerMap) DefaultMode() signingv1beta1.SignMode {
	return h.defaultMode
}

// GetSignBytes returns the sign bytes for the transaction for the requested mode.
func (h *HandlerMap) GetSignBytes(ctx context.Context, signMode signingv1beta1.SignMode, signerData SignerData, txData TxData) ([]byte, error) {
	handler, ok := h.signModeHandlers[signMode]
	if !ok {
		return nil, fmt.Errorf("unsupported sign mode %s", signMode)
	}

	return handler.GetSignBytes(ctx, signerData, txData)
}
