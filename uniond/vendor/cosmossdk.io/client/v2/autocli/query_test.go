package autocli

import (
	"context"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/spf13/cobra"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/timestamppb"
	"gotest.tools/v3/assert"
	"gotest.tools/v3/golden"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	queryv1beta1 "cosmossdk.io/api/cosmos/base/query/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/client/v2/internal/testpb"

	"github.com/cosmos/cosmos-sdk/client"
)

var buildModuleQueryCommand = func(moduleName string, f *fixture) (*cobra.Command, error) {
	ctx := context.WithValue(context.Background(), client.ClientContextKey, &f.clientCtx)
	cmd := topLevelCmd(ctx, moduleName, fmt.Sprintf("Querying commands for the %s module", moduleName))

	err := f.b.AddQueryServiceCommands(cmd, testCmdDesc)
	return cmd, err
}

var buildModuleQueryCommandOptional = func(moduleName string, f *fixture) (*cobra.Command, error) {
	ctx := context.WithValue(context.Background(), client.ClientContextKey, &f.clientCtx)
	cmd := topLevelCmd(ctx, moduleName, fmt.Sprintf("Querying commands for the %s module", moduleName))

	err := f.b.AddQueryServiceCommands(cmd, testCmdDescOptional)
	return cmd, err
}

var buildModuleVargasOptional = func(moduleName string, f *fixture) (*cobra.Command, error) {
	ctx := context.WithValue(context.Background(), client.ClientContextKey, &f.clientCtx)
	cmd := topLevelCmd(ctx, moduleName, fmt.Sprintf("Querying commands for the %s module", moduleName))

	err := f.b.AddQueryServiceCommands(cmd, testCmdDescInvalidOptAndVargas)
	return cmd, err
}

var testCmdDesc = &autocliv1.ServiceCommandDescriptor{
	Service: testpb.Query_ServiceDesc.ServiceName,
	RpcCommandOptions: []*autocliv1.RpcCommandOptions{
		{
			RpcMethod:  "Echo",
			Use:        "echo [pos1] [pos2] [pos3...]",
			Version:    "1.0",
			Alias:      []string{"e"},
			SuggestFor: []string{"eco"},
			Example:    "echo 1 abc {}",
			Short:      "echo echos the value provided by the user",
			Long:       "echo echos the value provided by the user as a proto JSON object with populated with the provided fields and positional arguments",
			PositionalArgs: []*autocliv1.PositionalArgDescriptor{
				{
					ProtoField: "positional1",
				},
				{
					ProtoField: "positional2",
				},
				{
					ProtoField: "positional3_varargs",
					Varargs:    true,
				},
			},
			FlagOptions: map[string]*autocliv1.FlagOptions{
				"u32": {
					Name:      "uint32",
					Shorthand: "u",
					Usage:     "some random uint32",
				},
				"i32": {
					Usage:        "some random int32",
					DefaultValue: "3",
				},
				"u64": {
					Usage:        "some random uint64",
					DefaultValue: "5",
				},
				"deprecated_field": {
					Deprecated: "don't use this",
				},
				"shorthand_deprecated_field": {
					Shorthand:  "s",
					Deprecated: "bad idea",
				},
				"hidden_bool": {
					Hidden: true,
				},
				"a_coin": {
					Usage: "some random coin",
				},
				"duration": {
					Usage: "some random duration",
				},
				"bz": {
					Usage: "some bytes",
				},
				"map_string_string": {
					Usage: "some map of string to string",
				},
				"map_string_uint32": {
					Usage: "some map of string to int32",
				},
				"map_string_coin": {
					Usage: "some map of string to coin",
				},
			},
		},
	},
	SubCommands: map[string]*autocliv1.ServiceCommandDescriptor{
		// we test the sub-command functionality using the same service with different options
		"deprecatedecho": {
			Service: testpb.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod:  "Echo",
					Deprecated: "don't use this",
				},
			},
		},
		"skipecho": {
			Service: testpb.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Echo",
					Skip:      true,
				},
			},
		},
	},
}

var testCmdDescOptional = &autocliv1.ServiceCommandDescriptor{
	Service: testpb.Query_ServiceDesc.ServiceName,
	RpcCommandOptions: []*autocliv1.RpcCommandOptions{
		{
			RpcMethod:  "Echo",
			Use:        "echo [pos1] [pos2] [pos3...]",
			Version:    "1.0",
			Alias:      []string{"e"},
			SuggestFor: []string{"eco"},
			Example:    "echo 1 abc {}",
			Short:      "echo echos the value provided by the user",
			Long:       "echo echos the value provided by the user as a proto JSON object with populated with the provided fields and positional arguments",
			PositionalArgs: []*autocliv1.PositionalArgDescriptor{
				{
					ProtoField: "positional1",
				},
				{
					ProtoField: "positional2",
					Optional:   true,
				},
			},
		},
	},
}

