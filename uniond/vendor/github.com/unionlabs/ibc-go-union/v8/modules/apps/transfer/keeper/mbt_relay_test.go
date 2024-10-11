package keeper_test

// This file is a test driver for model-based tests generated from the TLA+ model of token transfer
// Written by Andrey Kuprianov within the scope of IBC Audit performed by Informal Systems.
// In case of any questions please don't hesitate to contact andrey@informal.systems.

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cometbft/cometbft/crypto"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

type TlaBalance struct {
	Address []string `json:"address"`
	Denom   []string `json:"denom"`
	Amount  int64    `json:"amount"`
}

type TlaFungibleTokenPacketData struct {
	Sender   string   `json:"sender"`
	Receiver string   `json:"receiver"`
	Amount   string   `json:"amount"`
	Denom    []string `json:"denom"`
}

type TlaFungibleTokenPacket struct {
	SourceChannel string                     `json:"sourceChannel"`
	SourcePort    string                     `json:"sourcePort"`
	DestChannel   string                     `json:"destChannel"`
	DestPort      string                     `json:"destPort"`
	Data          TlaFungibleTokenPacketData `json:"data"`
}

type TlaOnRecvPacketTestCase = struct {
	// The required subset of bank balances
	BankBefore []TlaBalance `json:"bankBefore"`
	// The packet to process
	Packet TlaFungibleTokenPacket `json:"packet"`
	// The handler to call
	Handler string `json:"handler"`
	// The expected changes in the bank
	BankAfter []TlaBalance `json:"bankAfter"`
	// Whether OnRecvPacket should fail or not
	Error bool `json:"error"`
}

type FungibleTokenPacket struct {
	SourceChannel string
	SourcePort    string
	DestChannel   string
	DestPort      string
	Data          types.FungibleTokenPacketData
}

type OnRecvPacketTestCase = struct {
	description string
	// The required subset of bank balances
	bankBefore []Balance
	// The packet to process
	packet FungibleTokenPacket
	// The handler to call
	handler string
	// The expected bank state after processing (wrt. bankBefore)
	bankAfter []Balance
	// Whether OnRecvPacket should pass or fail
	pass bool
}

type OwnedCoin struct {
	Address string
	Denom   string
}

type Balance struct {
	ID      string
	Address string
	Denom   string
	Amount  sdkmath.Int
}

func AddressFromString(address string) string {
	return sdk.AccAddress(crypto.AddressHash([]byte(address))).String()
}

func AddressFromTla(addr []string) string {
	if len(addr) != 3 {
		panic(errors.New("failed to convert from TLA+ address: wrong number of address components"))
	}
	s := ""
	if len(addr[0]) == 0 && len(addr[1]) == 0 { //nolint:gocritic
		// simple address: id
		s = addr[2]
	} else if len(addr[2]) == 0 {
		// escrow address: ics20-1\x00port/channel
		s = fmt.Sprintf("%s\x00%s/%s", types.Version, addr[0], addr[1])
	} else {
		panic(errors.New("failed to convert from TLA+ address: neither simple nor escrow address"))
	}
	return s
}

func DenomFromTla(denom []string) string {
	var i int
	for i = 0; i+1 < len(denom); i += 2 {
		if !(len(denom[i]) == 0 && len(denom[i+1]) == 0) {
			break
		}
	}
	return strings.Join(denom[i:], "/")
}

func BalanceFromTla(balance TlaBalance) Balance {
	return Balance{
		ID:      AddressFromTla(balance.Address),
		Address: AddressFromString(AddressFromTla(balance.Address)),
		Denom:   DenomFromTla(balance.Denom),
		Amount:  sdkmath.NewInt(balance.Amount),
	}
}

func BalancesFromTla(tla []TlaBalance) []Balance {
	balances := make([]Balance, 0)
	for _, b := range tla {
		balances = append(balances, BalanceFromTla(b))
	}
	return balances
}

func FungibleTokenPacketFromTla(packet TlaFungibleTokenPacket) FungibleTokenPacket {
	return FungibleTokenPacket{
		SourceChannel: packet.SourceChannel,
		SourcePort:    packet.SourcePort,
		DestChannel:   packet.DestChannel,
		DestPort:      packet.DestPort,
		Data: types.NewFungibleTokenPacketData(
			DenomFromTla(packet.Data.Denom),
			packet.Data.Amount,
			AddressFromString(packet.Data.Sender),
			AddressFromString(packet.Data.Receiver),
			""),
	}
}

