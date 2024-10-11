package testsuite

import (
	"bytes"
	"context"
	"errors"
	"fmt"

	abcicli "github.com/cometbft/cometbft/abci/client"
	"github.com/cometbft/cometbft/abci/types"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
)

func InitChain(ctx context.Context, client abcicli.Client) error {
	total := 10
	vals := make([]types.ValidatorUpdate, total)
	for i := 0; i < total; i++ {
		pubkey := cmtrand.Bytes(33)
		power := cmtrand.Int()
		vals[i] = types.ValidatorUpdate{Power: int64(power), PubKeyType: "", PubKeyBytes: pubkey}
	}
	_, err := client.InitChain(ctx, &types.InitChainRequest{
		Validators: vals,
	})
	if err != nil {
		fmt.Printf("Failed test: InitChain - %v\n", err)
		return err
	}
	fmt.Println("Passed test: InitChain")
	return nil
}

func Commit(ctx context.Context, client abcicli.Client) error {
	_, err := client.Commit(ctx, &types.CommitRequest{})
	if err != nil {
		fmt.Println("Failed test: Commit")
		fmt.Printf("error while committing: %v\n", err)
		return err
	}
	fmt.Println("Passed test: Commit")
	return nil
}

func FinalizeBlock(ctx context.Context, client abcicli.Client, txBytes [][]byte, codeExp []uint32, dataExp []byte, hashExp []byte) error {
	res, _ := client.FinalizeBlock(ctx, &types.FinalizeBlockRequest{Txs: txBytes})
	appHash := res.AppHash
	for i, tx := range res.TxResults {
		code, data, log := tx.Code, tx.Data, tx.Log
		if code != codeExp[i] {
			fmt.Println("Failed test: FinalizeBlock")
			fmt.Printf("FinalizeBlock response code was unexpected. Got %v expected %v. Log: %v\n",
				code, codeExp, log)
			return errors.New("FinalizeBlock error")
		}
		if !bytes.Equal(data, dataExp) {
			fmt.Println("Failed test:  FinalizeBlock")
			fmt.Printf("FinalizeBlock response data was unexpected. Got %X expected %X\n",
				data, dataExp)
			return errors.New("FinalizeBlock  error")
		}
	}
	if !bytes.Equal(appHash, hashExp) {
		fmt.Println("Failed test: FinalizeBlock")
		fmt.Printf("Application hash was unexpected. Got %X expected %X\n", appHash, hashExp)
		return errors.New("FinalizeBlock  error")
	}
	fmt.Println("Passed test: FinalizeBlock")
	return nil
}

func PrepareProposal(ctx context.Context, client abcicli.Client, txBytes [][]byte, txExpected [][]byte, _ []byte) error {
	res, _ := client.PrepareProposal(ctx, &types.PrepareProposalRequest{Txs: txBytes})
	for i, tx := range res.Txs {
		if !bytes.Equal(tx, txExpected[i]) {
			fmt.Println("Failed test: PrepareProposal")
			fmt.Printf("PrepareProposal transaction was unexpected. Got %x expected %x.",
				tx, txExpected[i])
			return errors.New("PrepareProposal error")
		}
	}
	fmt.Println("Passed test: PrepareProposal")
	return nil
}

func ProcessProposal(ctx context.Context, client abcicli.Client, txBytes [][]byte, statusExp types.ProcessProposalStatus) error {
	res, _ := client.ProcessProposal(ctx, &types.ProcessProposalRequest{Txs: txBytes})
	if res.Status != statusExp {
		fmt.Println("Failed test: ProcessProposal")
		fmt.Printf("ProcessProposal response status was unexpected. Got %v expected %v.",
			res.Status, statusExp)
		return errors.New("ProcessProposal error")
	}
	fmt.Println("Passed test: ProcessProposal")
	return nil
}

func CheckTx(ctx context.Context, client abcicli.Client, txBytes []byte, codeExp uint32, dataExp []byte) error {
	res, _ := client.CheckTx(ctx, &types.CheckTxRequest{Tx: txBytes, Type: types.CHECK_TX_TYPE_CHECK})
	code, data, log := res.Code, res.Data, res.Log
	if code != codeExp {
		fmt.Println("Failed test: CheckTx")
		fmt.Printf("CheckTx response code was unexpected. Got %v expected %v. Log: %v\n",
			code, codeExp, log)
		return errors.New("checkTx")
	}
	if !bytes.Equal(data, dataExp) {
		fmt.Println("Failed test: CheckTx")
		fmt.Printf("CheckTx response data was unexpected. Got %X expected %X\n",
			data, dataExp)
		return errors.New("checkTx")
	}
	fmt.Println("Passed test: CheckTx")
	return nil
}
