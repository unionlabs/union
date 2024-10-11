package flags

// via https://github.com/urfave/cli/issues/602

import (
	"fmt"
	"strings"

	"github.com/urfave/cli/v2"
)

// EnumValue allows the cli to present a fixed set of string values.
type EnumValue struct {
	Name        string
	Usage       string
	Destination *string
	Enum        []string
	Value       string
}

func (e *EnumValue) Set(value string) error {
	for _, enum := range e.Enum {
		if enum == value {
			*e.Destination = value
			return nil
		}
	}

	return fmt.Errorf("allowed values are %s", strings.Join(e.Enum, ", "))
}

func (e *EnumValue) String() string {
	if e.Destination == nil {
		return e.Value
	}
	if *e.Destination == "" {
		return e.Value
	}
	return *e.Destination
}

// GenericFlag wraps the EnumValue in a GenericFlag value so that it satisfies the cli.Flag interface.
func (e EnumValue) GenericFlag() *cli.GenericFlag {
	*e.Destination = e.Value
	var i cli.Generic = &e
	return &cli.GenericFlag{Name: e.Name, Usage: e.Usage, Destination: i, Value: i}
}
