package autocli

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/spf13/cobra"
	"gotest.tools/v3/assert"
	"gotest.tools/v3/golden"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	"cosmossdk.io/client/v2/internal/testpb"

	"github.com/cosmos/cosmos-sdk/client"
)

var buildModuleMsgCommand = func(moduleName string, f *fixture) (*cobra.Command, error) {
	ctx := context.WithValue(context.Background(), client.ClientContextKey, &f.clientCtx)
	cmd := topLevelCmd(ctx, moduleName, fmt.Sprintf("Transactions commands for the %s module", moduleName))
	err := f.b.AddMsgServiceCommands(cmd, bankAutoCLI)
	return cmd, err
}

func buildCustomModuleMsgCommand(cmdDescriptor *autocliv1.ServiceCommandDescriptor) func(moduleName string, f *fixture) (*cobra.Command, error) {
	return func(moduleName string, f *fixture) (*cobra.Command, error) {
		ctx := context.WithValue(context.Background(), client.ClientContextKey, &f.clientCtx)
		cmd := topLevelCmd(ctx, moduleName, fmt.Sprintf("Transactions commands for the %s module", moduleName))
		err := f.b.AddMsgServiceCommands(cmd, cmdDescriptor)
		return cmd, err
	}
}

var bankAutoCLI = &autocliv1.ServiceCommandDescriptor{
	Service: bankv1beta1.Msg_ServiceDesc.ServiceName,
	RpcCommandOptions: []*autocliv1.RpcCommandOptions{
		{
			RpcMethod:      "Send",
			Use:            "send <from_key_or_address> <to_address> <amount> [flags]",
			Short:          "Send coins from one account to another",
			PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "from_address"}, {ProtoField: "to_address"}, {ProtoField: "amount"}},
		},
	},
	EnhanceCustomCommand: true,
}

func TestMsg(t *testing.T) {
	fixture := initFixture(t)
	out, err := runCmd(fixture, buildModuleMsgCommand, "send",
		"cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk", "cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk", "1foo",
		"--generate-only",
		"--output", "json",
	)
	assert.NilError(t, err)
	assertNormalizedJSONEqual(t, out.Bytes(), goldenLoad(t, "msg-output.golden"))

	out, err = runCmd(fixture, buildCustomModuleMsgCommand(&autocliv1.ServiceCommandDescriptor{
		Service: bankv1beta1.Msg_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod:      "Send",
				Use:            "send <from_key_or_address> <to_address> <amount> [flags]",
				Short:          "Send coins from one account to another",
				PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "from_address"}, {ProtoField: "to_address"}, {ProtoField: "amount"}},
			},
		},
		EnhanceCustomCommand: true,
	}), "send",
		"cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk", "cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk", "1foo",
		"--generate-only",
		"--output", "json",
	)
	assert.NilError(t, err)
	assertNormalizedJSONEqual(t, out.Bytes(), goldenLoad(t, "msg-output.golden"))

	out, err = runCmd(fixture, buildCustomModuleMsgCommand(&autocliv1.ServiceCommandDescriptor{
		Service: bankv1beta1.Msg_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod:      "Send",
				Use:            "send <from_key_or_address> <to_address> <amount> [flags]",
				Short:          "Send coins from one account to another",
				PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "to_address"}, {ProtoField: "amount"}},
				// from_address should be automatically added
			},
		},
		EnhanceCustomCommand: true,
	}), "send",
		"cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk", "1foo",
		"--from", "cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk",
		"--generate-only",
		"--output", "json",
	)
	assert.NilError(t, err)
	assertNormalizedJSONEqual(t, out.Bytes(), goldenLoad(t, "msg-output.golden"))

	out, err = runCmd(fixture, buildCustomModuleMsgCommand(&autocliv1.ServiceCommandDescriptor{
		Service: bankv1beta1.Msg_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod:      "Send",
				Use:            "send <from_key_or_address> <to_address> <amount> [flags]",
				Short:          "Send coins from one account to another",
				PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "to_address"}, {ProtoField: "amount"}},
				FlagOptions: map[string]*autocliv1.FlagOptions{
					"from_address": {Name: "sender"}, // use a custom flag for signer
				},
			},
		},
		EnhanceCustomCommand: true,
	}), "send",
		"cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk", "1foo",
		"--sender", "cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk",
		"--generate-only",
		"--output", "json",
	)
	assert.NilError(t, err)
	assertNormalizedJSONEqual(t, out.Bytes(), goldenLoad(t, "msg-output.golden"))
}

