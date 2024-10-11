package rpc

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	slashing "github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history"
	"go.opencensus.io/trace"
)

// ExportSlashingProtection handles the rpc call returning the json slashing history.
// The format of the export follows the EIP-3076 standard which makes it
// easy to migrate machines or Ethereum consensus clients.
//
// Steps:
//  1. Call the function which exports the data from
//     the validator's db into an EIP standard slashing protection format.
//  2. Format and send JSON in the response.
func (s *Server) ExportSlashingProtection(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.ExportSlashingProtection")
	defer span.End()

	if s.valDB == nil {
		httputil.HandleError(w, "could not find validator database", http.StatusInternalServerError)
		return
	}

	eipJSON, err := slashing.ExportStandardProtectionJSON(ctx, s.valDB)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "could not export slashing protection history").Error(), http.StatusInternalServerError)
		return
	}

	encoded, err := json.MarshalIndent(eipJSON, "", "\t")
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "could not JSON marshal slashing protection history").Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ExportSlashingProtectionResponse{
		File: string(encoded),
	})
}

// ImportSlashingProtection reads an input slashing protection EIP-3076
// standard JSON string and inserts the data into validator DB.
//
// Read the JSON string passed through rpc, then call the func
// which actually imports the data from the JSON file into our database. Use the Keymanager APIs if an API is required.
func (s *Server) ImportSlashingProtection(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.ImportSlashingProtection")
	defer span.End()

	if s.valDB == nil {
		httputil.HandleError(w, "could not find validator database", http.StatusInternalServerError)
		return
	}

	var req ImportSlashingProtectionRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	if req.SlashingProtectionJson == "" {
		httputil.HandleError(w, "empty slashing_protection_json specified", http.StatusBadRequest)
		return
	}
	enc := []byte(req.SlashingProtectionJson)
	buf := bytes.NewBuffer(enc)
	if err := slashing.ImportStandardProtectionJSON(ctx, s.valDB, buf); err != nil {
		httputil.HandleError(w, errors.Wrap(err, "could not import slashing protection history").Error(), http.StatusInternalServerError)
		return
	}
	log.Info("Slashing protection JSON successfully imported")
}
