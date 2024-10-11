package gateway

import (
	gwruntime "github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/api/gateway"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/protobuf/encoding/protojson"
)

// MuxConfig contains configuration that should be used when registering the beacon node in the gateway.
type MuxConfig struct {
	Handler      gateway.MuxHandler
	EthPbMux     *gateway.PbMux
	V1AlphaPbMux *gateway.PbMux
}

// DefaultConfig returns a fully configured MuxConfig with standard gateway behavior.
func DefaultConfig(enableDebugRPCEndpoints bool, httpModules string) MuxConfig {
	var v1AlphaPbHandler, ethPbHandler *gateway.PbMux
	if flags.EnableHTTPPrysmAPI(httpModules) {
		v1AlphaRegistrations := []gateway.PbHandlerRegistration{
			ethpbalpha.RegisterNodeHandler,
			ethpbalpha.RegisterBeaconChainHandler,
			ethpbalpha.RegisterBeaconNodeValidatorHandler,
			ethpbalpha.RegisterHealthHandler,
		}
		if enableDebugRPCEndpoints {
			v1AlphaRegistrations = append(v1AlphaRegistrations, ethpbalpha.RegisterDebugHandler)
		}
		v1AlphaMux := gwruntime.NewServeMux(
			gwruntime.WithMarshalerOption(gwruntime.MIMEWildcard, &gwruntime.HTTPBodyMarshaler{
				Marshaler: &gwruntime.JSONPb{
					MarshalOptions: protojson.MarshalOptions{
						EmitUnpopulated: true,
					},
					UnmarshalOptions: protojson.UnmarshalOptions{
						DiscardUnknown: true,
					},
				},
			}),
			gwruntime.WithMarshalerOption(
				api.EventStreamMediaType, &gwruntime.EventSourceJSONPb{},
			),
		)
		v1AlphaPbHandler = &gateway.PbMux{
			Registrations: v1AlphaRegistrations,
			Patterns:      []string{"/eth/v1alpha1/", "/eth/v1alpha2/"},
			Mux:           v1AlphaMux,
		}
	}
	if flags.EnableHTTPEthAPI(httpModules) {
		ethRegistrations := []gateway.PbHandlerRegistration{}
		ethMux := gwruntime.NewServeMux(
			gwruntime.WithMarshalerOption(gwruntime.MIMEWildcard, &gwruntime.HTTPBodyMarshaler{
				Marshaler: &gwruntime.JSONPb{
					MarshalOptions: protojson.MarshalOptions{
						UseProtoNames:   true,
						EmitUnpopulated: true,
					},
					UnmarshalOptions: protojson.UnmarshalOptions{
						DiscardUnknown: true,
					},
				},
			}),
		)
		ethPbHandler = &gateway.PbMux{
			Registrations: ethRegistrations,
			Patterns:      []string{"/internal/eth/v1/", "/internal/eth/v2/"},
			Mux:           ethMux,
		}
	}

	return MuxConfig{
		EthPbMux:     ethPbHandler,
		V1AlphaPbMux: v1AlphaPbHandler,
	}
}
