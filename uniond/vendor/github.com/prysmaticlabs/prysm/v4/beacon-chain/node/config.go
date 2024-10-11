package node

import (
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	fastssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	tracing2 "github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/urfave/cli/v2"
)

func configureTracing(cliCtx *cli.Context) error {
	return tracing2.Setup(
		"beacon-chain", // service name
		cliCtx.String(cmd.TracingProcessNameFlag.Name),
		cliCtx.String(cmd.TracingEndpointFlag.Name),
		cliCtx.Float64(cmd.TraceSampleFractionFlag.Name),
		cliCtx.Bool(cmd.EnableTracingFlag.Name),
	)
}

func configureChainConfig(cliCtx *cli.Context) error {
	if cliCtx.IsSet(cmd.ChainConfigFileFlag.Name) {
		chainConfigFileName := cliCtx.String(cmd.ChainConfigFileFlag.Name)
		return params.LoadChainConfigFile(chainConfigFileName, nil)
	}
	return nil
}

func configureHistoricalSlasher(cliCtx *cli.Context) error {
	if cliCtx.Bool(flags.HistoricalSlasherNode.Name) {
		c := params.BeaconConfig().Copy()
		// Save a state every 4 epochs.
		c.SlotsPerArchivedPoint = params.BeaconConfig().SlotsPerEpoch * 4
		if err := params.SetActive(c); err != nil {
			return err
		}
		cmdConfig := cmd.Get()
		// Allow up to 4096 attestations at a time to be requested from the beacon node.
		cmdConfig.MaxRPCPageSize = int(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().MaxAttestations)) // lint:ignore uintcast -- Page size should not exceed int64 with these constants.
		cmd.Init(cmdConfig)
		log.Warnf(
			"Setting %d slots per archive point and %d max RPC page size for historical slasher usage. This requires additional storage",
			c.SlotsPerArchivedPoint,
			cmdConfig.MaxRPCPageSize,
		)
	}
	return nil
}

