package main

import (
	"bufio"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/spf13/cobra"

	abcicli "github.com/cometbft/cometbft/abci/client"
	"github.com/cometbft/cometbft/abci/example/kvstore"
	"github.com/cometbft/cometbft/abci/server"
	servertest "github.com/cometbft/cometbft/abci/tests/server"
	"github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/abci/version"
	crypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	cmtos "github.com/cometbft/cometbft/internal/os"
	"github.com/cometbft/cometbft/libs/log"
)

// client is a global variable so it can be reused by the console.
var (
	client abcicli.Client
	logger log.Logger
)

// flags.
var (
	// global.
	flagAddress  string
	flagAbci     string
	flagVerbose  bool   // for the println output
	flagLogLevel string // for the logger

	// query.
	flagPath   string
	flagHeight int
	flagProve  bool

	// kvstore.
	flagPersist string
)

var RootCmd = &cobra.Command{
	Use:   "abci-cli",
	Short: "the ABCI CLI tool wraps an ABCI client",
	Long:  "the ABCI CLI tool wraps an ABCI client and is used for testing ABCI servers",
	PersistentPreRunE: func(cmd *cobra.Command, _ []string) error {
		switch cmd.Use {
		case "kvstore", "version", "help [command]":
			return nil
		}

		if logger == nil {
			allowLevel, err := log.AllowLevel(flagLogLevel)
			if err != nil {
				return err
			}
			logger = log.NewFilter(log.NewTMLogger(log.NewSyncWriter(os.Stdout)), allowLevel)
		}
		if client == nil {
			var err error
			client, err = abcicli.NewClient(flagAddress, flagAbci, false)
			if err != nil {
				return err
			}
			client.SetLogger(logger.With("module", "abci-client"))
			if err := client.Start(); err != nil {
				return err
			}
		}
		return nil
	},
}

// Structure for data passed to print response.
type response struct {
	// generic abci response
	Data   []byte
	Code   uint32
	Info   string
	Log    string
	Status int32

	Query *queryResponse
}

type queryResponse struct {
	Key      []byte
	Value    []byte
	Height   int64
	ProofOps *crypto.ProofOps
}

func Execute() error {
	addGlobalFlags()
	addCommands()
	return RootCmd.Execute()
}

func addGlobalFlags() {
	RootCmd.PersistentFlags().StringVarP(&flagAddress,
		"address",
		"",
		"tcp://0.0.0.0:26658",
		"address of application socket")
	RootCmd.PersistentFlags().StringVarP(&flagAbci, "abci", "", "socket", "either socket or grpc")
	RootCmd.PersistentFlags().BoolVarP(&flagVerbose,
		"verbose",
		"v",
		false,
		"print the command and results as if it were a console session")
	RootCmd.PersistentFlags().StringVarP(&flagLogLevel, "log_level", "", "debug", "set the logger level")
}

func addQueryFlags() {
	queryCmd.PersistentFlags().StringVarP(&flagPath, "path", "", "/store", "path to prefix query with")
	queryCmd.PersistentFlags().IntVarP(&flagHeight, "height", "", 0, "height to query the blockchain at")
	queryCmd.PersistentFlags().BoolVarP(&flagProve,
		"prove",
		"",
		false,
		"whether or not to return a merkle proof of the query result")
}

func addKVStoreFlags() {
	kvstoreCmd.PersistentFlags().StringVarP(&flagPersist, "persist", "", "", "directory to use for a database")
}

func addCommands() {
	RootCmd.AddCommand(batchCmd)
	RootCmd.AddCommand(consoleCmd)
	RootCmd.AddCommand(echoCmd)
	RootCmd.AddCommand(infoCmd)
	RootCmd.AddCommand(checkTxCmd)
	RootCmd.AddCommand(commitCmd)
	RootCmd.AddCommand(versionCmd)
	RootCmd.AddCommand(testCmd)
	RootCmd.AddCommand(prepareProposalCmd)
	RootCmd.AddCommand(processProposalCmd)
	addQueryFlags()
	RootCmd.AddCommand(queryCmd)
	RootCmd.AddCommand(finalizeBlockCmd)

	// examples
	addKVStoreFlags()
	RootCmd.AddCommand(kvstoreCmd)
}

var batchCmd = &cobra.Command{
	Use:   "batch",
	Short: "run a batch of abci commands against an application",
	Long: `run a batch of abci commands against an application

This command is run by piping in a file containing a series of commands
you'd like to run:

    abci-cli batch < example.file

where example.file looks something like:

    check_tx 0x00
    check_tx 0xff
    finalize_block 0x00
    check_tx 0x00
    finalize_block 0x01 0x04 0xff
    info
`,
	Args: cobra.ExactArgs(0),
	RunE: cmdBatch,
}

