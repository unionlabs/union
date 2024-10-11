package cli

import (
	"bytes"
	"encoding/base64"
	"fmt"
	"io"
	"os"
	"strings"
	"testing"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"
	banktypes "cosmossdk.io/x/bank/types"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	strPlus   = ` string\s+`
	startStr  = `(?m:^\s+--`
	strDollar = `$)`
	strHelp   = "help output"
)

func TestParseSubmitLegacyProposal(t *testing.T) {
	okJSON := testutil.WriteToNewTempFile(t, `
{
  "title": "Test Proposal",
  "description": "My awesome proposal",
  "type": "Text",
  "deposit": "1000test"
}
`)

	badJSON := testutil.WriteToNewTempFile(t, "bad json")
	fs := NewCmdSubmitLegacyProposal().Flags()

	// nonexistent json
	err := fs.Set(FlagProposal, "fileDoesNotExist")
	require.NoError(t, err)

	_, err = parseSubmitLegacyProposal(fs)
	require.Error(t, err)

	// invalid json
	err = fs.Set(FlagProposal, badJSON.Name())
	require.NoError(t, err)
	_, err = parseSubmitLegacyProposal(fs)
	require.Error(t, err)

	// ok json
	err = fs.Set(FlagProposal, okJSON.Name())
	require.NoError(t, err)
	proposal1, err := parseSubmitLegacyProposal(fs)
	require.Nil(t, err, "unexpected error")
	require.Equal(t, "Test Proposal", proposal1.Title)
	require.Equal(t, "My awesome proposal", proposal1.Description)
	require.Equal(t, "Text", proposal1.Type)
	require.Equal(t, "1000test", proposal1.Deposit)

	// flags that can't be used with --proposal
	for _, incompatibleFlag := range ProposalFlags {
		err = fs.Set(incompatibleFlag, "some value")
		require.NoError(t, err)
		_, err := parseSubmitLegacyProposal(fs)
		require.Error(t, err)
		err = fs.Set(incompatibleFlag, "")
		require.NoError(t, err)
	}

	// no --proposal, only flags
	err = fs.Set(FlagProposal, "")
	require.NoError(t, err)
	flagTestCases := map[string]struct {
		pTitle       string
		pDescription string
		pType        string
		expErr       bool
		errMsg       string
	}{
		"valid flags": {
			pTitle:       proposal1.Title,
			pDescription: proposal1.Description,
			pType:        proposal1.Type,
		},
		"empty type": {
			pTitle:       proposal1.Title,
			pDescription: proposal1.Description,
			expErr:       true,
			errMsg:       "proposal type is required",
		},
		"empty title": {
			pDescription: proposal1.Description,
			pType:        proposal1.Type,
			expErr:       true,
			errMsg:       "proposal title is required",
		},
		"empty description": {
			pTitle: proposal1.Title,
			pType:  proposal1.Type,
			expErr: true,
			errMsg: "proposal description is required",
		},
	}
	for name, tc := range flagTestCases {
		t.Run(name, func(t *testing.T) {
			err = fs.Set(FlagTitle, tc.pTitle)
			require.NoError(t, err)
			err = fs.Set(FlagDescription, tc.pDescription)
			require.NoError(t, err)
			err = fs.Set(FlagProposalType, tc.pType)
			require.NoError(t, err)
			err = fs.Set(FlagDeposit, proposal1.Deposit)
			require.NoError(t, err)
			proposal2, err := parseSubmitLegacyProposal(fs)

			if tc.expErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.errMsg)
			} else {
				require.NoError(t, err)
				require.Equal(t, proposal1.Title, proposal2.Title)
				require.Equal(t, proposal1.Description, proposal2.Description)
				require.Equal(t, proposal1.Type, proposal2.Type)
				require.Equal(t, proposal1.Deposit, proposal2.Deposit)
			}
		})
	}

	err = okJSON.Close()
	require.Nil(t, err, "unexpected error")
	err = badJSON.Close()
	require.Nil(t, err, "unexpected error")
}

