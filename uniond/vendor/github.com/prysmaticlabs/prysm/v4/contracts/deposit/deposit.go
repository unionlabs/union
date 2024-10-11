// Package depositutil contains useful functions for dealing
// with Ethereum deposit inputs.
package deposit

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// DepositInput for a given key. This input data can be used to when making a
// validator deposit. The input data includes a proof of possession field
// signed by the deposit key.
//
// Spec details about general deposit workflow:
//
//	To submit a deposit:
//
//	- Pack the validator's initialization parameters into deposit_data, a Deposit_Data SSZ object.
//	- Let amount be the amount in Gwei to be deposited by the validator where MIN_DEPOSIT_AMOUNT <= amount <= MAX_EFFECTIVE_BALANCE.
//	- Set deposit_data.amount = amount.
//	- Let signature be the result of bls_sign of the signing_root(deposit_data) with domain=compute_domain(DOMAIN_DEPOSIT). (Deposits are valid regardless of fork version, compute_domain will default to zeroes there).
//	- Send a transaction on the Ethereum 1.0 chain to DEPOSIT_CONTRACT_ADDRESS executing `deposit(pubkey: bytes[48], withdrawal_credentials: bytes[32], signature: bytes[96])` along with a deposit of amount Gwei.
//
// See: https://github.com/ethereum/consensus-specs/blob/master/specs/validator/0_beacon-chain-validator.md#submit-deposit
func DepositInput(depositKey, withdrawalKey bls.SecretKey, amountInGwei uint64) (*ethpb.Deposit_Data, [32]byte, error) {
	depositMessage := &ethpb.DepositMessage{
		PublicKey:             depositKey.PublicKey().Marshal(),
		WithdrawalCredentials: WithdrawalCredentialsHash(withdrawalKey),
		Amount:                amountInGwei,
	}

	sr, err := depositMessage.HashTreeRoot()
	if err != nil {
		return nil, [32]byte{}, err
	}

	domain, err := signing.ComputeDomain(
		params.BeaconConfig().DomainDeposit,
		nil, /*forkVersion*/
		nil, /*genesisValidatorsRoot*/
	)
	if err != nil {
		return nil, [32]byte{}, err
	}
	root, err := (&ethpb.SigningData{ObjectRoot: sr[:], Domain: domain}).HashTreeRoot()
	if err != nil {
		return nil, [32]byte{}, err
	}
	di := &ethpb.Deposit_Data{
		PublicKey:             depositMessage.PublicKey,
		WithdrawalCredentials: depositMessage.WithdrawalCredentials,
		Amount:                depositMessage.Amount,
		Signature:             depositKey.Sign(root[:]).Marshal(),
	}

	dr, err := di.HashTreeRoot()
	if err != nil {
		return nil, [32]byte{}, err
	}

	return di, dr, nil
}

// WithdrawalCredentialsHash forms a 32 byte hash of the withdrawal public
// address.
//
// The specification is as follows:
//
//	withdrawal_credentials[:1] == BLS_WITHDRAWAL_PREFIX_BYTE
//	withdrawal_credentials[1:] == hash(withdrawal_pubkey)[1:]
//
// where withdrawal_credentials is of type bytes32.
func WithdrawalCredentialsHash(withdrawalKey bls.SecretKey) []byte {
	h := hash.Hash(withdrawalKey.PublicKey().Marshal())
	return append([]byte{params.BeaconConfig().BLSWithdrawalPrefixByte}, h[1:]...)[:32]
}

// VerifyDepositSignature verifies the correctness of Eth1 deposit BLS signature
func VerifyDepositSignature(dd *ethpb.Deposit_Data, domain []byte) error {
	ddCopy := ethpb.CopyDepositData(dd)
	publicKey, err := bls.PublicKeyFromBytes(ddCopy.PublicKey)
	if err != nil {
		return errors.Wrap(err, "could not convert bytes to public key")
	}
	sig, err := bls.SignatureFromBytes(ddCopy.Signature)
	if err != nil {
		return errors.Wrap(err, "could not convert bytes to signature")
	}
	di := &ethpb.DepositMessage{
		PublicKey:             ddCopy.PublicKey,
		WithdrawalCredentials: ddCopy.WithdrawalCredentials,
		Amount:                ddCopy.Amount,
	}
	root, err := di.HashTreeRoot()
	if err != nil {
		return errors.Wrap(err, "could not get signing root")
	}
	signingData := &ethpb.SigningData{
		ObjectRoot: root[:],
		Domain:     domain,
	}
	ctrRoot, err := signingData.HashTreeRoot()
	if err != nil {
		return errors.Wrap(err, "could not get container root")
	}
	if !sig.Verify(publicKey, ctrRoot[:]) {
		return signing.ErrSigFailedToVerify
	}
	return nil
}