var consoleCmd = &cobra.Command{
	Use:   "console",
	Short: "start an interactive ABCI console for multiple commands",
	Long: `start an interactive ABCI console for multiple commands

This command opens an interactive console for running any of the other commands
without opening a new connection each time
`,
	Args:      cobra.ExactArgs(0),
	ValidArgs: []string{"echo", "info", "finalize_block", "check_tx", "prepare_proposal", "process_proposal", "commit", "query"},
	RunE:      cmdConsole,
}

var echoCmd = &cobra.Command{
	Use:   "echo",
	Short: "have the application echo a message",
	Long:  "have the application echo a message",
	Args:  cobra.ExactArgs(1),
	RunE:  cmdEcho,
}

var infoCmd = &cobra.Command{
	Use:   "info",
	Short: "get some info about the application",
	Long:  "get some info about the application",
	Args:  cobra.ExactArgs(0),
	RunE:  cmdInfo,
}

var finalizeBlockCmd = &cobra.Command{
	Use:   "finalize_block",
	Short: "deliver a block of transactions to the application",
	Long:  "deliver a block of transactions to the application",
	Args:  cobra.MinimumNArgs(1),
	RunE:  cmdFinalizeBlock,
}

var checkTxCmd = &cobra.Command{
	Use:   "check_tx",
	Short: "validate a transaction",
	Long:  "validate a transaction",
	Args:  cobra.ExactArgs(1),
	RunE:  cmdCheckTx,
}

var commitCmd = &cobra.Command{
	Use:   "commit",
	Short: "commit the application state and return the Merkle root hash",
	Long:  "commit the application state and return the Merkle root hash",
	Args:  cobra.ExactArgs(0),
	RunE:  cmdCommit,
}

var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "print ABCI console version",
	Long:  "print ABCI console version",
	Args:  cobra.ExactArgs(0),
	RunE: func(_ *cobra.Command, _ []string) error {
		fmt.Println(version.Version)
		return nil
	},
}

var prepareProposalCmd = &cobra.Command{
	Use:   "prepare_proposal",
	Short: "prepare proposal",
	Long:  "prepare proposal",
	Args:  cobra.MinimumNArgs(0),
	RunE:  cmdPrepareProposal,
}

var processProposalCmd = &cobra.Command{
	Use:   "process_proposal",
	Short: "process proposal",
	Long:  "process proposal",
	Args:  cobra.MinimumNArgs(0),
	RunE:  cmdProcessProposal,
}

var queryCmd = &cobra.Command{
	Use:   "query",
	Short: "query the application state",
	Long:  "query the application state",
	Args:  cobra.ExactArgs(1),
	RunE:  cmdQuery,
}

var kvstoreCmd = &cobra.Command{
	Use:   "kvstore",
	Short: "ABCI demo example",
	Long:  "ABCI demo example",
	Args:  cobra.ExactArgs(0),
	RunE:  cmdKVStore,
}

var testCmd = &cobra.Command{
	Use:   "test",
	Short: "run integration tests",
	Long:  "run integration tests",
	Args:  cobra.ExactArgs(0),
	RunE:  cmdTest,
}

// Generates new Args array based off of previous call args to maintain flag persistence.
func persistentArgs(line []byte) []string {
	// generate the arguments to run from original os.Args
	// to maintain flag arguments
	args := os.Args
	args = args[:len(args)-1] // remove the previous command argument

	if len(line) > 0 { // prevents introduction of extra space leading to argument parse errors
		args = append(args, strings.Split(string(line), " ")...)
	}
	return args
}

// --------------------------------------------------------------------------------

func compose(fs []func() error) error {
	if len(fs) == 0 {
		return nil
	}

	err := fs[0]()
	if err == nil {
		return compose(fs[1:])
	}

	return err
}