func TestParseSubmitProposal(t *testing.T) {
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(addr)
	require.NoError(t, err)

	interfaceRegistry := codectypes.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(interfaceRegistry)
	banktypes.RegisterInterfaces(interfaceRegistry)
	stakingtypes.RegisterInterfaces(interfaceRegistry)
	v1beta1.RegisterInterfaces(interfaceRegistry)
	v1.RegisterInterfaces(interfaceRegistry)
	expectedMetadata := []byte{42}

	okJSON := testutil.WriteToNewTempFile(t, fmt.Sprintf(`
{
	"messages": [
		{
			"@type": "/cosmos.bank.v1beta1.MsgSend",
			"from_address": "%s",
			"to_address": "%s",
			"amount":[{"denom": "stake","amount": "10"}]
		},
		{
			"@type": "/cosmos.staking.v1beta1.MsgDelegate",
			"delegator_address": "%s",
			"validator_address": "%s",
			"amount":{"denom": "stake","amount": "10"}
		},
		{
			"@type": "/cosmos.gov.v1.MsgExecLegacyContent",
			"authority": "%s",
			"content": {
				"@type": "/cosmos.gov.v1beta1.TextProposal",
				"title": "My awesome title",
				"description": "My awesome description"
			}
		}
  	],
	"metadata": "%s",
	"title": "My awesome title",
	"summary": "My awesome summary",
	"deposit": "1000test",
	"proposal_type": "expedited"
}
`, addr, addr, addr, addr, addr, base64.StdEncoding.EncodeToString(expectedMetadata)))

	badJSON := testutil.WriteToNewTempFile(t, "bad json")

	// nonexistent json
	_, _, _, err = parseSubmitProposal(cdc, "fileDoesNotExist")
	require.Error(t, err)

	// invalid json
	_, _, _, err = parseSubmitProposal(cdc, badJSON.Name())
	require.Error(t, err)

	// ok json
	proposal, msgs, deposit, err := parseSubmitProposal(cdc, okJSON.Name())
	require.NoError(t, err, "unexpected error")
	require.Equal(t, sdk.NewCoins(sdk.NewCoin("test", sdkmath.NewInt(1000))), deposit)
	require.Equal(t, base64.StdEncoding.EncodeToString(expectedMetadata), proposal.Metadata)
	require.Len(t, msgs, 3)
	msg1, ok := msgs[0].(*banktypes.MsgSend)
	require.True(t, ok)
	require.Equal(t, addrStr, msg1.FromAddress)
	require.Equal(t, addrStr, msg1.ToAddress)
	require.Equal(t, sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10))), msg1.Amount)
	msg2, ok := msgs[1].(*stakingtypes.MsgDelegate)
	require.True(t, ok)
	require.Equal(t, addrStr, msg2.DelegatorAddress)
	require.Equal(t, addrStr, msg2.ValidatorAddress)
	require.Equal(t, sdk.NewCoin("stake", sdkmath.NewInt(10)), msg2.Amount)
	msg3, ok := msgs[2].(*v1.MsgExecLegacyContent)
	require.True(t, ok)
	require.Equal(t, addrStr, msg3.Authority)
	textProp, ok := msg3.Content.GetCachedValue().(*v1beta1.TextProposal)
	require.True(t, ok)
	require.Equal(t, "My awesome title", textProp.Title)
	require.Equal(t, "My awesome description", textProp.Description)
	require.Equal(t, "My awesome title", proposal.Title)
	require.Equal(t, "My awesome summary", proposal.Summary)
	require.Equal(t, v1.ProposalType_PROPOSAL_TYPE_EXPEDITED, proposal.proposalType)

	err = okJSON.Close()
	require.Nil(t, err, "unexpected error")
	err = badJSON.Close()
	require.Nil(t, err, "unexpected error")
}

