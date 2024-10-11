package cli

import (
	"fmt"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/testutil/network"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authcli "github.com/cosmos/cosmos-sdk/x/auth/client/cli"
)

// CheckTxCode verifies that the transaction result returns a specific code
// Takes a network, wait for two blocks and fetch the transaction from its hash
func CheckTxCode(network network.NetworkI, clientCtx client.Context, txHash string, expectedCode uint32) error {
	// wait for 2 blocks
	for i := 0; i < 2; i++ {
		if err := network.WaitForNextBlock(); err != nil {
			return fmt.Errorf("failed to wait for next block: %w", err)
		}
	}

	cmd := authcli.QueryTxCmd()
	out, err := ExecTestCLICmd(clientCtx, cmd, []string{txHash, fmt.Sprintf("--%s=json", flags.FlagOutput)})
	if err != nil {
		return err
	}

	var response sdk.TxResponse
	if err := clientCtx.Codec.UnmarshalJSON(out.Bytes(), &response); err != nil {
		return err
	}

	if response.Code != expectedCode {
		return fmt.Errorf("expected code %d, got %d", expectedCode, response.Code)
	}

	return nil
}

// GetTxResponse returns queries the transaction response of a transaction from its hash
// Takes a network, wait for two blocks and fetch the transaction from its hash
func GetTxResponse(network network.NetworkI, clientCtx client.Context, txHash string) (sdk.TxResponse, error) {
	// wait for 2 blocks
	for i := 0; i < 2; i++ {
		if err := network.WaitForNextBlock(); err != nil {
			return sdk.TxResponse{}, fmt.Errorf("failed to wait for next block: %w", err)
		}
	}

	cmd := authcli.QueryTxCmd()
	out, err := ExecTestCLICmd(clientCtx, cmd, []string{txHash, fmt.Sprintf("--%s=json", flags.FlagOutput)})
	if err != nil {
		return sdk.TxResponse{}, err
	}

	var response sdk.TxResponse
	if err := clientCtx.Codec.UnmarshalJSON(out.Bytes(), &response); err != nil {
		return sdk.TxResponse{}, err
	}

	return response, nil
}