func cmdTest(cmd *cobra.Command, _ []string) error {
	ctx := cmd.Context()
	return compose(
		[]func() error{
			func() error { return servertest.InitChain(ctx, client) },
			func() error { return servertest.Commit(ctx, client) },
			func() error {
				return servertest.FinalizeBlock(ctx, client, [][]byte{
					[]byte("abc"),
				}, []uint32{
					kvstore.CodeTypeInvalidTxFormat,
				}, nil, nil)
			},
			func() error { return servertest.Commit(ctx, client) },
			func() error {
				return servertest.FinalizeBlock(ctx, client, [][]byte{
					{0x00},
				}, []uint32{
					kvstore.CodeTypeOK,
				}, nil, []byte{0, 0, 0, 0, 0, 0, 0, 1})
			},
			func() error { return servertest.Commit(ctx, client) },
			func() error {
				return servertest.FinalizeBlock(ctx, client, [][]byte{
					{0x00},
					{0x01},
					{0x00, 0x02},
					{0x00, 0x03},
					{0x00, 0x00, 0x04},
					{0x00, 0x00, 0x06},
				}, []uint32{
					kvstore.CodeTypeInvalidTxFormat,
					kvstore.CodeTypeOK,
					kvstore.CodeTypeOK,
					kvstore.CodeTypeOK,
					kvstore.CodeTypeOK,
					kvstore.CodeTypeInvalidTxFormat,
				}, nil, []byte{0, 0, 0, 0, 0, 0, 0, 5})
			},
			func() error { return servertest.Commit(ctx, client) },
			func() error {
				return servertest.PrepareProposal(ctx, client, [][]byte{
					{0x01},
				}, [][]byte{{0x01}}, nil)
			},
			func() error {
				return servertest.ProcessProposal(ctx, client, [][]byte{
					{0x01},
				}, types.PROCESS_PROPOSAL_STATUS_ACCEPT)
			},
		})
}

func cmdBatch(cmd *cobra.Command, _ []string) error {
	bufReader := bufio.NewReader(os.Stdin)
LOOP:
	for {
		line, more, err := bufReader.ReadLine()
		switch {
		case more:
			return errors.New("input line is too long")
		case errors.Is(err, io.EOF):
			break LOOP
		case len(line) == 0:
			continue
		case err != nil:
			return err
		}

		cmdArgs := persistentArgs(line)
		if err := muxOnCommands(cmd, cmdArgs); err != nil {
			return err
		}
		fmt.Println()
	}
	return nil
}

func cmdConsole(cmd *cobra.Command, _ []string) error {
	for {
		fmt.Printf("> ")
		bufReader := bufio.NewReader(os.Stdin)
		line, more, err := bufReader.ReadLine()
		if more {
			return errors.New("input is too long")
		} else if err != nil {
			return err
		}

		pArgs := persistentArgs(line)
		if err := muxOnCommands(cmd, pArgs); err != nil {
			return err
		}
	}
}

func muxOnCommands(cmd *cobra.Command, pArgs []string) error {
	if len(pArgs) < 2 {
		return errors.New("expecting persistent args of the form: abci-cli [command] <...>")
	}

	// TODO: this parsing is fragile
	args := []string{}
	for i := 0; i < len(pArgs); i++ {
		arg := pArgs[i]

		// check for flags
		if strings.HasPrefix(arg, "-") {
			// if it has an equal, we can just skip
			if strings.Contains(arg, "=") {
				continue
			}
			// if its a boolean, we can just skip
			_, err := cmd.Flags().GetBool(strings.TrimLeft(arg, "-"))
			if err == nil {
				continue
			}

			// otherwise, we need to skip the next one too
			i++
			continue
		}

		// append the actual arg
		args = append(args, arg)
	}
	var subCommand string
	var actualArgs []string
	if len(args) > 1 {
		subCommand = args[1]
	}
	if len(args) > 2 {
		actualArgs = args[2:]
	}
	cmd.Use = subCommand // for later print statements ...

	switch strings.ToLower(subCommand) {
	case "check_tx":
		return cmdCheckTx(cmd, actualArgs)
	case "commit":
		return cmdCommit(cmd, actualArgs)
	case "finalize_block":
		return cmdFinalizeBlock(cmd, actualArgs)
	case "echo":
		return cmdEcho(cmd, actualArgs)
	case "info":
		return cmdInfo(cmd, actualArgs)
	case "query":
		return cmdQuery(cmd, actualArgs)
	case "prepare_proposal":
		return cmdPrepareProposal(cmd, actualArgs)
	case "process_proposal":
		return cmdProcessProposal(cmd, actualArgs)
	default:
		return cmdUnimplemented(cmd, pArgs)
	}
}

func cmdUnimplemented(cmd *cobra.Command, args []string) error {
	msg := "unimplemented command"

	if len(args) > 0 {
		msg += fmt.Sprintf(" args: [%s]", strings.Join(args, " "))
	}
	printResponse(cmd, args, response{
		Code: codeBad,
		Log:  msg,
	})

	fmt.Println("Available commands:")
	fmt.Printf("%s: %s\n", echoCmd.Use, echoCmd.Short)
	fmt.Printf("%s: %s\n", checkTxCmd.Use, checkTxCmd.Short)
	fmt.Printf("%s: %s\n", commitCmd.Use, commitCmd.Short)
	fmt.Printf("%s: %s\n", finalizeBlockCmd.Use, finalizeBlockCmd.Short)
	fmt.Printf("%s: %s\n", infoCmd.Use, infoCmd.Short)
	fmt.Printf("%s: %s\n", queryCmd.Use, queryCmd.Short)
	fmt.Printf("%s: %s\n", prepareProposalCmd.Use, prepareProposalCmd.Short)
	fmt.Printf("%s: %s\n", processProposalCmd.Use, processProposalCmd.Short)

	fmt.Println("Use \"[command] --help\" for more information about a command.")

	return nil
}

