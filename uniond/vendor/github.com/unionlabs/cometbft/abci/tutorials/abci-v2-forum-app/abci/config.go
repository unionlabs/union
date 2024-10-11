package abci

import (
	"errors"
	"fmt"

	"github.com/BurntSushi/toml"
)

type Config struct {
	ChainID    string `toml:"chain_id"`
	CurseWords string `toml:"curse_words"`
}

func LoadConfig(file string) (*Config, error) {
	cfg := &Config{
		ChainID:    "forum_chain",
		CurseWords: "bad|apple|muggles",
	}
	_, err := toml.DecodeFile(file, &cfg)
	if err != nil {
		return nil, fmt.Errorf("failed to load config from %q: %w", file, err)
	}
	return cfg, cfg.Validate()
}

// Validate validates the configuration. We don't do exhaustive config
// validation here, instead relying on Testnet.Validate() to handle it.
func (cfg Config) Validate() error {
	switch {
	case cfg.ChainID == "":
		return errors.New("chain_id parameter is required")
	default:
		return nil
	}
}
