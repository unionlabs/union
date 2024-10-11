package codec

import (
	"github.com/cosmos/gogoproto/proto"
	"sigs.k8s.io/yaml"
)

// MarshalYAML marshals toPrint using JSONCodec to leverage specialized MarshalJSON methods
// (usually related to serialize data with protobuf or amin depending on a configuration).
// This involves additional roundtrip through JSON.
func MarshalYAML(cdc JSONCodec, toPrint proto.Message) ([]byte, error) {
	// We are OK with the performance hit of the additional JSON roundtip. MarshalYAML is not
	// used in any critical parts of the system.
	bz, err := cdc.MarshalJSON(toPrint)
	if err != nil {
		return nil, err
	}

	return yaml.JSONToYAML(bz)
}
