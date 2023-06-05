package cli

import (
	"github.com/cosmos/cosmos-sdk/x/gov/client/cli"
	gov "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	"github.com/cosmos/cosmos-sdk/x/upgrade/types"
	"github.com/spf13/pflag"
)

func parseArgsToContent(fs *pflag.FlagSet, name string) (gov.Content, error) {
	title, err := fs.GetString(cli.FlagTitle)
	if err != nil {
		return nil, err
	}

	description, err := fs.GetString(cli.FlagDescription) //nolint:staticcheck // we are intentionally using a deprecated flag here.
	if err != nil {
		return nil, err
	}

	height, err := fs.GetInt64(FlagUpgradeHeight)
	if err != nil {
		return nil, err
	}

	info, err := fs.GetString(FlagUpgradeInfo)
	if err != nil {
		return nil, err
	}

	plan := types.Plan{Name: name, Height: height, Info: info}
	content := types.NewSoftwareUpgradeProposal(title, description, plan)
	return content, nil
}