var testCmdDescInvalidOptAndVargas = &autocliv1.ServiceCommandDescriptor{
	Service: testpb.Query_ServiceDesc.ServiceName,
	RpcCommandOptions: []*autocliv1.RpcCommandOptions{
		{
			RpcMethod:  "Echo",
			Use:        "echo [pos1] [pos2] [pos3...]",
			Version:    "1.0",
			Alias:      []string{"e"},
			SuggestFor: []string{"eco"},
			Example:    "echo 1 abc {}",
			Short:      "echo echos the value provided by the user",
			Long:       "echo echos the value provided by the user as a proto JSON object with populated with the provided fields and positional arguments",
			PositionalArgs: []*autocliv1.PositionalArgDescriptor{
				{
					ProtoField: "positional1",
				},
				{
					ProtoField: "positional2",
					Optional:   true,
				},
				{
					ProtoField: "positional3_varargs",
					Varargs:    true,
				},
			},
		},
	},
}

func TestCoin(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1",
		"abc",
		"1234foo,4321bar",
		"100uatom",
		"--a-coin", "100000foo",
	)
	assert.ErrorContains(t, err, "coin flag must be a single coin, specific multiple coins with multiple flags or spaces")

	_, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1",
		"abc",
		"1234foo",
		"4321bar",
		"100uatom",
		"--a-coin", "100000foo",
		"--coins", "100000bar",
		"--coins", "100uatom",
	)
	assert.NilError(t, err)
	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())
	expectedResp := &testpb.EchoResponse{
		Request: &testpb.EchoRequest{
			Positional1: 1,
			Positional2: "abc",
			Positional3Varargs: []*basev1beta1.Coin{
				{Amount: "1234", Denom: "foo"},
				{Amount: "4321", Denom: "bar"},
				{Amount: "100", Denom: "uatom"},
			},
			ACoin: &basev1beta1.Coin{
				Amount: "100000",
				Denom:  "foo",
			},
			Coins: []*basev1beta1.Coin{
				{Amount: "100000", Denom: "bar"},
				{Amount: "100", Denom: "uatom"},
			},
			Page: &queryv1beta1.PageRequest{},
			I32:  3,
			U64:  5,
		},
	}
	assert.DeepEqual(t, fixture.conn.lastResponse.(*testpb.EchoResponse), expectedResp, protocmp.Transform())
}

func TestOptional(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommandOptional,
		"echo",
		"1",
		"abc",
	)
	assert.NilError(t, err)
	request := fixture.conn.lastRequest.(*testpb.EchoRequest)
	assert.Equal(t, request.Positional2, "abc")
	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())

	_, err = runCmd(fixture, buildModuleQueryCommandOptional,
		"echo",
		"1",
	)
	assert.NilError(t, err)

	request = fixture.conn.lastRequest.(*testpb.EchoRequest)
	assert.Equal(t, request.Positional2, "")
	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())

	_, err = runCmd(fixture, buildModuleQueryCommandOptional,
		"echo",
		"1",
		"abc",
		"extra-arg",
	)
	assert.ErrorContains(t, err, "accepts between 1 and 2 arg(s), received 3")

	_, err = runCmd(fixture, buildModuleVargasOptional,
		"echo",
		"1",
		"abc",
		"extra-arg",
	)
	assert.ErrorContains(t, err, "optional positional argument positional2 must be the last argument")
}

