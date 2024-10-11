package graffiti

import (
	"encoding/hex"
	"os"
	"strings"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"gopkg.in/yaml.v2"
)

const (
	hexGraffitiPrefix = "hex"
	hex0xPrefix       = "0x"
)

// Graffiti is a graffiti container.
type Graffiti struct {
	Hash     [32]byte
	Default  string                               `yaml:"default,omitempty"`
	Ordered  []string                             `yaml:"ordered,omitempty"`
	Random   []string                             `yaml:"random,omitempty"`
	Specific map[primitives.ValidatorIndex]string `yaml:"specific,omitempty"`
}

// ParseGraffitiFile parses the graffiti file and returns the graffiti struct.
func ParseGraffitiFile(f string) (*Graffiti, error) {
	yamlFile, err := os.ReadFile(f) // #nosec G304
	if err != nil {
		return nil, err
	}
	g := &Graffiti{}
	if err := yaml.UnmarshalStrict(yamlFile, g); err != nil {
		if _, ok := err.(*yaml.TypeError); !ok {
			return nil, err
		} else {
			log.WithError(err).Error("There were some issues parsing graffiti from a yaml file.")
		}
	}

	for i, o := range g.Specific {
		g.Specific[i] = ParseHexGraffiti(o)
	}

	for i, v := range g.Ordered {
		g.Ordered[i] = ParseHexGraffiti(v)
	}

	for i, v := range g.Random {
		g.Random[i] = ParseHexGraffiti(v)
	}

	g.Default = ParseHexGraffiti(g.Default)
	g.Hash = hash.Hash(yamlFile)

	return g, nil
}

// ParseHexGraffiti checks if a graffiti input is being represented in hex and converts it to ASCII if so
func ParseHexGraffiti(rawGraffiti string) string {
	splitGraffiti := strings.SplitN(rawGraffiti, ":", 2)
	if strings.ToLower(splitGraffiti[0]) == hexGraffitiPrefix {
		target := splitGraffiti[1]
		if target == "" {
			log.WithField("graffiti", rawGraffiti).Debug("Blank hex tag to be interpreted as itself")
			return rawGraffiti
		}
		if len(target) > 3 && target[:2] == hex0xPrefix {
			target = target[2:]
		}
		if target == "" {
			log.WithField("graffiti", rawGraffiti).Debug("Nothing after 0x prefix, hex tag to be interpreted as itself")
			return rawGraffiti
		}
		graffiti, err := hex.DecodeString(target)
		if err != nil {
			log.WithError(err).Debug("Error while decoding hex string")
			return rawGraffiti
		}
		return string(graffiti)
	}
	return rawGraffiti
}
