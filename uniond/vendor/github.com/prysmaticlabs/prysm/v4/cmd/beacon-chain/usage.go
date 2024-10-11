// This code was adapted from https://github.com/ethereum/go-ethereum/blob/master/cmd/geth/usage.go
package main

import (
	"io"
	"sort"

	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/storage"
	backfill "github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/sync/backfill/flags"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/sync/checkpoint"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/sync/genesis"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/runtime/debug"
	"github.com/urfave/cli/v2"
)

var appHelpTemplate = `NAME:
   {{.App.Name}} - {{.App.Usage}}
USAGE:
   {{.App.HelpName}} [options]{{if .App.Commands}} command [command options]{{end}} {{if .App.ArgsUsage}}{{.App.ArgsUsage}}{{else}}[arguments...]{{end}}
   {{if .App.Version}}
AUTHOR:
   {{range .App.Authors}}{{ . }}{{end}}
   {{end}}{{if .App.Commands}}
GLOBAL OPTIONS:
   {{range .App.Commands}}{{join .Names ", "}}{{ "\t" }}{{.Usage}}
   {{end}}{{end}}{{if .FlagGroups}}
{{range .FlagGroups}}{{.Name}} OPTIONS:
   {{range .Flags}}{{.}}
   {{end}}
{{end}}{{end}}{{if .App.Copyright }}
COPYRIGHT:
   {{.App.Copyright}}
VERSION:
   {{.App.Version}}
   {{end}}{{if len .App.Authors}}
   {{end}}
`

type flagGroup struct {
	Name  string
	Flags []cli.Flag
}

var appHelpFlagGroups = []flagGroup{
	{
		Name: "cmd",
		Flags: []cli.Flag{
			cmd.MinimalConfigFlag,
			cmd.E2EConfigFlag,
			cmd.RPCMaxPageSizeFlag,
			cmd.NoDiscovery,
			cmd.BootstrapNode,
			cmd.RelayNode,
			cmd.P2PUDPPort,
			cmd.P2PTCPPort,
			cmd.DataDirFlag,
			cmd.VerbosityFlag,
			cmd.EnableTracingFlag,
			cmd.TracingProcessNameFlag,
			cmd.TracingEndpointFlag,
			cmd.TraceSampleFractionFlag,
			cmd.MonitoringHostFlag,
			flags.MonitoringPortFlag,
			cmd.DisableMonitoringFlag,
			cmd.MaxGoroutines,
			cmd.ForceClearDB,
			cmd.ClearDB,
			cmd.ConfigFileFlag,
			cmd.ChainConfigFileFlag,
			cmd.GrpcMaxCallRecvMsgSizeFlag,
			cmd.AcceptTosFlag,
			cmd.RestoreSourceFileFlag,
			cmd.RestoreTargetDirFlag,
			cmd.ValidatorMonitorIndicesFlag,
			cmd.ApiTimeoutFlag,
		},
	},
	{
		Name: "debug",
		Flags: []cli.Flag{
			debug.PProfFlag,
			debug.PProfAddrFlag,
			debug.PProfPortFlag,
			debug.MemProfileRateFlag,
			debug.CPUProfileFlag,
			debug.TraceFlag,
			debug.BlockProfileRateFlag,
			debug.MutexProfileFractionFlag,
		},
	},
	{
		Name: "beacon-chain",
		Flags: []cli.Flag{
			flags.InteropMockEth1DataVotesFlag,
			flags.DepositContractFlag,
			flags.ContractDeploymentBlock,
			flags.RPCHost,
			flags.RPCPort,
			flags.CertFlag,
			flags.KeyFlag,
			flags.HTTPModules,
			flags.DisableGRPCGateway,
			flags.GRPCGatewayHost,
			flags.GRPCGatewayPort,
			flags.GPRCGatewayCorsDomain,
			flags.ExecutionEngineEndpoint,
			flags.ExecutionEngineHeaders,
			flags.ExecutionJWTSecretFlag,
			flags.SetGCPercent,
			flags.SlotsPerArchivedPoint,
			flags.BlockBatchLimit,
			flags.BlockBatchLimitBurstFactor,
			flags.BlobBatchLimit,
			flags.BlobBatchLimitBurstFactor,
			flags.EnableDebugRPCEndpoints,
			flags.SubscribeToAllSubnets,
			flags.HistoricalSlasherNode,
			flags.ChainID,
			flags.NetworkID,
			flags.WeakSubjectivityCheckpoint,
			flags.Eth1HeaderReqLimit,
			flags.MinPeersPerSubnet,
			flags.MevRelayEndpoint,
			flags.MaxBuilderEpochMissedSlots,
			flags.MaxBuilderConsecutiveMissedSlots,
			flags.EngineEndpointTimeoutSeconds,
			flags.SlasherDirFlag,
			flags.LocalBlockValueBoost,
			flags.JwtId,
			checkpoint.BlockPath,
			checkpoint.StatePath,
			checkpoint.RemoteURL,
			genesis.StatePath,
			genesis.BeaconAPIURL,
			storage.BlobStoragePathFlag,
			storage.BlobRetentionEpochFlag,
			backfill.EnableExperimentalBackfill,
			backfill.BackfillWorkerCount,
			backfill.BackfillBatchSize,
		},
	},
	{
		Name: "merge",
		Flags: []cli.Flag{
			flags.SuggestedFeeRecipient,
			flags.TerminalTotalDifficultyOverride,
			flags.TerminalBlockHashOverride,
			flags.TerminalBlockHashActivationEpochOverride,
		},
	},
	{
		Name: "p2p",
		Flags: []cli.Flag{
			cmd.P2PIP,
			cmd.P2PHost,
			cmd.P2PHostDNS,
			cmd.P2PMaxPeers,
			cmd.P2PPrivKey,
			cmd.P2PStaticID,
			cmd.P2PMetadata,
			cmd.P2PAllowList,
			cmd.P2PDenyList,
			cmd.PubsubQueueSize,
			cmd.StaticPeers,
			cmd.EnableUPnPFlag,
			flags.MinSyncPeers,
		},
	},
	{
		Name: "log",
		Flags: []cli.Flag{
			cmd.LogFormat,
			cmd.LogFileName,
		},
	},
	{
		Name:  "features",
		Flags: features.ActiveFlags(features.BeaconChainFlags),
	},
	{
		Name: "interop",
		Flags: []cli.Flag{
			genesis.StatePath,
			flags.InteropGenesisTimeFlag,
			flags.InteropNumValidatorsFlag,
		},
	},
	{
		Name: "deprecated",
		Flags: []cli.Flag{
			cmd.BackupWebhookOutputDir,
		},
	},
}

func init() {
	cli.AppHelpTemplate = appHelpTemplate

	type helpData struct {
		App        interface{}
		FlagGroups []flagGroup
	}

	originalHelpPrinter := cli.HelpPrinter
	cli.HelpPrinter = func(w io.Writer, tmpl string, data interface{}) {
		if tmpl == appHelpTemplate {
			for _, group := range appHelpFlagGroups {
				sort.Sort(cli.FlagsByName(group.Flags))
			}
			originalHelpPrinter(w, tmpl, helpData{data, appHelpFlagGroups})
		} else {
			originalHelpPrinter(w, tmpl, data)
		}
	}
}