func configureBuilderCircuitBreaker(cliCtx *cli.Context) error {
	if cliCtx.IsSet(flags.MaxBuilderConsecutiveMissedSlots.Name) {
		c := params.BeaconConfig().Copy()
		c.MaxBuilderConsecutiveMissedSlots = primitives.Slot(cliCtx.Int(flags.MaxBuilderConsecutiveMissedSlots.Name))
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	if cliCtx.IsSet(flags.MaxBuilderEpochMissedSlots.Name) {
		c := params.BeaconConfig().Copy()
		c.MaxBuilderEpochMissedSlots = primitives.Slot(cliCtx.Int(flags.MaxBuilderEpochMissedSlots.Name))
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	if cliCtx.IsSet(flags.LocalBlockValueBoost.Name) {
		c := params.BeaconConfig().Copy()
		c.LocalBlockValueBoost = cliCtx.Uint64(flags.LocalBlockValueBoost.Name)
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	return nil
}

func configureSlotsPerArchivedPoint(cliCtx *cli.Context) error {
	if cliCtx.IsSet(flags.SlotsPerArchivedPoint.Name) {
		c := params.BeaconConfig().Copy()
		c.SlotsPerArchivedPoint = primitives.Slot(cliCtx.Int(flags.SlotsPerArchivedPoint.Name))
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	return nil
}

func configureEth1Config(cliCtx *cli.Context) error {
	c := params.BeaconConfig().Copy()
	if cliCtx.IsSet(flags.ChainID.Name) {
		c.DepositChainID = cliCtx.Uint64(flags.ChainID.Name)
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	if cliCtx.IsSet(flags.NetworkID.Name) {
		c.DepositNetworkID = cliCtx.Uint64(flags.NetworkID.Name)
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	if cliCtx.IsSet(flags.EngineEndpointTimeoutSeconds.Name) {
		c.ExecutionEngineTimeoutValue = cliCtx.Uint64(flags.EngineEndpointTimeoutSeconds.Name)
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	if cliCtx.IsSet(flags.DepositContractFlag.Name) {
		c.DepositContractAddress = cliCtx.String(flags.DepositContractFlag.Name)
		if err := params.SetActive(c); err != nil {
			return err
		}
	}
	return nil
}

func configureNetwork(cliCtx *cli.Context) {
	if len(cliCtx.StringSlice(cmd.BootstrapNode.Name)) > 0 {
		c := params.BeaconNetworkConfig()
		c.BootstrapNodes = cliCtx.StringSlice(cmd.BootstrapNode.Name)
		params.OverrideBeaconNetworkConfig(c)
	}
	if cliCtx.IsSet(flags.ContractDeploymentBlock.Name) {
		networkCfg := params.BeaconNetworkConfig()
		networkCfg.ContractDeploymentBlock = uint64(cliCtx.Int(flags.ContractDeploymentBlock.Name))
		params.OverrideBeaconNetworkConfig(networkCfg)
	}
}

func configureInteropConfig(cliCtx *cli.Context) error {
	// an explicit chain config was specified, don't mess with it
	if cliCtx.IsSet(cmd.ChainConfigFileFlag.Name) {
		return nil
	}
	genTimeIsSet := cliCtx.IsSet(flags.InteropGenesisTimeFlag.Name)
	numValsIsSet := cliCtx.IsSet(flags.InteropNumValidatorsFlag.Name)
	votesIsSet := cliCtx.IsSet(flags.InteropMockEth1DataVotesFlag.Name)

	if genTimeIsSet || numValsIsSet || votesIsSet {
		if err := params.SetActive(params.InteropConfig().Copy()); err != nil {
			return err
		}
	}
	return nil
}

func configureExecutionSetting(cliCtx *cli.Context) error {
	if cliCtx.IsSet(flags.TerminalTotalDifficultyOverride.Name) {
		c := params.BeaconConfig()
		c.TerminalTotalDifficulty = cliCtx.String(flags.TerminalTotalDifficultyOverride.Name)
		log.WithField("terminal block difficult", c.TerminalTotalDifficulty).Warn("Terminal block difficult overridden")
		params.OverrideBeaconConfig(c)
	}
	if cliCtx.IsSet(flags.TerminalBlockHashOverride.Name) {
		c := params.BeaconConfig()
		c.TerminalBlockHash = common.HexToHash(cliCtx.String(flags.TerminalBlockHashOverride.Name))
		log.WithField("terminal block hash", c.TerminalBlockHash.Hex()).Warn("Terminal block hash overridden")
		params.OverrideBeaconConfig(c)
	}
	if cliCtx.IsSet(flags.TerminalBlockHashActivationEpochOverride.Name) {
		c := params.BeaconConfig()
		c.TerminalBlockHashActivationEpoch = primitives.Epoch(cliCtx.Uint64(flags.TerminalBlockHashActivationEpochOverride.Name))
		log.WithField("terminal block hash activation epoch", c.TerminalBlockHashActivationEpoch).Warn("Terminal block hash activation epoch overridden")
		params.OverrideBeaconConfig(c)
	}

	if !cliCtx.IsSet(flags.SuggestedFeeRecipient.Name) {
		log.Warnf("In order to receive transaction fees from proposing blocks, " +
			"you must provide flag --" + flags.SuggestedFeeRecipient.Name + " with a valid ethereum address when starting your beacon node. " +
			"Please see our documentation for more information on this requirement (https://docs.prylabs.network/docs/execution-node/fee-recipient).")
		return nil
	}

	c := params.BeaconConfig().Copy()
	ha := cliCtx.String(flags.SuggestedFeeRecipient.Name)
	if !common.IsHexAddress(ha) {
		log.Warnf("%s is not a valid fee recipient address, setting suggested-fee-recipient failed", ha)
		return nil
	}
	mixedcaseAddress, err := common.NewMixedcaseAddressFromString(ha)
	if err != nil {
		log.WithError(err).Error(fmt.Sprintf("Could not decode fee recipient %s, setting suggested-fee-recipient failed", ha))
		return nil
	}
	checksumAddress := common.HexToAddress(ha)
	if !mixedcaseAddress.ValidChecksum() {
		log.Warnf("Fee recipient %s is not a checksum Ethereum address. "+
			"The checksummed address is %s and will be used as the fee recipient. "+
			"We recommend using a mixed-case address (checksum) "+
			"to prevent spelling mistakes in your fee recipient Ethereum address", ha, checksumAddress.Hex())
	}
	c.DefaultFeeRecipient = checksumAddress
	log.Infof("Default fee recipient is set to %s, recipient may be overwritten from validator client and persist in db."+
		" Default fee recipient will be used as a fall back", checksumAddress.Hex())
	return params.SetActive(c)
}

func configureFastSSZHashingAlgorithm() {
	fastssz.EnableVectorizedHTR = true
}
