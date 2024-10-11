package eth1

import (
	"context"
	"crypto/ecdsa"
	"crypto/sha256"
	"fmt"
	"math/big"
	mathRand "math/rand"
	"os"
	"time"

	"github.com/MariusVanDerWijden/FuzzyVM/filler"
	txfuzz "github.com/MariusVanDerWijden/tx-fuzz"
	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto/kzg4844"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/holiman/uint256"
	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

const txCount = 20

var fundedAccount *keystore.Key

type TransactionGenerator struct {
	keystore string
	seed     int64
	started  chan struct{}
	cancel   context.CancelFunc
}

func NewTransactionGenerator(keystore string, seed int64) *TransactionGenerator {
	return &TransactionGenerator{keystore: keystore, seed: seed}
}

func (t *TransactionGenerator) Start(ctx context.Context) error {
	// Wrap context with a cancel func
	ctx, ccl := context.WithCancel(ctx)
	t.cancel = ccl

	client, err := rpc.DialHTTP(fmt.Sprintf("http://127.0.0.1:%d", e2e.TestParams.Ports.Eth1RPCPort))
	if err != nil {
		return err
	}
	defer client.Close()

	seed := t.seed
	newGen := rand.NewDeterministicGenerator()
	if seed == 0 {
		seed = newGen.Int63()
		logrus.Infof("Seed for transaction generator is: %d", seed)
	}
	// Set seed so that all transactions can be
	// deterministically generated.
	mathRand.Seed(seed)

	keystoreBytes, err := os.ReadFile(t.keystore) // #nosec G304
	if err != nil {
		return err
	}
	mineKey, err := keystore.DecryptKey(keystoreBytes, KeystorePassword)
	if err != nil {
		return err
	}
	newKey := keystore.NewKeyForDirectICAP(newGen)
	if err := fundAccount(client, mineKey, newKey); err != nil {
		return err
	}
	fundedAccount = newKey
	rnd := make([]byte, 10000)
	// #nosec G404
	_, err = mathRand.Read(rnd)
	if err != nil {
		return err
	}
	f := filler.NewFiller(rnd)
	// Broadcast Transactions every slot
	txPeriod := time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second
	ticker := time.NewTicker(txPeriod)
	gasPrice := big.NewInt(1e11)
	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			backend := ethclient.NewClient(client)
			err = SendTransaction(client, mineKey.PrivateKey, f, gasPrice, mineKey.Address.String(), txCount, backend, false)
			if err != nil {
				return err
			}
			backend.Close()
		}
	}
}

// Started checks whether beacon node set is started and all nodes are ready to be queried.
func (s *TransactionGenerator) Started() <-chan struct{} {
	return s.started
}

func SendTransaction(client *rpc.Client, key *ecdsa.PrivateKey, f *filler.Filler, gasPrice *big.Int, addr string, N uint64, backend *ethclient.Client, al bool) error {
	sender := common.HexToAddress(addr)
	nonce, err := backend.PendingNonceAt(context.Background(), fundedAccount.Address)
	if err != nil {
		return err
	}
	chainid, err := backend.ChainID(context.Background())
	if err != nil {
		return err
	}
	expectedPrice, err := backend.SuggestGasPrice(context.Background())
	if err != nil {
		return err
	}
	if expectedPrice.Cmp(gasPrice) > 0 {
		gasPrice = expectedPrice
	}
	g, _ := errgroup.WithContext(context.Background())
	txs := make([]*types.Transaction, 10)
	for i := uint64(0); i < 10; i++ {
		index := i
		g.Go(func() error {
			tx, err := RandomBlobTx(client, f, fundedAccount.Address, nonce+index, gasPrice, chainid, al)
			if err != nil {
				logrus.WithError(err).Error("Could not create blob tx")
				// In the event the transaction constructed is not valid, we continue with the routine
				// rather than complete stop it.
				//nolint:nilerr
				return nil
			}
			signedTx, err := types.SignTx(tx, types.NewCancunSigner(chainid), fundedAccount.PrivateKey)
			if err != nil {
				logrus.WithError(err).Error("Could not sign blob tx")
				// We continue on in the event there is a reason we can't sign this
				// transaction(unlikely).
				//nolint:nilerr
				return nil
			}
			txs[index] = signedTx
			return nil
		})
	}

	if err := g.Wait(); err != nil {
		return err
	}
	for _, tx := range txs {
		if tx == nil {
			continue
		}
		err = backend.SendTransaction(context.Background(), tx)
		if err != nil {
			// Do nothing
			continue
		}
	}

	nonce, err = backend.PendingNonceAt(context.Background(), sender)
	if err != nil {
		return err
	}

	txs = make([]*types.Transaction, N)
	for i := uint64(0); i < N; i++ {
		index := i
		g.Go(func() error {
			tx, err := txfuzz.RandomValidTx(client, f, sender, nonce+index, gasPrice, chainid, al)
			if err != nil {
				// In the event the transaction constructed is not valid, we continue with the routine
				// rather than complete stop it.
				//nolint:nilerr
				return nil
			}
			signedTx, err := types.SignTx(tx, types.NewLondonSigner(chainid), key)
			if err != nil {
				// We continue on in the event there is a reason we can't sign this
				// transaction(unlikely).
				//nolint:nilerr
				return nil
			}
			txs[index] = signedTx
			return nil
		})
	}
	if err := g.Wait(); err != nil {
		return err
	}
	for _, tx := range txs {
		if tx == nil {
			continue
		}
		err = backend.SendTransaction(context.Background(), tx)
		if err != nil {
			// Do nothing
			continue
		}
	}
	return nil
}

