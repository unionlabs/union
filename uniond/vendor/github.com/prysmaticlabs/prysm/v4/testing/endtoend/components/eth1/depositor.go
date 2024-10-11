package eth1

import (
	"context"
	"fmt"
	"math/big"
	"os"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/common"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	contracts "github.com/prysmaticlabs/prysm/v4/contracts/deposit"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	log "github.com/sirupsen/logrus"
)

var gweiPerEth = big.NewInt(int64(params.BeaconConfig().GweiPerEth))

func amtInGwei(deposit *eth.Deposit) *big.Int {
	amt := big.NewInt(0).SetUint64(deposit.Data.Amount)
	return amt.Mul(amt, gweiPerEth)
}

// computeDeposits uses the deterministic validator generator to generate deposits for `nvals` (number of validators).
// To control which validators should receive deposits, so that we can generate deposits at different stages of e2e,
// the `offset` parameter skips the first N validators in the deterministic list.
// In order to test the requirement that our deposit follower is able to handle multiple partial deposits,
// the `partial` flag specifies that half of the deposits should be broken up into 2 transactions.
func computeDeposits(offset, nvals int, partial bool) ([]*eth.Deposit, error) {
	balances := make([]uint64, offset+nvals)
	partialIndex := len(balances) // set beyond loop invariant so by default nothing gets partial
	if partial {
		// Validators in this range will get 2 half (MAX_EFFECTIVE_BALANCE/2) deposits.
		// Upper half of the range of desired validator indices is used because these can be easily
		// duplicated with a slice from this offset to the end.
		partialIndex = offset + nvals/2
	}
	// Start setting values at `offset`. Lower elements will be discarded - setting them to zero
	// ensures they don't slip through and add extra deposits.
	for i := offset; i < len(balances); i++ {
		if i >= partialIndex {
			// these deposits will be duplicated, resulting in 2 deposits summing to MAX_EFFECTIVE_BALANCE
			balances[i] = params.BeaconConfig().MaxEffectiveBalance / 2
		} else {
			balances[i] = params.BeaconConfig().MaxEffectiveBalance
		}
	}
	deposits, _, err := util.DepositsWithBalance(balances)
	if err != nil {
		return []*eth.Deposit{}, err
	}

	// if partial = false, these will be a no-op (partialIndex == len(deposits)),
	// otherwise it will duplicate the partial deposits.
	deposits = append(deposits[offset:], deposits[partialIndex:]...)

	return deposits, nil
}

type Depositor struct {
	// The EmptyComponent type is embedded in the Depositor so that it satisfies the ComponentRunner interface.
	// This allows other components or e2e set up code to block until its Start method has been called.
	types.EmptyComponent
	Key       *keystore.Key
	Client    *ethclient.Client
	ChainID   *big.Int
	NetworkId *big.Int
	cd        *contracts.DepositContract
	sent      *DepositHistory
}

var _ types.ComponentRunner = &Depositor{}

// History exposes the DepositHistory value for a Depositor.
func (d *Depositor) History() *DepositHistory {
	if d.sent == nil {
		d.sent = &DepositHistory{}
	}
	return d.sent
}

// DepositHistory is a type used by Depositor to keep track of batches of deposit for test assertion purposes.
type DepositHistory struct {
	sync.RWMutex
	byBatch  map[types.DepositBatch][]*SentDeposit
	deposits []*SentDeposit
}

func (h *DepositHistory) record(sd *SentDeposit) {
	h.Lock()
	defer h.Unlock()
	h.deposits = append(h.deposits, sd)
	h.updateByBatch(sd)
}

func (h *DepositHistory) updateByBatch(sd *SentDeposit) {
	if h.byBatch == nil {
		h.byBatch = make(map[types.DepositBatch][]*SentDeposit)
	}
	h.byBatch[sd.batch] = append(h.byBatch[sd.batch], sd)
}

// Balances sums, by validator, all deposit amounts that were sent as part of the given batch.
// This can be used in e2e evaluators to check that the results of deposit transactions are visible on chain.
func (h *DepositHistory) Balances(batch types.DepositBatch) map[[48]byte]uint64 {
	balances := make(map[[48]byte]uint64)
	h.RLock()
	defer h.RUnlock()
	for _, d := range h.byBatch[batch] {
		k := bytesutil.ToBytes48(d.deposit.Data.PublicKey)
		if _, ok := balances[k]; !ok {
			balances[k] = d.deposit.Data.Amount
		} else {
			balances[k] += d.deposit.Data.Amount
		}
	}
	return balances
}

