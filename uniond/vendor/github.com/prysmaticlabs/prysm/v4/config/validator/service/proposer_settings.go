package validator_service_config

import (
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
)

// ToSettings converts struct to ProposerSettings
func ToSettings(ps *validatorpb.ProposerSettingsPayload) (*ProposerSettings, error) {
	settings := &ProposerSettings{}
	if ps.ProposerConfig != nil {
		settings.ProposeConfig = make(map[[fieldparams.BLSPubkeyLength]byte]*ProposerOption)
		for key, optionPayload := range ps.ProposerConfig {
			if optionPayload.FeeRecipient == "" {
				continue
			}
			b, err := hexutil.Decode(key)
			if err != nil {
				return nil, errors.Wrap(err, fmt.Sprintf("cannot decode public key %s", key))
			}
			p := &ProposerOption{
				FeeRecipientConfig: &FeeRecipientConfig{
					FeeRecipient: common.HexToAddress(optionPayload.FeeRecipient),
				},
			}
			if optionPayload.Builder != nil {
				p.BuilderConfig = ToBuilderConfig(optionPayload.Builder)
			}
			settings.ProposeConfig[bytesutil.ToBytes48(b)] = p
		}
	}
	if ps.DefaultConfig != nil {
		d := &ProposerOption{}
		if ps.DefaultConfig.FeeRecipient != "" {
			d.FeeRecipientConfig = &FeeRecipientConfig{
				FeeRecipient: common.HexToAddress(ps.DefaultConfig.FeeRecipient),
			}
		}
		if ps.DefaultConfig.Builder != nil {
			d.BuilderConfig = ToBuilderConfig(ps.DefaultConfig.Builder)
		}
		settings.DefaultConfig = d
	}
	return settings, nil
}

// BuilderConfig is the struct representation of the JSON config file set in the validator through the CLI.
// GasLimit is a number set to help the network decide on the maximum gas in each block.
type BuilderConfig struct {
	Enabled  bool             `json:"enabled" yaml:"enabled"`
	GasLimit validator.Uint64 `json:"gas_limit,omitempty" yaml:"gas_limit,omitempty"`
	Relays   []string         `json:"relays,omitempty" yaml:"relays,omitempty"`
}

// ToBuilderConfig converts protobuf to a builder config used in inmemory storage
func ToBuilderConfig(from *validatorpb.BuilderConfig) *BuilderConfig {
	if from == nil {
		return nil
	}
	config := &BuilderConfig{
		Enabled:  from.Enabled,
		GasLimit: from.GasLimit,
	}
	if from.Relays != nil {
		relays := make([]string, len(from.Relays))
		copy(relays, from.Relays)
		config.Relays = relays
	}

	return config
}

// ProposerSettings is a Prysm internal representation of the fee recipient config on the validator client.
// validatorpb.ProposerSettingsPayload maps to ProposerSettings on import through the CLI.
type ProposerSettings struct {
	ProposeConfig map[[fieldparams.BLSPubkeyLength]byte]*ProposerOption
	DefaultConfig *ProposerOption
}

// ShouldBeSaved goes through checks to see if the value should be saveable
// Pseudocode: conditions for being saved into the database
// 1. settings are not nil
// 2. proposeconfig is not nil (this defines specific settings for each validator key), default config can be nil in this case and fall back to beacon node settings
// 3. defaultconfig is not nil, meaning it has at least fee recipient settings (this defines general settings for all validator keys but keys will use settings from propose config if available), propose config can be nil in this case
func (settings *ProposerSettings) ShouldBeSaved() bool {
	return settings != nil && (settings.ProposeConfig != nil || settings.DefaultConfig != nil && settings.DefaultConfig.FeeRecipientConfig != nil)
}

