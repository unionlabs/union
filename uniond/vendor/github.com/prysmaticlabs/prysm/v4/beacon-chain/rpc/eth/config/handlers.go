package config

import (
	"fmt"
	"net/http"
	"reflect"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"go.opencensus.io/trace"
)

// GetDepositContract retrieves deposit contract address and genesis fork version.
func GetDepositContract(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "config.GetDepositContract")
	defer span.End()

	httputil.WriteJson(w, &GetDepositContractResponse{
		Data: &DepositContractData{
			ChainId: strconv.FormatUint(params.BeaconConfig().DepositChainID, 10),
			Address: params.BeaconConfig().DepositContractAddress,
		},
	})
}

// GetForkSchedule retrieve all scheduled upcoming forks this node is aware of.
func GetForkSchedule(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "config.GetForkSchedule")
	defer span.End()

	schedule := params.BeaconConfig().ForkVersionSchedule
	if len(schedule) == 0 {
		httputil.WriteJson(w, &GetForkScheduleResponse{
			Data: make([]*shared.Fork, 0),
		})
		return
	}

	versions := forks.SortedForkVersions(schedule)
	chainForks := make([]*shared.Fork, len(schedule))
	var previous, current []byte
	for i, v := range versions {
		if i == 0 {
			previous = params.BeaconConfig().GenesisForkVersion
		} else {
			previous = current
		}
		copyV := v
		current = copyV[:]
		chainForks[i] = &shared.Fork{
			PreviousVersion: hexutil.Encode(previous),
			CurrentVersion:  hexutil.Encode(current),
			Epoch:           fmt.Sprintf("%d", schedule[v]),
		}
	}

	httputil.WriteJson(w, &GetForkScheduleResponse{
		Data: chainForks,
	})
}

// GetSpec retrieves specification configuration (without Phase 1 params) used on this node. Specification params list
// Values are returned with following format:
// - any value starting with 0x in the spec is returned as a hex string.
// - all other values are returned as number.
func GetSpec(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "config.GetSpec")
	defer span.End()

	data, err := prepareConfigSpec()
	if err != nil {
		httputil.HandleError(w, "Could not prepare config spec: "+err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &GetSpecResponse{Data: data})
}

func prepareConfigSpec() (map[string]string, error) {
	data := make(map[string]string)
	config := *params.BeaconConfig()
	t := reflect.TypeOf(config)
	v := reflect.ValueOf(config)

	for i := 0; i < t.NumField(); i++ {
		tField := t.Field(i)
		_, isSpecField := tField.Tag.Lookup("spec")
		if !isSpecField {
			// Field should not be returned from API.
			continue
		}

		tagValue := strings.ToUpper(tField.Tag.Get("yaml"))
		vField := v.Field(i)
		switch vField.Kind() {
		case reflect.Int:
			data[tagValue] = strconv.FormatInt(vField.Int(), 10)
		case reflect.Uint64:
			data[tagValue] = strconv.FormatUint(vField.Uint(), 10)
		case reflect.Slice:
			data[tagValue] = hexutil.Encode(vField.Bytes())
		case reflect.Array:
			data[tagValue] = hexutil.Encode(reflect.ValueOf(&config).Elem().Field(i).Slice(0, vField.Len()).Bytes())
		case reflect.String:
			data[tagValue] = vField.String()
		case reflect.Uint8:
			data[tagValue] = hexutil.Encode([]byte{uint8(vField.Uint())})
		default:
			return nil, fmt.Errorf("unsupported config field type: %s", vField.Kind().String())
		}
	}

	return data, nil
}