func getCommandHelp(t *testing.T, cmd *cobra.Command) string {
	t.Helper()
	// Create a pipe, so we can capture the help sent to stdout.
	reader, writer, err := os.Pipe()
	require.NoError(t, err, "creating os.Pipe()")
	outChan := make(chan string)
	defer func(origCmdOut io.Writer) {
		cmd.SetOut(origCmdOut)
		// Ignoring these errors since we're just ensuring cleanup here,
		// and they will return an error if already called (which we don't care about).
		_ = reader.Close()
		_ = writer.Close()
		close(outChan)
	}(cmd.OutOrStdout())
	cmd.SetOut(writer)

	// Do the reading in a separate goroutine from the writing (a best practice).
	go func() {
		var b bytes.Buffer
		_, buffErr := io.Copy(&b, reader)
		if buffErr != nil {
			// Due to complexities of goroutines and multiple channels, I'm sticking with a
			// single channel and just putting the error in there (which I'll test for later).
			b.WriteString("buffer error: " + buffErr.Error())
		}
		outChan <- b.String()
	}()

	err = cmd.Help()
	require.NoError(t, err, "cmd.Help()")
	require.NoError(t, writer.Close(), "pipe writer .Close()")
	rv := <-outChan
	require.NotContains(t, rv, "buffer error: ", "buffer output")
	return rv
}

func TestAddGovPropFlagsToCmd(t *testing.T) {
	cmd := &cobra.Command{
		Short: "Just a test command that does nothing but we can add flags to it.",
		Run: func(cmd *cobra.Command, args []string) {
			t.Errorf("The cmd has run with the args %q, but Run shouldn't have been called.", args)
		},
	}
	testFunc := func() {
		AddGovPropFlagsToCmd(cmd)
	}
	require.NotPanics(t, testFunc, "AddGovPropFlagsToCmd")

	help := getCommandHelp(t, cmd)

	expDepositDesc := "The deposit to include with the governance proposal"
	expMetadataDesc := "The metadata to include with the governance proposal"
	expTitleDesc := "The title to put on the governance proposal"
	expSummaryDesc := "The summary to include with the governance proposal"
	// Regexp notes: (?m:...) = multi-line mode so ^ and $ match the beginning and end of each line.
	// Each regexp assertion checks for a line containing only a specific flag and its description.
	assert.Regexp(t, startStr+FlagDeposit+strPlus+expDepositDesc+strDollar, help, strHelp)
	assert.Regexp(t, startStr+FlagMetadata+strPlus+expMetadataDesc+strDollar, help, strHelp)
	assert.Regexp(t, startStr+FlagTitle+strPlus+expTitleDesc+strDollar, help, strHelp)
	assert.Regexp(t, startStr+FlagSummary+strPlus+expSummaryDesc+strDollar, help, strHelp)
}

