package accounts

import (
	"context"
	"fmt"
	"math"

	"github.com/pkg/errors"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
)

// List pretty-prints accounts in the wallet.
func (acm *CLIManager) List(ctx context.Context) error {
	if acm.listValidatorIndices {
		client, _, err := acm.prepareBeaconClients(ctx)
		if err != nil {
			return err
		}
		return listValidatorIndices(ctx, acm.keymanager, *client)
	}
	return acm.keymanager.ListKeymanagerAccounts(ctx,
		keymanager.ListKeymanagerAccountConfig{
			ShowDepositData:          acm.showDepositData,
			ShowPrivateKeys:          acm.showPrivateKeys,
			WalletAccountsDir:        acm.wallet.AccountsDir(),
			KeymanagerConfigFileName: wallet.KeymanagerConfigFileName,
		})
}

func listValidatorIndices(ctx context.Context, km keymanager.IKeymanager, client iface.ValidatorClient) error {
	pubKeys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return errors.Wrap(err, "could not get validating public keys")
	}
	var pks [][]byte
	for i := range pubKeys {
		pks = append(pks, pubKeys[i][:])
	}
	req := &ethpb.MultipleValidatorStatusRequest{PublicKeys: pks}
	resp, err := client.MultipleValidatorStatus(ctx, req)
	if err != nil {
		return errors.Wrap(err, "could not request validator indices")
	}
	fmt.Println(au.BrightGreen("Validator indices:").Bold())
	for i, idx := range resp.Indices {
		if idx != math.MaxUint64 {
			fmt.Printf("%#x: %d\n", pubKeys[i][0:4], idx)
		}
	}
	return nil
}