// Pause pauses the component and its underlying process.
func (t *TransactionGenerator) Pause() error {
	return nil
}

// Resume resumes the component and its underlying process.
func (t *TransactionGenerator) Resume() error {
	return nil
}

// Stop stops the component and its underlying process.
func (t *TransactionGenerator) Stop() error {
	t.cancel()
	return nil
}

func RandomBlobTx(rpc *rpc.Client, f *filler.Filler, sender common.Address, nonce uint64, gasPrice, chainID *big.Int, al bool) (*types.Transaction, error) {
	// Set fields if non-nil
	if rpc != nil {
		client := ethclient.NewClient(rpc)
		var err error
		if gasPrice == nil {
			gasPrice, err = client.SuggestGasPrice(context.Background())
			if err != nil {
				gasPrice = big.NewInt(1)
			}
		}
		if chainID == nil {
			chainID, err = client.ChainID(context.Background())
			if err != nil {
				chainID = big.NewInt(1)
			}
		}
	}
	gas := uint64(100000)
	to := randomAddress()
	code := txfuzz.RandomCode(f)
	value := big.NewInt(0)
	if len(code) > 128 {
		code = code[:128]
	}
	mod := 2
	if al {
		mod = 1
	}
	switch f.Byte() % byte(mod) {
	case 0:
		// 4844 transaction without AL
		tip, feecap, err := getCaps(rpc, gasPrice)
		if err != nil {
			return nil, err
		}
		data, err := randomBlobData()
		if err != nil {
			return nil, err
		}
		return New4844Tx(nonce, &to, gas, chainID, tip, feecap, value, code, big.NewInt(1000000), data, make(types.AccessList, 0)), nil
	case 1:
		// 4844 transaction with AL
		tx := types.NewTransaction(nonce, to, value, gas, gasPrice, code)
		al, err := txfuzz.CreateAccessList(rpc, tx, sender)
		if err != nil {
			return nil, err
		}
		tip, feecap, err := getCaps(rpc, gasPrice)
		if err != nil {
			return nil, err
		}
		data, err := randomBlobData()
		if err != nil {
			return nil, err
		}
		return New4844Tx(nonce, &to, gas, chainID, tip, feecap, value, code, big.NewInt(1000000), data, *al), nil
	}
	return nil, errors.New("asdf")
}

func New4844Tx(nonce uint64, to *common.Address, gasLimit uint64, chainID, tip, feeCap, value *big.Int, code []byte, blobFeeCap *big.Int, blobData []byte, al types.AccessList) *types.Transaction {
	blobs, comms, proofs, versionedHashes, err := EncodeBlobs(blobData)
	if err != nil {
		panic(err)
	}
	tx := types.NewTx(&types.BlobTx{
		ChainID:    uint256.MustFromBig(chainID),
		Nonce:      nonce,
		GasTipCap:  uint256.MustFromBig(tip),
		GasFeeCap:  uint256.MustFromBig(feeCap),
		Gas:        gasLimit,
		To:         *to,
		Value:      uint256.MustFromBig(value),
		Data:       code,
		AccessList: al,
		BlobFeeCap: uint256.MustFromBig(blobFeeCap),
		BlobHashes: versionedHashes,
		Sidecar: &types.BlobTxSidecar{
			Blobs:       blobs,
			Commitments: comms,
			Proofs:      proofs,
		},
	})
	return tx
}