func OnRecvPacketTestCaseFromTla(tc TlaOnRecvPacketTestCase) OnRecvPacketTestCase {
	return OnRecvPacketTestCase{
		description: "auto-generated",
		bankBefore:  BalancesFromTla(tc.BankBefore),
		packet:      FungibleTokenPacketFromTla(tc.Packet),
		handler:     tc.Handler,
		bankAfter:   BalancesFromTla(tc.BankAfter), // TODO different semantics
		pass:        !tc.Error,
	}
}

var addressMap = make(map[string]string)

type Bank struct {
	balances map[OwnedCoin]sdkmath.Int
}

// Make an empty bank
func MakeBank() Bank {
	return Bank{balances: make(map[OwnedCoin]sdkmath.Int)}
}

// Subtract other bank from this bank
func (bank *Bank) Sub(other *Bank) Bank {
	diff := MakeBank()
	for coin, amount := range bank.balances {
		otherAmount, exists := other.balances[coin]
		if exists {
			diff.balances[coin] = amount.Sub(otherAmount)
		} else {
			diff.balances[coin] = amount
		}
	}
	for coin, amount := range other.balances {
		if _, exists := bank.balances[coin]; !exists {
			diff.balances[coin] = amount.Neg()
		}
	}
	return diff
}

// Set specific bank balance
func (bank *Bank) SetBalance(address string, denom string, amount sdkmath.Int) {
	bank.balances[OwnedCoin{address, denom}] = amount
}

// Set several balances at once
func (bank *Bank) SetBalances(balances []Balance) {
	for _, balance := range balances {
		bank.balances[OwnedCoin{balance.Address, balance.Denom}] = balance.Amount
		addressMap[balance.Address] = balance.ID
	}
}

func NullCoin() OwnedCoin {
	return OwnedCoin{
		Address: AddressFromString(""),
		Denom:   "",
	}
}

// Set several balances at once
func BankFromBalances(balances []Balance) Bank {
	bank := MakeBank()
	for _, balance := range balances {
		coin := OwnedCoin{balance.Address, balance.Denom}
		if coin != NullCoin() { // ignore null coin
			bank.balances[coin] = balance.Amount
			addressMap[balance.Address] = balance.ID
		}
	}
	return bank
}

// String representation of all bank balances
func (bank *Bank) String() string {
	str := ""
	for coin, amount := range bank.balances {
		str += coin.Address
		if addressMap[coin.Address] != "" {
			str += "(" + addressMap[coin.Address] + ")"
		}
		str += " : " + coin.Denom + " = " + amount.String() + "\n"
	}
	return str
}

// String representation of non-zero bank balances
func (bank *Bank) NonZeroString() string {
	str := ""
	for coin, amount := range bank.balances {
		if !amount.IsZero() {
			str += coin.Address + " : " + coin.Denom + " = " + amount.String() + "\n"
		}
	}
	return str
}

// Construct a bank out of the chain bank
func BankOfChain(chain *ibctesting.TestChain) Bank {
	bank := MakeBank()
	chain.GetSimApp().BankKeeper.IterateAllBalances(chain.GetContext(), func(address sdk.AccAddress, coin sdk.Coin) (stop bool) {
		fullDenom := coin.Denom
		if strings.HasPrefix(coin.Denom, "ibc/") {
			fullDenom, _ = chain.GetSimApp().TransferKeeper.DenomPathFromHash(chain.GetContext(), coin.Denom)
		}
		bank.SetBalance(address.String(), fullDenom, coin.Amount)
		return false
	})
	return bank
}

// Check that the state of the bank is the bankBefore + expectedBankChange
func (*KeeperTestSuite) CheckBankBalances(chain *ibctesting.TestChain, bankBefore *Bank, expectedBankChange *Bank) error {
	bankAfter := BankOfChain(chain)
	bankChange := bankAfter.Sub(bankBefore)
	diff := bankChange.Sub(expectedBankChange)
	nonZeroString := diff.NonZeroString()
	if len(nonZeroString) != 0 {
		return errorsmod.Wrap(ibcerrors.ErrInvalidCoins, "Unexpected changes in the bank: \n"+nonZeroString)
	}
	return nil
}

