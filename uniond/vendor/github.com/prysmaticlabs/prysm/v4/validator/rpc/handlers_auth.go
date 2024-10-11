package rpc

import (
	"net/http"
	"path/filepath"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"go.opencensus.io/trace"
)

// Initialize returns metadata regarding whether the caller has authenticated and has a wallet.
func (s *Server) Initialize(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "validator.web.Initialize")
	defer span.End()
	walletExists, err := wallet.Exists(s.walletDir)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not check if wallet exists").Error(), http.StatusInternalServerError)
		return
	}
	authTokenPath := filepath.Join(s.walletDir, AuthTokenFileName)
	httputil.WriteJson(w, &InitializeAuthResponse{
		HasSignedUp: file.Exists(authTokenPath),
		HasWallet:   walletExists,
	})
}
