package checkpoint

import (
	"fmt"

	"github.com/urfave/cli/v2"
)

var checkpointCmd = &cli.Command{
	Name:   "latest",
	Usage:  "deprecated - please use 'prysmctl weak-subjectivity checkpoint' instead!",
	Action: cliDeprecatedLatest,
}

func cliDeprecatedLatest(_ *cli.Context) error {
	return fmt.Errorf("this command has moved. Please use 'prysmctl weak-subjectivity checkpoint' instead")
}
