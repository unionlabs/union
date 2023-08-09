package server

// DONTCOVER

import (
	"fmt"

	"github.com/cometbft/cometbft/light"
	"github.com/cometbft/cometbft/node"
	"github.com/cometbft/cometbft/p2p"
	pvm "github.com/cometbft/cometbft/privval"
	cmtstore "github.com/cometbft/cometbft/proto/tendermint/store"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/statesync"
	"github.com/cometbft/cometbft/store"
	tversion "github.com/cometbft/cometbft/version"
	"github.com/spf13/cobra"
	"sigs.k8s.io/yaml"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ShowNodeIDCmd - ported from Tendermint, dump node ID to stdout
func ShowNodeIDCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "show-node-id",
		Short: "Show this node's ID",
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config

			nodeKey, err := p2p.LoadNodeKey(cfg.NodeKeyFile())
			if err != nil {
				return err
			}

			fmt.Println(nodeKey.ID())
			return nil
		},
	}
}

// ShowValidatorCmd - ported from Tendermint, show this node's validator info
func ShowValidatorCmd() *cobra.Command {
	cmd := cobra.Command{
		Use:   "show-validator",
		Short: "Show this node's tendermint validator info",
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config

			privValidator := pvm.LoadFilePV(cfg.PrivValidatorKeyFile(), cfg.PrivValidatorStateFile())
			pk, err := privValidator.GetPubKey()
			if err != nil {
				return err
			}

			sdkPK, err := cryptocodec.FromTmPubKeyInterface(pk)
			if err != nil {
				return err
			}

			clientCtx := client.GetClientContextFromCmd(cmd)
			bz, err := clientCtx.Codec.MarshalInterfaceJSON(sdkPK)
			if err != nil {
				return err
			}

			fmt.Println(string(bz))
			return nil
		},
	}

	return &cmd
}

// ShowAddressCmd - show this node's validator address
func ShowAddressCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "show-address",
		Short: "Shows this node's tendermint validator consensus address",
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config

			privValidator := pvm.LoadFilePV(cfg.PrivValidatorKeyFile(), cfg.PrivValidatorStateFile())

			valConsAddr := (sdk.ConsAddress)(privValidator.GetAddress())
			fmt.Println(valConsAddr.String())
			return nil
		},
	}

	return cmd
}

// VersionCmd prints tendermint and ABCI version numbers.
func VersionCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "version",
		Short: "Print tendermint libraries' version",
		Long:  "Print protocols' and libraries' version numbers against which this app has been compiled.",
		RunE: func(cmd *cobra.Command, args []string) error {
			bs, err := yaml.Marshal(&struct {
				Tendermint    string
				ABCI          string
				BlockProtocol uint64
				P2PProtocol   uint64
			}{
				Tendermint:    tversion.TMCoreSemVer,
				ABCI:          tversion.ABCIVersion,
				BlockProtocol: tversion.BlockProtocol,
				P2PProtocol:   tversion.P2PProtocol,
			})
			if err != nil {
				return err
			}

			fmt.Println(string(bs))
			return nil
		},
	}
}

func BootstrapStateCmd(appCreator types.AppCreator) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "bootstrap-state",
		Short: "Bootstrap CometBFT state at an arbitrary block height using a light client",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config

			height, err := cmd.Flags().GetInt64("height")
			if err != nil {
				return err
			}
			if height == 0 {
				home := serverCtx.Viper.GetString(flags.FlagHome)
				db, err := openDB(home, GetAppDBBackend(serverCtx.Viper))
				if err != nil {
					return err
				}

				app := appCreator(serverCtx.Logger, db, nil, serverCtx.Viper)
				height = app.CommitMultiStore().LastCommitID().Version
			}

			blockStoreDB, err := node.DefaultDBProvider(&node.DBContext{ID: "blockstore", Config: cfg})
			if err != nil {
				return err
			}
			blockStore := store.NewBlockStore(blockStoreDB)

			stateDB, err := node.DefaultDBProvider(&node.DBContext{ID: "state", Config: cfg})
			if err != nil {
				return err
			}
			stateStore := sm.NewStore(stateDB, sm.StoreOptions{
				DiscardABCIResponses: cfg.Storage.DiscardABCIResponses,
			})

			genState, _, err := node.LoadStateFromDBOrGenesisDocProvider(stateDB, node.DefaultGenesisDocProviderFunc(cfg))
			if err != nil {
				return err
			}

			stateProvider, err := statesync.NewLightClientStateProvider(
				cmd.Context(),
				genState.ChainID, genState.Version, genState.InitialHeight,
				cfg.StateSync.RPCServers, light.TrustOptions{
					Period: cfg.StateSync.TrustPeriod,
					Height: cfg.StateSync.TrustHeight,
					Hash:   cfg.StateSync.TrustHashBytes(),
				}, serverCtx.Logger.With("module", "light"))
			if err != nil {
				return fmt.Errorf("failed to set up light client state provider: %w", err)
			}

			state, err := stateProvider.State(cmd.Context(), uint64(height))
			if err != nil {
				return fmt.Errorf("failed to get state: %w", err)
			}

			commit, err := stateProvider.Commit(cmd.Context(), uint64(height))
			if err != nil {
				return fmt.Errorf("failed to get commit: %w", err)
			}

			if err := stateStore.Bootstrap(state); err != nil {
				return fmt.Errorf("failed to bootstrap state: %w", err)
			}

			if err := blockStore.SaveSeenCommit(state.LastBlockHeight, commit); err != nil {
				return fmt.Errorf("failed to save seen commit: %w", err)
			}

			store.SaveBlockStoreState(&cmtstore.BlockStoreState{
				// it breaks the invariant that blocks in range [Base, Height] must exists, but it do works in practice.
				Base:   state.LastBlockHeight,
				Height: state.LastBlockHeight,
			}, blockStoreDB)

			return nil
		},
	}

	cmd.Flags().Int64("height", 0, "Block height to bootstrap state at, if not provided it uses the latest block height in app state")

	return cmd
}