func TestReadGovPropFlags(t *testing.T) {
	fromAddr := sdk.AccAddress("from_addr___________")
	argDeposit := "--" + FlagDeposit
	argMetadata := "--" + FlagMetadata
	argTitle := "--" + FlagTitle
	argSummary := "--" + FlagSummary

	fromAddrStr, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(fromAddr)
	require.NoError(t, err)
	// cz is a shorter way to define coins objects for these tests.
	cz := func(coins string) sdk.Coins {
		rv, err := sdk.ParseCoinsNormalized(coins)
		require.NoError(t, err, "ParseCoinsNormalized(%q)", coins)
		return rv
	}

	tests := []struct {
		name     string
		fromAddr sdk.AccAddress
		args     []string
		exp      *v1.MsgSubmitProposal
		expErr   []string
	}{
		{
			name:     "no args no from",
			fromAddr: nil,
			args:     []string{},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only from defined",
			fromAddr: fromAddr,
			args:     []string{},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       fromAddrStr,
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},

		// only deposit tests.
		{
			name:     "only deposit empty string",
			fromAddr: nil,
			args:     []string{argDeposit, ""},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only deposit one coin",
			fromAddr: nil,
			args:     []string{argDeposit, "1bigcoin"},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("1bigcoin"),
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only deposit invalid coins",
			fromAddr: nil,
			args:     []string{argDeposit, "not really coins"},
			expErr:   []string{"invalid deposit", "invalid character in denomination"},
		},
		{
			name:     "only deposit two coins",
			fromAddr: nil,
			args:     []string{argDeposit, "1acoin,2bcoin"},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("1acoin,2bcoin"),
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only deposit two coins other order",
			fromAddr: nil,
			args:     []string{argDeposit, "2bcoin,1acoin"},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("1acoin,2bcoin"),
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only deposit coin 1 of 3 bad",
			fromAddr: nil,
			args:     []string{argDeposit, "1bad^coin,2bcoin,3ccoin"},
			expErr:   []string{"invalid deposit", "invalid character in denomination"},
		},
		{
			name:     "only deposit coin 2 of 3 bad",
			fromAddr: nil,
			args:     []string{argDeposit, "1acoin,2bad^coin,3ccoin"},
			expErr:   []string{"invalid deposit", "invalid character in denomination"},
		},
		{
			name:     "only deposit coin 3 of 3 bad",
			fromAddr: nil,
			args:     []string{argDeposit, "1acoin,2bcoin,3bad^coin"},
			expErr:   []string{"invalid deposit", "invalid character in denomination"},
		},
		// As far as I can tell, there's no way to make flagSet.GetString return an error for a defined string flag.
		// So I don't have a test for the "could not read deposit" error case.

		// only metadata tests.
		{
			name:     "only metadata empty",
			fromAddr: nil,
			args:     []string{argMetadata, ""},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only metadata simple",
			fromAddr: nil,
			args:     []string{argMetadata, "just some metadata"},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "just some metadata",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only metadata super long",
			fromAddr: nil,
			args:     []string{argMetadata, strings.Repeat("Long", 1_000_000)},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       strings.Repeat("Long", 1_000_000),
				Title:          "",
				Summary:        "",
			},
		},
		// As far as I can tell, there's no way to make flagSet.GetString return an error for a defined string flag.
		// So I don't have a test for the "could not read metadata" error case.

		// only title tests.
		{
			name:     "only title empty",
			fromAddr: nil,
			args:     []string{argTitle, ""},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only title simple",
			fromAddr: nil,
			args:     []string{argTitle, "just a title"},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "just a title",
				Summary:        "",
			},
		},
		{
			name:     "only title super long",
			fromAddr: nil,
			args:     []string{argTitle, strings.Repeat("Long", 1_000_000)},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          strings.Repeat("Long", 1_000_000),
				Summary:        "",
			},
		},
		// As far as I can tell, there's no way to make flagSet.GetString return an error for a defined string flag.
		// So I don't have a test for the "could not read title" error case.

		// only summary tests.
		{
			name:     "only summary empty",
			fromAddr: nil,
			args:     []string{argSummary, ""},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "only summary simple",
			fromAddr: nil,
			args:     []string{argSummary, "just a short summary"},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "just a short summary",
			},
		},
		{
			name:     "only summary super long",
			fromAddr: nil,
			args:     []string{argSummary, strings.Repeat("Long", 1_000_000)},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        strings.Repeat("Long", 1_000_000),
			},
		},
		// As far as I can tell, there's no way to make flagSet.GetString return an error for a defined string flag.
		// So I don't have a test for the "could not read summary" error case.

		// Combo tests.
		{
			name:     "all together order 1",
			fromAddr: fromAddr,
			args: []string{
				argDeposit, "56depcoin",
				argMetadata, "my proposal is cool",
				argTitle, "Simple Gov Prop Title",
				argSummary, "This is just a test summary on a simple gov prop.",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("56depcoin"),
				Proposer:       fromAddrStr,
				Metadata:       "my proposal is cool",
				Title:          "Simple Gov Prop Title",
				Summary:        "This is just a test summary on a simple gov prop.",
			},
		},
		{
			name:     "all together order 2",
			fromAddr: fromAddr,
			args: []string{
				argTitle, "This title is a *bit* more complex.",
				argSummary, "This\nis\na\ncrazy\nsummary",
				argDeposit, "78coolcoin",
				argMetadata, "this proposal is cooler",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("78coolcoin"),
				Proposer:       fromAddrStr,
				Metadata:       "this proposal is cooler",
				Title:          "This title is a *bit* more complex.",
				Summary:        "This\nis\na\ncrazy\nsummary",
			},
		},
		{
			name:     "all except proposer",
			fromAddr: nil,
			args: []string{
				argMetadata, "https://example.com/lucky",
				argDeposit, "33luckycoin",
				argSummary, "This proposal will bring you luck and good fortune in the new year.",
				argTitle, "Increase Luck",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("33luckycoin"),
				Proposer:       "",
				Metadata:       "https://example.com/lucky",
				Title:          "Increase Luck",
				Summary:        "This proposal will bring you luck and good fortune in the new year.",
			},
		},
		{
			name:     "all except proposer but all empty",
			fromAddr: nil,
			args: []string{
				argMetadata, "",
				argDeposit, "",
				argSummary, "",
				argTitle, "",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       "",
				Metadata:       "",
				Title:          "",
				Summary:        "",
			},
		},
		{
			name:     "all except deposit",
			fromAddr: fromAddr,
			args: []string{
				argTitle, "This is a Title",
				argSummary, "This is a useless summary",
				argMetadata, "worthless metadata",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: nil,
				Proposer:       fromAddrStr,
				Metadata:       "worthless metadata",
				Title:          "This is a Title",
				Summary:        "This is a useless summary",
			},
			expErr: nil,
		},
		{
			name:     "all except metadata",
			fromAddr: fromAddr,
			args: []string{
				argTitle, "Bland Title",
				argSummary, "Boring summary",
				argDeposit, "99mdcoin",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("99mdcoin"),
				Proposer:       fromAddrStr,
				Metadata:       "",
				Title:          "Bland Title",
				Summary:        "Boring summary",
			},
		},
		{
			name:     "all except title",
			fromAddr: fromAddr,
			args: []string{
				argMetadata, "this metadata does not have the title either",
				argDeposit, "71whatcoin",
				argSummary, "This is a summary on a titleless proposal.",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("71whatcoin"),
				Proposer:       fromAddrStr,
				Metadata:       "this metadata does not have the title either",
				Title:          "",
				Summary:        "This is a summary on a titleless proposal.",
			},
			expErr: nil,
		},
		{
			name:     "all except summary",
			fromAddr: fromAddr,
			args: []string{
				argMetadata, "28",
				argTitle, "Now This is What I Call A Governance Proposal 28",
				argDeposit, "42musiccoin",
			},
			exp: &v1.MsgSubmitProposal{
				InitialDeposit: cz("42musiccoin"),
				Proposer:       fromAddrStr,
				Metadata:       "28",
				Title:          "Now This is What I Call A Governance Proposal 28",
				Summary:        "",
			},
			expErr: nil,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			cmd := &cobra.Command{
				Short: tc.name,
				Run: func(cmd *cobra.Command, args []string) {
					t.Errorf("The cmd for %q has run with the args %q, but Run shouldn't have been called.", tc.name, args)
				},
			}
			AddGovPropFlagsToCmd(cmd)
			err := cmd.ParseFlags(tc.args)
			require.NoError(t, err, "parsing test case args using cmd: %q", tc.args)
			flagSet := cmd.Flags()

			clientCtx := client.Context{
				FromAddress:  tc.fromAddr,
				AddressCodec: codectestutil.CodecOptions{}.GetAddressCodec(),
			}

			var msg *v1.MsgSubmitProposal
			testFunc := func() {
				msg, err = ReadGovPropFlags(clientCtx, flagSet)
			}
			require.NotPanics(t, testFunc, "ReadGovPropFlags")
			if len(tc.expErr) > 0 {
				require.Error(t, err, "ReadGovPropFlags error")
				for _, exp := range tc.expErr {
					assert.ErrorContains(t, err, exp, "ReadGovPropFlags error")
				}
			} else {
				require.NoError(t, err, "ReadGovPropFlags error")
			}
			assert.Equal(t, tc.exp, msg, "ReadGovPropFlags msg")
		})
	}
}