// Have the application echo a message.
func cmdEcho(cmd *cobra.Command, args []string) error {
	msg := ""
	if len(args) > 0 {
		msg = args[0]
	}
	res, err := client.Echo(cmd.Context(), msg)
	if err != nil {
		return err
	}

	printResponse(cmd, args, response{
		Data: []byte(res.Message),
	})

	return nil
}

// Get some info from the application.
func cmdInfo(cmd *cobra.Command, args []string) error {
	var version string
	if len(args) == 1 {
		version = args[0]
	}
	res, err := client.Info(cmd.Context(), &types.InfoRequest{Version: version})
	if err != nil {
		return err
	}
	printResponse(cmd, args, response{
		Data: []byte(res.Data),
	})
	return nil
}

const codeBad uint32 = 10

// Append new txs to application.
func cmdFinalizeBlock(cmd *cobra.Command, args []string) error {
	if len(args) == 0 {
		printResponse(cmd, args, response{
			Code: codeBad,
			Log:  "Must provide at least one transaction",
		})
		return nil
	}
	txs := make([][]byte, len(args))
	for i, arg := range args {
		txBytes, err := stringOrHexToBytes(arg)
		if err != nil {
			return err
		}
		txs[i] = txBytes
	}
	res, err := client.FinalizeBlock(cmd.Context(), &types.FinalizeBlockRequest{Txs: txs})
	if err != nil {
		return err
	}
	resps := make([]response, 0, len(res.TxResults)+1)
	for _, tx := range res.TxResults {
		resps = append(resps, response{
			Code: tx.Code,
			Data: tx.Data,
			Info: tx.Info,
			Log:  tx.Log,
		})
	}
	resps = append(resps, response{
		Data: res.AppHash,
	})
	printResponse(cmd, args, resps...)
	return nil
}

// Validate a tx.
func cmdCheckTx(cmd *cobra.Command, args []string) error {
	if len(args) == 0 {
		printResponse(cmd, args, response{
			Code: codeBad,
			Info: "want the tx",
		})
		return nil
	}
	txBytes, err := stringOrHexToBytes(args[0])
	if err != nil {
		return err
	}
	res, err := client.CheckTx(cmd.Context(), &types.CheckTxRequest{
		Tx:   txBytes,
		Type: types.CHECK_TX_TYPE_CHECK,
	})
	if err != nil {
		return err
	}
	printResponse(cmd, args, response{
		Code: res.Code,
		Data: res.Data,
		Info: res.Info,
		Log:  res.Log,
	})
	return nil
}

// Get application Merkle root hash.
func cmdCommit(cmd *cobra.Command, args []string) error {
	_, err := client.Commit(cmd.Context(), &types.CommitRequest{})
	if err != nil {
		return err
	}
	printResponse(cmd, args, response{})
	return nil
}

// Query application state.
func cmdQuery(cmd *cobra.Command, args []string) error {
	if len(args) == 0 {
		printResponse(cmd, args, response{
			Code: codeBad,
			Info: "want the query",
			Log:  "",
		})
		return nil
	}
	queryBytes, err := stringOrHexToBytes(args[0])
	if err != nil {
		return err
	}

	resQuery, err := client.Query(cmd.Context(), &types.QueryRequest{
		Data:   queryBytes,
		Path:   flagPath,
		Height: int64(flagHeight),
		Prove:  flagProve,
	})
	if err != nil {
		return err
	}
	printResponse(cmd, args, response{
		Code: resQuery.Code,
		Info: resQuery.Info,
		Log:  resQuery.Log,
		Query: &queryResponse{
			Key:      resQuery.Key,
			Value:    resQuery.Value,
			Height:   resQuery.Height,
			ProofOps: resQuery.ProofOps,
		},
	})
	return nil
}