// SentDeposit is the record of an individual deposit which has been successfully submitted as a transaction.
type SentDeposit struct {
	root    [32]byte
	deposit *eth.Deposit
	tx      *gethtypes.Transaction
	time    time.Time
	batch   types.DepositBatch
}

// SendAndMine uses the deterministic validator generator to generate deposits for `nvals` (number of validators).
// To control which validators should receive deposits, so that we can generate deposits at different stages of e2e,
// the `offset` parameter skips the first N validators in the deterministic list.
// In order to test the requirement that our deposit follower is able to handle multiple partial deposits,
// the `partial` flag specifies that half of the deposits should be broken up into 2 transactions.
// Once the set of deposits has been generated, it submits a transaction for each deposit
// (using 2 transactions for partial deposits) and then uses WaitForBlocks (which spams the miner node with transactions
// to and from its own address) to advance the chain until it has moved forward ETH1_FOLLOW_DISTANCE blocks.
func (d *Depositor) SendAndMine(ctx context.Context, offset, nvals int, batch types.DepositBatch, partial bool) error {
	balance, err := d.Client.BalanceAt(ctx, d.Key.Address, nil)
	if err != nil {
		return err
	}
	log.WithField("balance", balance.String()).WithField("account", d.Key.Address.Hex()).Info("SendAndMine balance check")
	// This is the "Send" part of the function. Compute deposits for `nvals` validators,
	// with half of those deposits being split over 2 transactions if the `partial` flag is true,
	// and throwing away any validators before `offset`.
	deposits, err := computeDeposits(offset, nvals, partial)
	if err != nil {
		return err
	}

	txo, err := d.txops(ctx)
	if err != nil {
		return err
	}
	for _, dd := range deposits {
		if err := d.SendDeposit(dd, txo, batch); err != nil {
			return err
		}
	}

	// This is the "AndMine" part of the function. WaitForBlocks will spam transactions to/from the given key
	// to advance the EL chain and until the chain has advanced the requested amount.
	if err = WaitForBlocks(d.Client, d.Key, params.BeaconConfig().Eth1FollowDistance); err != nil {
		return fmt.Errorf("failed to mine blocks %w", err)
	}
	return nil
}

// SendDeposit sends a single deposit. A record of this deposit will be tracked for the life of the Depositor,
// allowing evaluators to use the deposit history to make assertions about those deposits.
func (d *Depositor) SendDeposit(dep *eth.Deposit, txo *bind.TransactOpts, batch types.DepositBatch) error {
	contract, err := d.contractDepositor()
	if err != nil {
		return err
	}
	txo.Value = amtInGwei(dep)
	root, err := dep.Data.HashTreeRoot()
	if err != nil {
		return err
	}
	sent := time.Now()
	tx, err := contract.Deposit(txo, dep.Data.PublicKey, dep.Data.WithdrawalCredentials, dep.Data.Signature, root)
	if err != nil {
		return errors.Wrap(err, "unable to send transaction to contract")
	}
	d.History().record(&SentDeposit{deposit: dep, time: sent, tx: tx, root: root, batch: batch})
	txo.Nonce = txo.Nonce.Add(txo.Nonce, big.NewInt(1))
	return nil
}

// txops is a little helper method that creates a TransactOpts (type encapsulating auth/meta data needed to make a tx).
func (d *Depositor) txops(ctx context.Context) (*bind.TransactOpts, error) {
	txo, err := bind.NewKeyedTransactorWithChainID(d.Key.PrivateKey, d.NetworkId)
	if err != nil {
		return nil, err
	}
	txo.Context = ctx
	txo.GasLimit = e2e.DepositGasLimit
	nonce, err := d.Client.PendingNonceAt(ctx, txo.From)
	if err != nil {
		return nil, err
	}
	txo.Nonce = big.NewInt(0).SetUint64(nonce)
	return txo, nil
}

// contractDepositor is a little helper method that inits and caches a DepositContract value.
// DepositContract is a special-purpose client for calling the deposit contract.
func (d *Depositor) contractDepositor() (*contracts.DepositContract, error) {
	if d.cd == nil {
		addr := common.HexToAddress(params.BeaconConfig().DepositContractAddress)
		contract, err := contracts.NewDepositContract(addr, d.Client)
		if err != nil {
			return nil, err
		}
		d.cd = contract
	}
	return d.cd, nil
}

func (d *Depositor) UnderlyingProcess() *os.Process {
	return nil // No subprocess for this component.
}
