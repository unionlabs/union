package accounts

import (
	"io"
	"strings"

	"github.com/golang/protobuf/ptypes/empty"
	"github.com/pkg/errors"
	grpcutil "github.com/prysmaticlabs/prysm/v4/api/grpc"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/client"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	"github.com/prysmaticlabs/prysm/v4/validator/node"
	"github.com/urfave/cli/v2"
	"google.golang.org/grpc"
)

func Exit(c *cli.Context, r io.Reader) error {
	var w *wallet.Wallet
	var km keymanager.IKeymanager
	var err error
	dialOpts := client.ConstructDialOptions(
		c.Int(cmd.GrpcMaxCallRecvMsgSizeFlag.Name),
		c.String(flags.CertFlag.Name),
		c.Uint(flags.GrpcRetriesFlag.Name),
		c.Duration(flags.GrpcRetryDelayFlag.Name),
	)
	grpcHeaders := strings.Split(c.String(flags.GrpcHeadersFlag.Name), ",")
	beaconRPCProvider := c.String(flags.BeaconRPCProviderFlag.Name)
	if !c.IsSet(flags.Web3SignerURLFlag.Name) && !c.IsSet(flags.WalletDirFlag.Name) && !c.IsSet(flags.InteropNumValidators.Name) {
		return errors.Errorf("No validators found, please provide a prysm wallet directory via flag --%s "+
			"or a web3signer location with corresponding public keys via flags --%s and --%s ",
			flags.WalletDirFlag.Name,
			flags.Web3SignerURLFlag.Name,
			flags.Web3SignerPublicValidatorKeysFlag,
		)
	}
	if c.IsSet(flags.InteropNumValidators.Name) {
		km, err = local.NewInteropKeymanager(c.Context, c.Uint64(flags.InteropStartIndex.Name), c.Uint64(flags.InteropNumValidators.Name))
		if err != nil {
			return errors.Wrap(err, "could not generate interop keys for key manager")
		}
		w = &wallet.Wallet{}
	} else if c.IsSet(flags.Web3SignerURLFlag.Name) {
		ctx := grpcutil.AppendHeaders(c.Context, grpcHeaders)
		conn, err := grpc.DialContext(ctx, beaconRPCProvider, dialOpts...)
		if err != nil {
			return errors.Wrapf(err, "could not dial endpoint %s", beaconRPCProvider)
		}
		nodeClient := ethpb.NewNodeClient(conn)
		resp, err := nodeClient.GetGenesis(c.Context, &empty.Empty{})
		if err != nil {
			return errors.Wrapf(err, "failed to get genesis info")
		}
		if err := conn.Close(); err != nil {
			log.WithError(err).Error("Failed to close connection")
		}
		config, err := node.Web3SignerConfig(c)
		if err != nil {
			return errors.Wrapf(err, "could not configure web3signer")
		}
		config.GenesisValidatorsRoot = resp.GenesisValidatorsRoot
		w, km, err = walletWithWeb3SignerKeymanager(c, config)
		if err != nil {
			return err
		}
	} else {
		w, km, err = walletWithKeymanager(c)
		if err != nil {
			return err
		}
	}

	opts := []accounts.Option{
		accounts.WithWallet(w),
		accounts.WithKeymanager(km),
		accounts.WithGRPCDialOpts(dialOpts),
		accounts.WithBeaconRPCProvider(beaconRPCProvider),
		accounts.WithBeaconRESTApiProvider(c.String(flags.BeaconRESTApiProviderFlag.Name)),
		accounts.WithGRPCHeaders(grpcHeaders),
		accounts.WithExitJSONOutputPath(c.String(flags.VoluntaryExitJSONOutputPath.Name)),
	}
	// Get full set of public keys from the keymanager.
	validatingPublicKeys, err := km.FetchValidatingPublicKeys(c.Context)
	if err != nil {
		return err
	}
	if len(validatingPublicKeys) == 0 {
		return errors.New("wallet is empty, no accounts to delete")
	}
	// Filter keys either from CLI flag or from interactive session.
	rawPubKey, formattedPubKeys, err := accounts.FilterExitAccountsFromUserInput(c, r, validatingPublicKeys, c.Bool(flags.ForceExitFlag.Name))
	if err != nil {
		return errors.Wrap(err, "could not filter public keys for deletion")
	}
	opts = append(opts, accounts.WithRawPubKeys(rawPubKey))
	opts = append(opts, accounts.WithFormattedPubKeys(formattedPubKeys))
	acc, err := accounts.NewCLIManager(opts...)
	if err != nil {
		return err
	}
	return acc.Exit(c.Context)
}
