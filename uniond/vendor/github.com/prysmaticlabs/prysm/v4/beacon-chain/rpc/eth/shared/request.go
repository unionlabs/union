package shared

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
)

func UintFromQuery(w http.ResponseWriter, r *http.Request, name string, required bool) (string, uint64, bool) {
	trimmed := strings.ReplaceAll(r.URL.Query().Get(name), " ", "")
	if trimmed == "" && !required {
		return "", 0, true
	}
	v, valid := ValidateUint(w, name, trimmed)
	if !valid {
		return "", 0, false
	}
	return trimmed, v, true
}

func UintFromRoute(w http.ResponseWriter, r *http.Request, name string) (string, uint64, bool) {
	raw := mux.Vars(r)[name]
	v, valid := ValidateUint(w, name, raw)
	if !valid {
		return "", 0, false
	}
	return raw, v, true
}

func HexFromQuery(w http.ResponseWriter, r *http.Request, name string, length int, required bool) (string, []byte, bool) {
	raw := r.URL.Query().Get(name)
	if raw == "" && !required {
		return "", nil, true
	}
	v, valid := ValidateHex(w, name, raw, length)
	if !valid {
		return "", nil, false
	}
	return raw, v, true
}

func HexFromRoute(w http.ResponseWriter, r *http.Request, name string, length int) (string, []byte, bool) {
	raw := mux.Vars(r)[name]
	v, valid := ValidateHex(w, name, raw, length)
	if !valid {
		return "", nil, false
	}
	return raw, v, true
}

func ValidateHex(w http.ResponseWriter, name, s string, length int) ([]byte, bool) {
	if s == "" {
		errJson := &httputil.DefaultJsonError{
			Message: name + " is required",
			Code:    http.StatusBadRequest,
		}
		httputil.WriteError(w, errJson)
		return nil, false
	}
	hexBytes, err := hexutil.Decode(s)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: name + " is invalid: " + err.Error(),
			Code:    http.StatusBadRequest,
		}
		httputil.WriteError(w, errJson)
		return nil, false
	}
	if len(hexBytes) != length {
		httputil.HandleError(w, fmt.Sprintf("Invalid %s: %s is not length %d", name, s, length), http.StatusBadRequest)
		return nil, false
	}
	return hexBytes, true
}

func ValidateUint(w http.ResponseWriter, name, s string) (uint64, bool) {
	if s == "" {
		errJson := &httputil.DefaultJsonError{
			Message: name + " is required",
			Code:    http.StatusBadRequest,
		}
		httputil.WriteError(w, errJson)
		return 0, false
	}
	v, err := strconv.ParseUint(s, 10, 64)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: name + " is invalid: " + err.Error(),
			Code:    http.StatusBadRequest,
		}
		httputil.WriteError(w, errJson)
		return 0, false
	}
	return v, true
}

// IsSyncing checks whether the beacon node is currently syncing and writes out the sync status.
func IsSyncing(
	ctx context.Context,
	w http.ResponseWriter,
	syncChecker sync.Checker,
	headFetcher blockchain.HeadFetcher,
	timeFetcher blockchain.TimeFetcher,
	optimisticModeFetcher blockchain.OptimisticModeFetcher,
) bool {
	if !syncChecker.Syncing() {
		return false
	}

	headSlot := headFetcher.HeadSlot()
	isOptimistic, err := optimisticModeFetcher.IsOptimistic(ctx)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: "Could not check optimistic status: " + err.Error(),
			Code:    http.StatusInternalServerError,
		}
		httputil.WriteError(w, errJson)
		return true
	}
	syncDetails := &SyncDetailsContainer{
		Data: &SyncDetails{
			HeadSlot:     strconv.FormatUint(uint64(headSlot), 10),
			SyncDistance: strconv.FormatUint(uint64(timeFetcher.CurrentSlot()-headSlot), 10),
			IsSyncing:    true,
			IsOptimistic: isOptimistic,
		},
	}

	msg := "Beacon node is currently syncing and not serving request on that endpoint"
	details, err := json.Marshal(syncDetails)
	if err == nil {
		msg += " Details: " + string(details)
	}
	errJson := &httputil.DefaultJsonError{
		Message: msg,
		Code:    http.StatusServiceUnavailable}
	httputil.WriteError(w, errJson)
	return true
}

// IsOptimistic checks whether the beacon node is currently optimistic and writes it to the response.
func IsOptimistic(
	ctx context.Context,
	w http.ResponseWriter,
	optimisticModeFetcher blockchain.OptimisticModeFetcher,
) (bool, error) {
	isOptimistic, err := optimisticModeFetcher.IsOptimistic(ctx)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: "Could not check optimistic status: " + err.Error(),
			Code:    http.StatusInternalServerError,
		}
		httputil.WriteError(w, errJson)
		return true, err
	}
	if !isOptimistic {
		return false, nil
	}
	errJson := &httputil.DefaultJsonError{
		Code:    http.StatusServiceUnavailable,
		Message: "Beacon node is currently optimistic and not serving validators",
	}
	httputil.WriteError(w, errJson)
	return true, nil
}
