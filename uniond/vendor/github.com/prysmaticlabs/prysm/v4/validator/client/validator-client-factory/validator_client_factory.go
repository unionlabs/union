package validator_client_factory

import (
	"github.com/prysmaticlabs/prysm/v4/config/features"
	beaconApi "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api"
	grpcApi "github.com/prysmaticlabs/prysm/v4/validator/client/grpc-api"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
	validatorHelpers "github.com/prysmaticlabs/prysm/v4/validator/helpers"
)

func NewValidatorClient(
	validatorConn validatorHelpers.NodeConnection,
	jsonRestHandler beaconApi.JsonRestHandler,
	opt ...beaconApi.ValidatorClientOpt,
) iface.ValidatorClient {
	if features.Get().EnableBeaconRESTApi {
		return beaconApi.NewBeaconApiValidatorClient(jsonRestHandler, opt...)
	} else {
		return grpcApi.NewGrpcValidatorClient(validatorConn.GetGrpcClientConn())
	}
}