func TestMap(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1",
		"abc",
		"1234foo",
		"4321bar",
		"--map-string-uint32", "bar=123",
		"--map-string-string", "val=foo",
		"--map-string-coin", "baz=100000foo",
		"--map-string-coin", "sec=100000bar",
		"--map-string-coin", "multi=100000bar,flag=100000foo",
	)
	assert.NilError(t, err)
	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())

	_, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1",
		"abc",
		"1234foo",
		"4321bar",
		"--map-string-uint32", "bar=123",
		"--map-string-coin", "baz,100000foo",
		"--map-string-coin", "sec=100000bar",
	)
	assert.ErrorContains(t, err, "invalid argument \"baz,100000foo\" for \"--map-string-coin\" flag: invalid format, expected key=value")

	_, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1",
		"abc",
		"1234foo",
		"4321bar",
		"--map-string-uint32", "bar=not-unint32",
		"--map-string-coin", "baz=100000foo",
		"--map-string-coin", "sec=100000bar",
	)
	assert.ErrorContains(t, err, "invalid argument \"bar=not-unint32\" for \"--map-string-uint32\" flag: strconv.ParseUint: parsing \"not-unint32\": invalid syntax")

	_, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1",
		"abc",
		"1234foo",
		"4321bar",
		"--map-string-uint32", "bar=123.9",
		"--map-string-coin", "baz=100000foo",
		"--map-string-coin", "sec=100000bar",
	)
	assert.ErrorContains(t, err, "invalid argument \"bar=123.9\" for \"--map-string-uint32\" flag: strconv.ParseUint: parsing \"123.9\": invalid syntax")
}

// TestEverything tests all the different types of flags are correctly read and as well as correctly returned
// This tests the flag binding and the message building
func TestEverything(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1",
		"abc",
		"123.123123124foo",
		"4321bar",
		"--a-bool",
		"--an-enum", "one",
		"--a-message", `{"bar":"abc", "baz":-3}`,
		"--duration", "4h3s",
		"--uint32", "27",
		"--u64", "3267246890",
		"--i32", "-253",
		"--i64", "-234602347",
		"--str", "def",
		"--timestamp", "2019-01-02T00:01:02Z",
		"--a-coin", "100000foo",
		"--an-address", "cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk",
		"--a-validator-address", "cosmosvaloper1tnh2q55v8wyygtt9srz5safamzdengsn9dsd7z",
		"--a-consensus-address", "cosmosvalcons16vm0nx49eam4q0xasdnwdzsdl6ymgyjt757sgr",
		"--bz", "c2RncXdlZndkZ3NkZw==",
		"--page-count-total",
		"--page-key", "MTIzNTQ4N3NnaGRhcw==",
		"--page-limit", "1000",
		"--page-offset", "10",
		"--page-reverse",
		"--bools", "true",
		"--bools", "false,false,true",
		"--enums", "one",
		"--enums", "five",
		"--enums", "two",
		"--strings", "abc",
		"--strings", "xyz",
		"--strings", "xyz,qrs",
		"--durations", "3s",
		"--durations", "5s",
		"--durations", "10h",
		"--some-messages", "{}",
		"--some-messages", `{"bar":"baz"}`,
		"--some-messages", `{"baz":-1}`,
		"--uints", "1,2,3",
		"--uints", "4",
	)
	assert.NilError(t, err)

	expectedResp := &testpb.EchoResponse{
		Request: &testpb.EchoRequest{
			Positional1: 1,
			Positional2: "abc",
			Positional3Varargs: []*basev1beta1.Coin{
				{Amount: "123.123123124", Denom: "foo"},
				{Amount: "4321", Denom: "bar"},
			},
			ABool:  true,
			AnEnum: testpb.Enum_ENUM_ONE,
			AMessage: &testpb.AMessage{
				Bar: "abc",
				Baz: -3,
			},
			Duration: durationpb.New(4*time.Hour + 3*time.Second),
			U32:      27,
			U64:      3267246890,
			I32:      -253,
			I64:      -234602347,
			Str:      "def",
			Timestamp: &timestamppb.Timestamp{
				Seconds: 1546387262,
			},
			ACoin: &basev1beta1.Coin{
				Amount: "100000",
				Denom:  "foo",
			},
			AnAddress:         "cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk",
			AValidatorAddress: "cosmosvaloper1tnh2q55v8wyygtt9srz5safamzdengsn9dsd7z",
			AConsensusAddress: "cosmosvalcons16vm0nx49eam4q0xasdnwdzsdl6ymgyjt757sgr",
			Bz:                []byte("sdgqwefwdgsdg"),
			Page: &queryv1beta1.PageRequest{
				CountTotal: true,
				Key:        []byte("1235487sghdas"),
				Limit:      1000,
				Offset:     10,
				Reverse:    true,
			},
			Bools: []bool{true, false, false, true},
			Enums: []testpb.Enum{testpb.Enum_ENUM_ONE, testpb.Enum_ENUM_FIVE, testpb.Enum_ENUM_TWO},
			Strings: []string{
				"abc",
				"xyz",
				"xyz",
				"qrs",
			},
			Durations: []*durationpb.Duration{
				durationpb.New(3 * time.Second),
				durationpb.New(5 * time.Second),
				durationpb.New(10 * time.Hour),
			},
			SomeMessages: []*testpb.AMessage{
				{},
				{Bar: "baz"},
				{Baz: -1},
			},
			Uints: []uint32{1, 2, 3, 4},
		},
	}

	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())
	assert.DeepEqual(t, fixture.conn.lastResponse.(*testpb.EchoResponse), expectedResp, protocmp.Transform())
}