func goldenLoad(t *testing.T, filename string) []byte {
	t.Helper()
	content, err := os.ReadFile(filepath.Join("testdata", filename))
	assert.NilError(t, err)
	return content
}

func assertNormalizedJSONEqual(t *testing.T, expected, actual []byte) {
	t.Helper()
	normalizedExpected, err := normalizeJSON(expected)
	assert.NilError(t, err)
	normalizedActual, err := normalizeJSON(actual)
	assert.NilError(t, err)
	assert.Equal(t, string(normalizedExpected), string(normalizedActual))
}

// normalizeJSON normalizes the JSON content by removing unnecessary white spaces and newlines.
func normalizeJSON(content []byte) ([]byte, error) {
	var buf bytes.Buffer
	err := json.Compact(&buf, content)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

func TestMsgOptionsError(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleMsgCommand,
		"send", "5",
	)
	assert.ErrorContains(t, err, "accepts 3 arg(s)")

	_, err = runCmd(fixture, buildModuleMsgCommand,
		"send", "foo", "bar", "invalid",
	)
	assert.ErrorContains(t, err, "invalid argument")
}

func TestHelpMsg(t *testing.T) {
	fixture := initFixture(t)

	out, err := runCmd(fixture, buildModuleMsgCommand, "-h")
	assert.NilError(t, err)
	golden.Assert(t, out.String(), "help-toplevel-msg.golden")

	out, err = runCmd(fixture, buildModuleMsgCommand, "send", "-h")
	assert.NilError(t, err)
	golden.Assert(t, out.String(), "help-echo-msg.golden")
}

func TestBuildCustomMsgCommand(t *testing.T) {
	b := &Builder{}
	customCommandCalled := false
	appOptions := AppOptions{
		ModuleOptions: map[string]*autocliv1.ModuleOptions{
			"test": {
				Tx: &autocliv1.ServiceCommandDescriptor{
					Service:           testpb.Msg_ServiceDesc.ServiceName,
					RpcCommandOptions: []*autocliv1.RpcCommandOptions{},
				},
			},
		},
	}

	cmd, err := b.BuildMsgCommand(context.Background(), appOptions, map[string]*cobra.Command{
		"test": {Use: "test", Run: func(cmd *cobra.Command, args []string) {
			customCommandCalled = true
		}},
	})
	assert.NilError(t, err)
	cmd.SetArgs([]string{"test", "tx"})
	assert.NilError(t, cmd.Execute())
	assert.Assert(t, customCommandCalled)
}

func TestNotFoundErrorsMsg(t *testing.T) {
	fixture := initFixture(t)
	b := fixture.b
	b.AddQueryConnFlags = nil
	b.AddTxConnFlags = nil

	buildModuleMsgCommand := func(moduleName string, cmdDescriptor *autocliv1.ServiceCommandDescriptor) (*cobra.Command, error) {
		cmd := topLevelCmd(context.Background(), moduleName, fmt.Sprintf("Transactions commands for the %s module", moduleName))

		err := b.AddMsgServiceCommands(cmd, cmdDescriptor)
		return cmd, err
	}

	// Query non existent service
	_, err := buildModuleMsgCommand("test", &autocliv1.ServiceCommandDescriptor{Service: "un-existent-service"})
	assert.ErrorContains(t, err, "can't find service un-existent-service")

	_, err = buildModuleMsgCommand("test", &autocliv1.ServiceCommandDescriptor{
		Service:           testpb.Query_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{{RpcMethod: "un-existent-method"}},
	})
	assert.ErrorContains(t, err, "rpc method \"un-existent-method\" not found")

	_, err = buildModuleMsgCommand("test", &autocliv1.ServiceCommandDescriptor{
		Service: testpb.Msg_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod: "Send",
				PositionalArgs: []*autocliv1.PositionalArgDescriptor{
					{
						ProtoField: "un-existent-proto-field",
					},
				},
			},
		},
	})
	assert.ErrorContains(t, err, "can't find field un-existent-proto-field")

	_, err = buildModuleMsgCommand("test", &autocliv1.ServiceCommandDescriptor{
		Service: testpb.Msg_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod: "Send",
				FlagOptions: map[string]*autocliv1.FlagOptions{
					"un-existent-flag": {},
				},
			},
		},
	})
	assert.ErrorContains(t, err, "can't find field un-existent-flag")
}
