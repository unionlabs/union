package client

import (
	"fmt"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	_ AccountRetriever = TestAccountRetriever{}
	_ Account          = TestAccount{}
)

// TestAccount represents a client Account that can be used in unit tests
type TestAccount struct {
	Address sdk.AccAddress
	Num     uint64
	Seq     uint64
}

// GetAddress implements client Account.GetAddress
func (t TestAccount) GetAddress() sdk.AccAddress {
	return t.Address
}

// GetPubKey implements client Account.GetPubKey
func (t TestAccount) GetPubKey() cryptotypes.PubKey {
	return nil
}

// GetAccountNumber implements client Account.GetAccountNumber
func (t TestAccount) GetAccountNumber() uint64 {
	return t.Num
}

// GetSequence implements client Account.GetSequence
func (t TestAccount) GetSequence() uint64 {
	return t.Seq
}

// TestAccountRetriever is an AccountRetriever that can be used in unit tests
type TestAccountRetriever struct {
	Accounts map[string]TestAccount
}

// GetAccount implements AccountRetriever.GetAccount
func (t TestAccountRetriever) GetAccount(_ Context, addr sdk.AccAddress) (Account, error) {
	acc, ok := t.Accounts[addr.String()]
	if !ok {
		return nil, fmt.Errorf("account %s not found", addr)
	}

	return acc, nil
}

// GetAccountWithHeight implements AccountRetriever.GetAccountWithHeight
func (t TestAccountRetriever) GetAccountWithHeight(clientCtx Context, addr sdk.AccAddress) (Account, int64, error) {
	acc, err := t.GetAccount(clientCtx, addr)
	if err != nil {
		return nil, 0, err
	}

	return acc, 0, nil
}

// EnsureExists implements AccountRetriever.EnsureExists
func (t TestAccountRetriever) EnsureExists(_ Context, addr sdk.AccAddress) error {
	_, ok := t.Accounts[addr.String()]
	if !ok {
		return fmt.Errorf("account %s not found", addr)
	}
	return nil
}

// GetAccountNumberSequence implements AccountRetriever.GetAccountNumberSequence
func (t TestAccountRetriever) GetAccountNumberSequence(_ Context, addr sdk.AccAddress) (accNum uint64, accSeq uint64, err error) {
	acc, ok := t.Accounts[addr.String()]
	if !ok {
		return 0, 0, fmt.Errorf("account %s not found", addr)
	}
	return acc.Num, acc.Seq, nil
}