func encodeBlobs(data []byte) []kzg4844.Blob {
	blobs := []kzg4844.Blob{{}}
	blobIndex := 0
	fieldIndex := -1
	numOfElems := fieldparams.BlobLength / 32
	for i := 0; i < len(data); i += 31 {
		fieldIndex++
		if fieldIndex == numOfElems {
			if blobIndex >= 1 {
				break
			}
			blobs = append(blobs, kzg4844.Blob{})
			blobIndex++
			fieldIndex = 0
		}
		max := i + 31
		if max > len(data) {
			max = len(data)
		}
		copy(blobs[blobIndex][fieldIndex*32+1:], data[i:max])
	}
	return blobs
}

func EncodeBlobs(data []byte) ([]kzg4844.Blob, []kzg4844.Commitment, []kzg4844.Proof, []common.Hash, error) {
	var (
		blobs           = encodeBlobs(data)
		commits         []kzg4844.Commitment
		proofs          []kzg4844.Proof
		versionedHashes []common.Hash
	)
	for _, blob := range blobs {
		commit, err := kzg4844.BlobToCommitment(blob)
		if err != nil {
			return nil, nil, nil, nil, err
		}
		commits = append(commits, commit)

		proof, err := kzg4844.ComputeBlobProof(blob, commit)
		if err != nil {
			return nil, nil, nil, nil, err
		}
		if err := kzg4844.VerifyBlobProof(blob, commit, proof); err != nil {
			return nil, nil, nil, nil, err
		}
		proofs = append(proofs, proof)

		versionedHashes = append(versionedHashes, kZGToVersionedHash(commit))
	}
	return blobs, commits, proofs, versionedHashes, nil
}

var blobCommitmentVersionKZG uint8 = 0x01

// kZGToVersionedHash implements kzg_to_versioned_hash from EIP-4844
func kZGToVersionedHash(kzg kzg4844.Commitment) common.Hash {
	h := sha256.Sum256(kzg[:])
	h[0] = blobCommitmentVersionKZG

	return h
}

func randomBlobData() ([]byte, error) {
	// #nosec G404
	size := mathRand.Intn(fieldparams.BlobSize)
	data := make([]byte, size)
	// #nosec G404
	n, err := mathRand.Read(data)
	if err != nil {
		return nil, err
	}
	if n != size {
		return nil, fmt.Errorf("could not create random blob data with size %d: %v", size, err)
	}
	return data, nil
}

func randomAddress() common.Address {
	// #nosec G404
	switch mathRand.Int31n(5) {
	case 0, 1, 2:
		b := make([]byte, 20)
		// #nosec G404
		_, err := mathRand.Read(b)
		if err != nil {
			panic(err)
		}
		return common.BytesToAddress(b)
	case 3:
		return common.Address{}
	case 4:
		return common.HexToAddress("0xb02A2EdA1b317FBd16760128836B0Ac59B560e9D")
	}
	return common.Address{}
}

func getCaps(rpc *rpc.Client, defaultGasPrice *big.Int) (*big.Int, *big.Int, error) {
	if rpc == nil {
		tip := new(big.Int).Mul(big.NewInt(1), big.NewInt(0).SetUint64(params.BeaconConfig().GweiPerEth))
		if defaultGasPrice.Cmp(tip) >= 0 {
			feeCap := new(big.Int).Sub(defaultGasPrice, tip)
			return tip, feeCap, nil
		}
		return big.NewInt(0), defaultGasPrice, nil
	}
	client := ethclient.NewClient(rpc)
	tip, err := client.SuggestGasTipCap(context.Background())
	if err != nil {
		return nil, nil, err
	}
	feeCap, err := client.SuggestGasPrice(context.Background())
	return tip, feeCap, err
}

func fundAccount(client *rpc.Client, sourceKey, destKey *keystore.Key) error {
	backend := ethclient.NewClient(client)
	defer backend.Close()
	nonce, err := backend.PendingNonceAt(context.Background(), sourceKey.Address)
	if err != nil {
		return err
	}
	chainid, err := backend.ChainID(context.Background())
	if err != nil {
		return err
	}
	expectedPrice, err := backend.SuggestGasPrice(context.Background())
	if err != nil {
		return err
	}
	val, ok := big.NewInt(0).SetString("10000000000000000000000000", 10)
	if !ok {
		return errors.New("could not set big int for value")
	}
	tx := types.NewTransaction(nonce, destKey.Address, val, 100000, expectedPrice, nil)
	signedTx, err := types.SignTx(tx, types.NewLondonSigner(chainid), sourceKey.PrivateKey)
	if err != nil {
		return err
	}
	return backend.SendTransaction(context.Background(), signedTx)
}
