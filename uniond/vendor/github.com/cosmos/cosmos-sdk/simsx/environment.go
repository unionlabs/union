package simsx

import (
	"context"
	"errors"
	"math/rand"
	"slices"
	"time"

	"cosmossdk.io/core/address"
	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

// helper type for simple bank access
type contextAwareBalanceSource struct {
	ctx  context.Context
	bank BalanceSource
}

func (s contextAwareBalanceSource) SpendableCoins(accAddress sdk.AccAddress) sdk.Coins {
	return s.bank.SpendableCoins(s.ctx, accAddress)
}

func (s contextAwareBalanceSource) IsSendEnabledDenom(denom string) bool {
	return s.bank.IsSendEnabledDenom(s.ctx, denom)
}

// SimAccount is an extended simtypes.Account
type SimAccount struct {
	simtypes.Account
	r             *rand.Rand
	liquidBalance *SimsAccountBalance
	bank          contextAwareBalanceSource
}

// LiquidBalance spendable balance. This excludes not spendable amounts like staked or vested amounts.
func (a *SimAccount) LiquidBalance() *SimsAccountBalance {
	if a.liquidBalance == nil {
		a.liquidBalance = NewSimsAccountBalance(a, a.r, a.bank.SpendableCoins(a.Address))
	}
	return a.liquidBalance
}

// SimsAccountBalance is a helper type for common access methods to balance amounts.
type SimsAccountBalance struct {
	sdk.Coins
	owner *SimAccount
	r     *rand.Rand
}

// NewSimsAccountBalance constructor
func NewSimsAccountBalance(o *SimAccount, r *rand.Rand, coins sdk.Coins) *SimsAccountBalance {
	return &SimsAccountBalance{Coins: coins, r: r, owner: o}
}

type CoinsFilter interface {
	Accept(c sdk.Coins) bool // returns false to reject
}
type CoinsFilterFn func(c sdk.Coins) bool

func (f CoinsFilterFn) Accept(c sdk.Coins) bool {
	return f(c)
}

func WithSendEnabledCoins() CoinsFilter {
	return statefulCoinsFilterFn(func(s *SimAccount, coins sdk.Coins) bool {
		for _, coin := range coins {
			if !s.bank.IsSendEnabledDenom(coin.Denom) {
				return false
			}
		}
		return true
	})
}

// filter with context of SimAccount
type statefulCoinsFilter struct {
	s  *SimAccount
	do func(s *SimAccount, c sdk.Coins) bool
}

// constructor
func statefulCoinsFilterFn(f func(s *SimAccount, c sdk.Coins) bool) CoinsFilter {
	return &statefulCoinsFilter{do: f}
}

func (f statefulCoinsFilter) Accept(c sdk.Coins) bool {
	if f.s == nil {
		panic("account not set")
	}
	return f.do(f.s, c)
}

func (f *statefulCoinsFilter) visit(s *SimAccount) {
	f.s = s
}

var _ visitable = &statefulCoinsFilter{}

type visitable interface {
	visit(s *SimAccount)
}

// RandSubsetCoins return random amounts from the current balance. When the coins are empty, skip is called on the reporter.
// The amounts are removed from the liquid balance.
func (b *SimsAccountBalance) RandSubsetCoins(reporter SimulationReporter, filters ...CoinsFilter) sdk.Coins {
	amount := b.randomAmount(5, reporter, b.Coins, filters...)
	b.Coins = b.Coins.Sub(amount...)
	if amount.Empty() {
		reporter.Skip("got empty amounts")
	}
	return amount
}

// RandSubsetCoin return random amount from the current balance. When the coins are empty, skip is called on the reporter.
// The amount is removed from the liquid balance.
func (b *SimsAccountBalance) RandSubsetCoin(reporter SimulationReporter, denom string, filters ...CoinsFilter) sdk.Coin {
	ok, coin := b.Find(denom)
	if !ok {
		reporter.Skipf("no such coin: %s", denom)
		return sdk.NewCoin(denom, math.ZeroInt())
	}
	amounts := b.randomAmount(5, reporter, sdk.Coins{coin}, filters...)
	if amounts.Empty() {
		reporter.Skip("empty coin")
		return sdk.NewCoin(denom, math.ZeroInt())
	}
	b.BlockAmount(amounts[0])
	return amounts[0]
}

// BlockAmount returns true when balance is > requested amount and subtracts the amount from the liquid balance
func (b *SimsAccountBalance) BlockAmount(amount sdk.Coin) bool {
	ok, coin := b.Coins.Find(amount.Denom)
	if !ok || !coin.IsPositive() || !coin.IsGTE(amount) {
		return false
	}
	b.Coins = b.Coins.Sub(amount)
	return true
}

func (b *SimsAccountBalance) randomAmount(retryCount int, reporter SimulationReporter, coins sdk.Coins, filters ...CoinsFilter) sdk.Coins {
	if retryCount < 0 || b.Coins.Empty() {
		reporter.Skip("failed to find matching amount")
		return sdk.Coins{}
	}
	amount := simtypes.RandSubsetCoins(b.r, coins)
	for _, filter := range filters {
		if f, ok := filter.(visitable); ok {
			f.visit(b.owner)
		}
		if !filter.Accept(amount) {
			return b.randomAmount(retryCount-1, reporter, coins, filters...)
		}
	}
	return amount
}

func (b *SimsAccountBalance) RandFees() sdk.Coins {
	amount, err := simtypes.RandomFees(b.r, b.Coins)
	if err != nil {
		return sdk.Coins{}
	}
	return amount
}

type SimAccountFilter interface {
	// Accept returns true to accept the account or false to reject
	Accept(a SimAccount) bool
}
type SimAccountFilterFn func(a SimAccount) bool

func (s SimAccountFilterFn) Accept(a SimAccount) bool {
	return s(a)
}

func ExcludeAccounts(others ...SimAccount) SimAccountFilter {
	return SimAccountFilterFn(func(a SimAccount) bool {
		return !slices.ContainsFunc(others, func(o SimAccount) bool {
			return a.Address.Equals(o.Address)
		})
	})
}

// UniqueAccounts returns a stateful filter that rejects duplicate accounts.
// It uses a map to keep track of accounts that have been processed.
// If an account exists in the map, the filter function returns false
// to reject a duplicate, else it adds the account to the map and returns true.
//
// Example usage:
//
//	uniqueAccountsFilter := simsx.UniqueAccounts()
//
//	for {
//	    from := testData.AnyAccount(reporter, uniqueAccountsFilter)
//	    //... rest of the loop
//	}
func UniqueAccounts() SimAccountFilter {
	idx := make(map[string]struct{})
	return SimAccountFilterFn(func(a SimAccount) bool {
		if _, ok := idx[a.AddressBech32]; ok {
			return false
		}
		idx[a.AddressBech32] = struct{}{}
		return true
	})
}

func ExcludeAddresses(addrs ...string) SimAccountFilter {
	return SimAccountFilterFn(func(a SimAccount) bool {
		return !slices.Contains(addrs, a.AddressBech32)
	})
}

func WithDenomBalance(denom string) SimAccountFilter {
	return SimAccountFilterFn(func(a SimAccount) bool {
		return a.LiquidBalance().AmountOf(denom).IsPositive()
	})
}

func WithLiquidBalanceGTE(amount ...sdk.Coin) SimAccountFilter {
	return SimAccountFilterFn(func(a SimAccount) bool {
		return a.LiquidBalance().IsAllGTE(amount)
	})
}

// WithSpendableBalance Filters for liquid token but send may not be enabled for all or any
func WithSpendableBalance() SimAccountFilter {
	return SimAccountFilterFn(func(a SimAccount) bool {
		return !a.LiquidBalance().Empty()
	})
}

type ModuleAccountSource interface {
	GetModuleAddress(moduleName string) sdk.AccAddress
}

// BalanceSource is an interface for retrieving balance-related information for a given account.
type BalanceSource interface {
	SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	IsSendEnabledDenom(ctx context.Context, denom string) bool
}

// ChainDataSource provides common sims test data and helper methods
type ChainDataSource struct {
	r                         *rand.Rand
	addressToAccountsPosIndex map[string]int
	accounts                  []SimAccount
	accountSource             ModuleAccountSource
	addressCodec              address.Codec
	bank                      contextAwareBalanceSource
}

// NewChainDataSource constructor
func NewChainDataSource(
	ctx context.Context,
	r *rand.Rand,
	ak ModuleAccountSource,
	bk BalanceSource,
	codec address.Codec,
	oldSimAcc ...simtypes.Account,
) *ChainDataSource {
	acc := make([]SimAccount, len(oldSimAcc))
	index := make(map[string]int, len(oldSimAcc))
	bank := contextAwareBalanceSource{ctx: ctx, bank: bk}
	for i, a := range oldSimAcc {
		acc[i] = SimAccount{Account: a, r: r, bank: bank}
		index[a.AddressBech32] = i
		if a.AddressBech32 == "" {
			panic("test account has empty bech32 address")
		}
	}
	return &ChainDataSource{
		r:                         r,
		accountSource:             ak,
		addressCodec:              codec,
		accounts:                  acc,
		bank:                      bank,
		addressToAccountsPosIndex: index,
	}
}

// AnyAccount returns a random SimAccount matching the filter criteria. Module accounts are excluded.
// In case of an error or no matching account found, the reporter is set to skip and an empty value is returned.
func (c *ChainDataSource) AnyAccount(r SimulationReporter, filters ...SimAccountFilter) SimAccount {
	acc := c.randomAccount(r, 5, filters...)
	return acc
}

// GetAccountbyAccAddr return SimAccount with given binary address. Reporter skip flag is set when not found.
func (c ChainDataSource) GetAccountbyAccAddr(reporter SimulationReporter, addr sdk.AccAddress) SimAccount {
	if len(addr) == 0 {
		reporter.Skip("can not find account for empty address")
		return c.nullAccount()
	}
	addrStr, err := c.addressCodec.BytesToString(addr)
	if err != nil {
		reporter.Skipf("can not convert account address to string: %s", err)
		return c.nullAccount()
	}
	return c.GetAccount(reporter, addrStr)
}

func (c ChainDataSource) HasAccount(addr string) bool {
	_, ok := c.addressToAccountsPosIndex[addr]
	return ok
}

// GetAccount return SimAccount with given bench32 address. Reporter skip flag is set when not found.
func (c ChainDataSource) GetAccount(reporter SimulationReporter, addr string) SimAccount {
	pos, ok := c.addressToAccountsPosIndex[addr]
	if !ok {
		reporter.Skipf("no account: %s", addr)
		return c.nullAccount()
	}
	return c.accounts[pos]
}

func (c *ChainDataSource) randomAccount(reporter SimulationReporter, retryCount int, filters ...SimAccountFilter) SimAccount {
	if retryCount < 0 {
		reporter.Skip("failed to find a matching account")
		return c.nullAccount()
	}
	idx := c.r.Intn(len(c.accounts))
	acc := c.accounts[idx]
	for _, filter := range filters {
		if !filter.Accept(acc) {
			return c.randomAccount(reporter, retryCount-1, filters...)
		}
	}
	return acc
}

// create null object
func (c ChainDataSource) nullAccount() SimAccount {
	return SimAccount{
		Account:       simtypes.Account{},
		r:             c.r,
		liquidBalance: &SimsAccountBalance{},
		bank:          c.accounts[0].bank,
	}
}

func (c *ChainDataSource) ModuleAccountAddress(reporter SimulationReporter, moduleName string) string {
	acc := c.accountSource.GetModuleAddress(moduleName)
	if acc == nil {
		reporter.Skipf("unknown module account: %s", moduleName)
		return ""
	}
	res, err := c.addressCodec.BytesToString(acc)
	if err != nil {
		reporter.Skipf("failed to encode module address: %s", err)
		return ""
	}
	return res
}

func (c *ChainDataSource) AddressCodec() address.Codec {
	return c.addressCodec
}

func (c *ChainDataSource) Rand() *XRand {
	return &XRand{c.r}
}

func (c *ChainDataSource) IsSendEnabledDenom(denom string) bool {
	return c.bank.IsSendEnabledDenom(denom)
}

// AllAccounts returns all accounts in legacy format
func (c *ChainDataSource) AllAccounts() []simtypes.Account {
	return Collect(c.accounts, func(a SimAccount) simtypes.Account { return a.Account })
}

func (c *ChainDataSource) AccountsCount() int {
	return len(c.accounts)
}

// AccountAt return SimAccount within the accounts slice. Reporter skip flag is set when boundaries are exceeded.

func (c *ChainDataSource) AccountAt(reporter SimulationReporter, i int) SimAccount {
	if i > len(c.accounts) {
		reporter.Skipf("account index out of range: %d", i)
		return c.nullAccount()
	}
	return c.accounts[i]
}

type XRand struct {
	*rand.Rand
}

// NewXRand constructor
func NewXRand(rand *rand.Rand) *XRand {
	return &XRand{Rand: rand}
}

func (r *XRand) StringN(max int) string {
	return simtypes.RandStringOfLength(r.Rand, max)
}

func (r *XRand) SubsetCoins(src sdk.Coins) sdk.Coins {
	return simtypes.RandSubsetCoins(r.Rand, src)
}

// Coin return one coin from the list
func (r *XRand) Coin(src sdk.Coins) sdk.Coin {
	return src[r.Intn(len(src))]
}

func (r *XRand) DecN(max math.LegacyDec) math.LegacyDec {
	return simtypes.RandomDecAmount(r.Rand, max)
}

func (r *XRand) IntInRange(min, max int) int {
	return r.Rand.Intn(max-min) + min
}

// Uint64InRange returns a pseudo-random uint64 number in the range [min, max].
// It panics when min >= max
func (r *XRand) Uint64InRange(min, max uint64) uint64 {
	return uint64(r.Rand.Int63n(int64(max-min)) + int64(min))
}

// Uint32InRange returns a pseudo-random uint32 number in the range [min, max].
// It panics when min >= max
func (r *XRand) Uint32InRange(min, max uint32) uint32 {
	return uint32(r.Rand.Intn(int(max-min))) + min
}

func (r *XRand) PositiveSDKIntn(max math.Int) (math.Int, error) {
	return simtypes.RandPositiveInt(r.Rand, max)
}

func (r *XRand) PositiveSDKIntInRange(min, max math.Int) (math.Int, error) {
	diff := max.Sub(min)
	if !diff.IsPositive() {
		return math.Int{}, errors.New("min value must not be greater or equal to max")
	}
	result, err := r.PositiveSDKIntn(diff)
	if err != nil {
		return math.Int{}, err
	}
	return result.Add(min), nil
}

// Timestamp returns a timestamp between  Jan 1, 2062 and Jan 1, 2262
func (r *XRand) Timestamp() time.Time {
	return simtypes.RandTimestamp(r.Rand)
}

func (r *XRand) Bool() bool {
	return r.Intn(100) > 50
}

func (r *XRand) Amount(balance math.Int) math.Int {
	return simtypes.RandomAmount(r.Rand, balance)
}