func TestPubKeyParsingConsensusAddress(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--a-consensus-address", "{\"@type\":\"/cosmos.crypto.ed25519.PubKey\",\"key\":\"j8qdbR+AlH/V6aBTCSWXRvX3JUESF2bV+SEzndBhF0o=\"}",
		"-u", "27", // shorthand
	)
	assert.NilError(t, err)
	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())
}

func TestJSONParsing(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--some-messages", `{"bar":"baz"}`,
		"-u", "27", // shorthand
	)
	assert.NilError(t, err)
	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())

	_, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--some-messages", "testdata/some_message.json",
		"-u", "27", // shorthand
	)
	assert.NilError(t, err)
	assert.DeepEqual(t, fixture.conn.lastRequest, fixture.conn.lastResponse.(*testpb.EchoResponse).Request, protocmp.Transform())
}

func TestOptions(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "123foo",
		"-u", "27", // shorthand
		"--u64", "5", // no opt default value
	)
	assert.NilError(t, err)

	lastReq := fixture.conn.lastRequest.(*testpb.EchoRequest)
	assert.Equal(t, uint32(27), lastReq.U32) // shorthand got set
	assert.Equal(t, int32(3), lastReq.I32)   // default value got set
	assert.Equal(t, uint64(5), lastReq.U64)  // no opt default value got set
}

func TestBinaryFlag(t *testing.T) {
	// Create a temporary file with some content
	tempFile, err := os.Open("testdata/file.test")
	if err != nil {
		t.Fatal(err)
	}
	content := []byte("this is just a test file")
	if err := tempFile.Close(); err != nil {
		t.Fatal(err)
	}

	// Test cases
	tests := []struct {
		name     string
		input    string
		expected []byte
		hasError bool
		err      string
	}{
		{
			name:     "Valid file path with extension",
			input:    tempFile.Name(),
			expected: content,
			hasError: false,
			err:      "",
		},
		{
			name:     "Valid hex-encoded string",
			input:    "68656c6c6f20776f726c64",
			expected: []byte("hello world"),
			hasError: false,
			err:      "",
		},
		{
			name:     "Valid base64-encoded string",
			input:    "SGVsbG8gV29ybGQ=",
			expected: []byte("Hello World"),
			hasError: false,
			err:      "",
		},
		{
			name:     "Invalid input (not a file path or encoded string)",
			input:    "not a file or encoded string",
			expected: nil,
			hasError: true,
			err:      "input string is neither a valid file path, hex, or base64 encoded",
		},
	}

	// Run test cases
	fixture := initFixture(t)
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			_, err := runCmd(fixture, buildModuleQueryCommand,
				"echo",
				"1", "abc", `100foo`,
				"--bz", tc.input,
			)
			if tc.hasError {
				assert.ErrorContains(t, err, tc.err)
			} else {
				assert.NilError(t, err)
				lastReq := fixture.conn.lastRequest.(*testpb.EchoRequest)
				assert.DeepEqual(t, tc.expected, lastReq.Bz)
			}
		})
	}
}

func TestAddressValidation(t *testing.T) {
	fixture := initFixture(t)

	_, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--an-address", "cosmos1y74p8wyy4enfhfn342njve6cjmj5c8dtl6emdk",
	)
	assert.NilError(t, err)

	_, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--an-address", "regen1y74p8wyy4enfhfn342njve6cjmj5c8dtlqj7ule2",
	)
	assert.ErrorContains(t, err, "invalid account address")

	_, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--an-address", "cosmps1BAD_ENCODING",
	)
	assert.ErrorContains(t, err, "invalid account address")
}

func TestOutputFormat(t *testing.T) {
	fixture := initFixture(t)

	out, err := runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--output", "json",
	)
	assert.NilError(t, err)
	assert.Assert(t, strings.Contains(out.String(), "{"))

	out, err = runCmd(fixture, buildModuleQueryCommand,
		"echo",
		"1", "abc", "1foo",
		"--output", "text",
	)
	assert.NilError(t, err)
	assert.Assert(t, strings.Contains(out.String(), "  positional1: 1"))
}