func cmdPrepareProposal(cmd *cobra.Command, args []string) error {
	txsBytesArray := make([][]byte, len(args))

	for i, arg := range args {
		txBytes, err := stringOrHexToBytes(arg)
		if err != nil {
			return err
		}
		txsBytesArray[i] = txBytes
	}

	res, err := client.PrepareProposal(cmd.Context(), &types.PrepareProposalRequest{
		Txs: txsBytesArray,
		// kvstore has to have this parameter in order not to reject a tx as the default value is 0
		MaxTxBytes: 65536,
	})
	if err != nil {
		return err
	}
	resps := make([]response, 0, len(res.Txs))
	for _, tx := range res.Txs {
		resps = append(resps, response{
			Code: 0, // CodeOK
			Log:  "Succeeded. Tx: " + string(tx),
		})
	}

	printResponse(cmd, args, resps...)
	return nil
}

func cmdProcessProposal(cmd *cobra.Command, args []string) error {
	txsBytesArray := make([][]byte, len(args))

	for i, arg := range args {
		txBytes, err := stringOrHexToBytes(arg)
		if err != nil {
			return err
		}
		txsBytesArray[i] = txBytes
	}

	res, err := client.ProcessProposal(cmd.Context(), &types.ProcessProposalRequest{
		Txs: txsBytesArray,
	})
	if err != nil {
		return err
	}

	printResponse(cmd, args, response{
		Status: int32(res.Status),
	})
	return nil
}

func cmdKVStore(*cobra.Command, []string) error {
	logger := log.NewTMLogger(log.NewSyncWriter(os.Stdout))

	// Create the application - in memory or persisted to disk
	var app types.Application
	if flagPersist == "" {
		var err error
		flagPersist, err = os.MkdirTemp("", "persistent_kvstore_tmp")
		if err != nil {
			return err
		}
	}
	app = kvstore.NewPersistentApplication(flagPersist)

	// Start the listener
	srv, err := server.NewServer(flagAddress, flagAbci, app)
	if err != nil {
		return err
	}
	srv.SetLogger(logger.With("module", "abci-server"))
	if err := srv.Start(); err != nil {
		return err
	}

	// Stop upon receiving SIGTERM or CTRL-C.
	cmtos.TrapSignal(logger, func() {
		// Cleanup
		if err := srv.Stop(); err != nil {
			logger.Error("Error while stopping server", "err", err)
		}
	})

	// Run forever.
	select {}
}

// --------------------------------------------------------------------------------

func printResponse(cmd *cobra.Command, args []string, rsps ...response) {
	if flagVerbose {
		fmt.Println(">", cmd.Use, strings.Join(args, " "))
	}

	for _, rsp := range rsps {
		// Always print the status code.
		if rsp.Code == types.CodeTypeOK {
			fmt.Printf("-> code: OK\n")
		} else {
			fmt.Printf("-> code: %d\n", rsp.Code)
		}

		if len(rsp.Data) != 0 {
			// Do no print this line when using the finalize_block command
			// because the string comes out as gibberish
			if cmd.Use != "finalize_block" {
				fmt.Printf("-> data: %s\n", rsp.Data)
			}
			fmt.Printf("-> data.hex: 0x%X\n", rsp.Data)
		}
		if rsp.Log != "" {
			fmt.Printf("-> log: %s\n", rsp.Log)
		}
		if cmd.Use == "process_proposal" {
			fmt.Printf("-> status: %s\n", types.ProcessProposalStatus(rsp.Status).String())
		}

		if rsp.Query != nil {
			fmt.Printf("-> height: %d\n", rsp.Query.Height)
			if rsp.Query.Key != nil {
				fmt.Printf("-> key: %s\n", rsp.Query.Key)
				fmt.Printf("-> key.hex: %X\n", rsp.Query.Key)
			}
			if rsp.Query.Value != nil {
				fmt.Printf("-> value: %s\n", rsp.Query.Value)
				fmt.Printf("-> value.hex: %X\n", rsp.Query.Value)
			}
			if rsp.Query.ProofOps != nil {
				fmt.Printf("-> proof: %#v\n", rsp.Query.ProofOps)
			}
		}
	}
}

// NOTE: s is interpreted as a string unless prefixed with 0x.
func stringOrHexToBytes(s string) ([]byte, error) {
	if len(s) > 2 && strings.ToLower(s[:2]) == "0x" {
		b, err := hex.DecodeString(s[2:])
		if err != nil {
			err = fmt.Errorf("error decoding hex argument: %s", err.Error())
			return nil, err
		}
		return b, nil
	}

	if !strings.HasPrefix(s, "\"") || !strings.HasSuffix(s, "\"") {
		err := fmt.Errorf("invalid string arg: \"%s\". Must be quoted or a \"0x\"-prefixed hex string", s)
		return nil, err
	}

	return []byte(s[1 : len(s)-1]), nil
}
