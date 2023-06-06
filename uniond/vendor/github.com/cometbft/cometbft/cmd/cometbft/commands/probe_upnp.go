package commands

import (
	"fmt"

	"github.com/spf13/cobra"

	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/p2p/upnp"
)

// ProbeUpnpCmd adds capabilities to test the UPnP functionality.
var ProbeUpnpCmd = &cobra.Command{
	Use:     "probe-upnp",
	Aliases: []string{"probe_upnp"},
	Short:   "Test UPnP functionality",
	RunE:    probeUpnp,
}

func probeUpnp(cmd *cobra.Command, args []string) error {
	capabilities, err := upnp.Probe(logger)
	if err != nil {
		fmt.Println("Probe failed: ", err)
	} else {
		fmt.Println("Probe success!")
		jsonBytes, err := cmtjson.Marshal(capabilities)
		if err != nil {
			return err
		}
		fmt.Println(string(jsonBytes))
	}
	return nil
}