func TestHelpQuery(t *testing.T) {
	fixture := initFixture(t)

	out, err := runCmd(fixture, buildModuleQueryCommand, "-h")
	assert.NilError(t, err)
	golden.Assert(t, out.String(), "help-toplevel.golden")

	out, err = runCmd(fixture, buildModuleQueryCommand, "echo", "-h")
	assert.NilError(t, err)
	golden.Assert(t, out.String(), "help-echo.golden")

	out, err = runCmd(fixture, buildModuleQueryCommand, "deprecatedecho", "echo", "-h")
	assert.NilError(t, err)
	golden.Assert(t, out.String(), "help-deprecated.golden")

	out, err = runCmd(fixture, buildModuleQueryCommand, "skipecho", "-h")
	assert.NilError(t, err)
	golden.Assert(t, out.String(), "help-skip.golden")
}

func TestDeprecatedQuery(t *testing.T) {
	fixture := initFixture(t)

	out, err := runCmd(fixture, buildModuleQueryCommand, "echo",
		"1", "abc", "--deprecated-field", "foo")
	assert.NilError(t, err)
	assert.Assert(t, strings.Contains(out.String(), "--deprecated-field has been deprecated"))

	out, err = runCmd(fixture, buildModuleQueryCommand, "echo",
		"1", "abc", "-s", "foo")
	assert.NilError(t, err)
	assert.Assert(t, strings.Contains(out.String(), "--shorthand-deprecated-field has been deprecated"))
}

func TestBuildCustomQueryCommand(t *testing.T) {
	b := &Builder{}
	customCommandCalled := false

	appOptions := AppOptions{
		ModuleOptions: map[string]*autocliv1.ModuleOptions{
			"test": {
				Query: testCmdDesc,
			},
		},
	}

	cmd, err := b.BuildQueryCommand(context.Background(), appOptions, map[string]*cobra.Command{
		"test": {Use: "test", Run: func(cmd *cobra.Command, args []string) {
			customCommandCalled = true
		}},
	})
	assert.NilError(t, err)
	cmd.SetArgs([]string{"test", "query"})
	assert.NilError(t, cmd.Execute())
	assert.Assert(t, customCommandCalled)
}

func TestNotFoundErrorsQuery(t *testing.T) {
	fixture := initFixture(t)
	b := fixture.b
	b.AddQueryConnFlags = nil
	b.AddTxConnFlags = nil

	buildModuleQueryCommand := func(_ string, cmdDescriptor *autocliv1.ServiceCommandDescriptor) (*cobra.Command, error) {
		cmd := topLevelCmd(context.Background(), "query", "Querying subcommands")
		err := b.AddMsgServiceCommands(cmd, cmdDescriptor)
		return cmd, err
	}

	// bad service
	_, err := buildModuleQueryCommand("test", &autocliv1.ServiceCommandDescriptor{Service: "foo"})
	assert.ErrorContains(t, err, "can't find service foo")

	// bad method
	_, err = buildModuleQueryCommand("test", &autocliv1.ServiceCommandDescriptor{
		Service:           testpb.Query_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{{RpcMethod: "bar"}},
	})
	assert.ErrorContains(t, err, "rpc method \"bar\" not found")

	// bad positional field
	_, err = buildModuleQueryCommand("test", &autocliv1.ServiceCommandDescriptor{
		Service: testpb.Query_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod: "Echo",
				PositionalArgs: []*autocliv1.PositionalArgDescriptor{
					{
						ProtoField: "foo",
					},
				},
			},
		},
	})
	assert.ErrorContains(t, err, "can't find field foo")

	// bad flag field
	_, err = buildModuleQueryCommand("test", &autocliv1.ServiceCommandDescriptor{
		Service: testpb.Query_ServiceDesc.ServiceName,
		RpcCommandOptions: []*autocliv1.RpcCommandOptions{
			{
				RpcMethod: "Echo",
				FlagOptions: map[string]*autocliv1.FlagOptions{
					"baz": {},
				},
			},
		},
	})
	assert.ErrorContains(t, err, "can't find field baz")
}

func TestDurationMarshal(t *testing.T) {
	fixture := initFixture(t)

	out, err := runCmd(fixture, buildModuleQueryCommand, "echo", "1", "abc", "--duration", "1s")
	assert.NilError(t, err)
	assert.Assert(t, strings.Contains(out.String(), "duration: 1s"))
}