// ToPayload converts struct to ProposerSettingsPayload
func (ps *ProposerSettings) ToPayload() *validatorpb.ProposerSettingsPayload {
	if ps == nil {
		return nil
	}
	payload := &validatorpb.ProposerSettingsPayload{}
	if ps.ProposeConfig != nil {
		payload.ProposerConfig = make(map[string]*validatorpb.ProposerOptionPayload)
		for key, option := range ps.ProposeConfig {
			p := &validatorpb.ProposerOptionPayload{}
			if option.FeeRecipientConfig != nil {
				p.FeeRecipient = option.FeeRecipientConfig.FeeRecipient.Hex()
			}
			if option.BuilderConfig != nil {
				p.Builder = option.BuilderConfig.ToPayload()
			}
			payload.ProposerConfig[hexutil.Encode(key[:])] = p
		}
	}
	if ps.DefaultConfig != nil {
		p := &validatorpb.ProposerOptionPayload{}
		if ps.DefaultConfig.FeeRecipientConfig != nil {
			p.FeeRecipient = ps.DefaultConfig.FeeRecipientConfig.FeeRecipient.Hex()
		}
		if ps.DefaultConfig.BuilderConfig != nil {
			p.Builder = ps.DefaultConfig.BuilderConfig.ToPayload()
		}
		payload.DefaultConfig = p
	}
	return payload
}

// FeeRecipientConfig is a prysm internal representation to see if the fee recipient was set.
type FeeRecipientConfig struct {
	FeeRecipient common.Address
}

// ProposerOption is a Prysm internal representation of the ProposerOptionPayload on the validator client in bytes format instead of hex.
type ProposerOption struct {
	FeeRecipientConfig *FeeRecipientConfig
	BuilderConfig      *BuilderConfig
}

// Clone creates a deep copy of the proposer settings
func (ps *ProposerSettings) Clone() *ProposerSettings {
	if ps == nil {
		return nil
	}
	clone := &ProposerSettings{}
	if ps.DefaultConfig != nil {
		clone.DefaultConfig = ps.DefaultConfig.Clone()
	}
	if ps.ProposeConfig != nil {
		clone.ProposeConfig = make(map[[fieldparams.BLSPubkeyLength]byte]*ProposerOption)
		for k, v := range ps.ProposeConfig {
			keyCopy := k
			valCopy := v.Clone()
			clone.ProposeConfig[keyCopy] = valCopy
		}
	}

	return clone
}

// Clone creates a deep copy of fee recipient config
func (fo *FeeRecipientConfig) Clone() *FeeRecipientConfig {
	if fo == nil {
		return nil
	}
	return &FeeRecipientConfig{fo.FeeRecipient}
}

// Clone creates a deep copy of builder config
func (bc *BuilderConfig) Clone() *BuilderConfig {
	if bc == nil {
		return nil
	}
	config := &BuilderConfig{}
	config.Enabled = bc.Enabled
	config.GasLimit = bc.GasLimit
	var relays []string
	if bc.Relays != nil {
		relays = make([]string, len(bc.Relays))
		copy(relays, bc.Relays)
		config.Relays = relays
	}
	return config
}

// ToPayload converts Builder Config to the protobuf object
func (bc *BuilderConfig) ToPayload() *validatorpb.BuilderConfig {
	if bc == nil {
		return nil
	}
	config := &validatorpb.BuilderConfig{}
	config.Enabled = bc.Enabled
	var relays []string
	if bc.Relays != nil {
		relays = make([]string, len(bc.Relays))
		copy(relays, bc.Relays)
		config.Relays = relays
	}
	config.GasLimit = bc.GasLimit
	return config
}

// Clone creates a deep copy of proposer option
func (po *ProposerOption) Clone() *ProposerOption {
	if po == nil {
		return nil
	}
	p := &ProposerOption{}
	if po.FeeRecipientConfig != nil {
		p.FeeRecipientConfig = po.FeeRecipientConfig.Clone()
	}
	if po.BuilderConfig != nil {
		p.BuilderConfig = po.BuilderConfig.Clone()
	}
	return p
}