func (suite *KeeperTestSuite) TestModelBasedRelay() {
	dirname := "model_based_tests/"
	files, err := os.ReadDir(dirname)
	if err != nil {
		panic(fmt.Errorf("Failed to read model-based test files: %w", err))
	}
	for _, fileInfo := range files {
		tlaTestCases := []TlaOnRecvPacketTestCase{}
		if !strings.HasSuffix(fileInfo.Name(), ".json") {
			continue
		}
		jsonBlob, err := os.ReadFile(dirname + fileInfo.Name())
		if err != nil {
			panic(fmt.Errorf("Failed to read JSON test fixture: %w", err))
		}
		err = json.Unmarshal(jsonBlob, &tlaTestCases)
		if err != nil {
			panic(fmt.Errorf("Failed to parse JSON test fixture: %w", err))
		}

		suite.SetupTest()
		pathAtoB := ibctesting.NewTransferPath(suite.chainA, suite.chainB)
		pathBtoC := ibctesting.NewTransferPath(suite.chainB, suite.chainC)
		suite.coordinator.Setup(pathAtoB)
		suite.coordinator.Setup(pathBtoC)

		for i, tlaTc := range tlaTestCases {
			tc := OnRecvPacketTestCaseFromTla(tlaTc)
			registerDenomFn := func() {
				denomTrace := types.ParseDenomTrace(tc.packet.Data.Denom)
				traceHash := denomTrace.Hash()
				if !suite.chainB.GetSimApp().TransferKeeper.HasDenomTrace(suite.chainB.GetContext(), traceHash) {
					suite.chainB.GetSimApp().TransferKeeper.SetDenomTrace(suite.chainB.GetContext(), denomTrace)
				}
			}

			description := fileInfo.Name() + " # " + strconv.Itoa(i+1)
			suite.Run(fmt.Sprintf("Case %s", description), func() {
				seq := uint64(1)
				packet := channeltypes.NewPacket(tc.packet.Data.GetBytes(), seq, tc.packet.SourcePort, tc.packet.SourceChannel, tc.packet.DestPort, tc.packet.DestChannel, clienttypes.NewHeight(1, 100), 0)
				bankBefore := BankFromBalances(tc.bankBefore)
				realBankBefore := BankOfChain(suite.chainB)
				// First validate the packet itself (mimics what happens when the packet is being sent and/or received)
				err := packet.ValidateBasic()
				if err != nil {
					suite.Require().False(tc.pass, err.Error())
					return
				}
				switch tc.handler {
				case "SendTransfer":
					var sender sdk.AccAddress
					sender, err = sdk.AccAddressFromBech32(tc.packet.Data.Sender)
					if err != nil {
						panic(errors.New("MBT failed to convert sender address"))
					}
					registerDenomFn()
					denomTrace := types.ParseDenomTrace(tc.packet.Data.Denom)
					denom := denomTrace.IBCDenom()
					err = sdk.ValidateDenom(denom)
					if err == nil {
						amount, ok := sdkmath.NewIntFromString(tc.packet.Data.Amount)
						if !ok {
							panic(errors.New("MBT failed to parse amount from string"))
						}
						msg := types.NewMsgTransfer(
							tc.packet.SourcePort,
							tc.packet.SourceChannel,
							sdk.NewCoin(denom, amount),
							sender.String(),
							tc.packet.Data.Receiver,
							suite.chainA.GetTimeoutHeight(), 0, // only use timeout height
							"",
						)

						_, err = suite.chainB.GetSimApp().TransferKeeper.Transfer(suite.chainB.GetContext(), msg)

					}
				case "OnRecvPacket":
					err = suite.chainB.GetSimApp().TransferKeeper.OnRecvPacket(suite.chainB.GetContext(), packet, tc.packet.Data)
				case "OnTimeoutPacket":
					registerDenomFn()
					err = suite.chainB.GetSimApp().TransferKeeper.OnTimeoutPacket(suite.chainB.GetContext(), packet, tc.packet.Data)
				case "OnRecvAcknowledgementResult":
					err = suite.chainB.GetSimApp().TransferKeeper.OnAcknowledgementPacket(
						suite.chainB.GetContext(), packet, tc.packet.Data,
						channeltypes.NewResultAcknowledgement(nil))
				case "OnRecvAcknowledgementError":
					registerDenomFn()
					err = suite.chainB.GetSimApp().TransferKeeper.OnAcknowledgementPacket(
						suite.chainB.GetContext(), packet, tc.packet.Data,
						channeltypes.NewErrorAcknowledgement(fmt.Errorf("MBT Error Acknowledgement")))
				default:
					err = fmt.Errorf("Unknown handler:  %s", tc.handler)
				}
				if err != nil {
					suite.Require().False(tc.pass, err.Error())
					return
				}
				bankAfter := BankFromBalances(tc.bankAfter)
				expectedBankChange := bankAfter.Sub(&bankBefore)
				if err := suite.CheckBankBalances(suite.chainB, &realBankBefore, &expectedBankChange); err != nil {
					suite.Require().False(tc.pass, err.Error())
					return
				}
				suite.Require().True(tc.pass)
			})
		}
	}
}
