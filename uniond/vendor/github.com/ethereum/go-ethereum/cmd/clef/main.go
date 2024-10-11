// Copyright 2018 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"bufio"
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math/big"
	"net"
	"os"
	"os/signal"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/cmd/utils"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/internal/ethapi"
	"github.com/ethereum/go-ethereum/internal/flags"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/ethereum/go-ethereum/signer/core"
	"github.com/ethereum/go-ethereum/signer/core/apitypes"
	"github.com/ethereum/go-ethereum/signer/fourbyte"
	"github.com/ethereum/go-ethereum/signer/rules"
	"github.com/ethereum/go-ethereum/signer/storage"
	"github.com/mattn/go-colorable"
	"github.com/mattn/go-isatty"
	"github.com/urfave/cli/v2"
)

const legalWarning = `
WARNING!

Clef is an account management tool. It may, like any software, contain bugs.

Please take care to
- backup your keystore files,
- verify that the keystore(s) can be opened with your password.

Clef is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
PURPOSE. See the GNU General Public License for more details.
`

var (
	logLevelFlag = &cli.IntFlag{
		Name:  "loglevel",
		Value: 3,
		Usage: "log level to emit to the screen",
	}
	advancedMode = &cli.BoolFlag{
		Name:  "advanced",
		Usage: "If enabled, issues warnings instead of rejections for suspicious requests. Default off",
	}
	acceptFlag = &cli.BoolFlag{
		Name:  "suppress-bootwarn",
		Usage: "If set, does not show the warning during boot",
	}
	keystoreFlag = &cli.StringFlag{
		Name:  "keystore",
		Value: filepath.Join(node.DefaultDataDir(), "keystore"),
		Usage: "Directory for the keystore",
	}
	configdirFlag = &cli.StringFlag{
		Name:  "configdir",
		Value: DefaultConfigDir(),
		Usage: "Directory for Clef configuration",
	}
	chainIdFlag = &cli.Int64Flag{
		Name:  "chainid",
		Value: params.MainnetChainConfig.ChainID.Int64(),
		Usage: "Chain id to use for signing (1=mainnet, 5=Goerli)",
	}
	rpcPortFlag = &cli.IntFlag{
		Name:     "http.port",
		Usage:    "HTTP-RPC server listening port",
		Value:    node.DefaultHTTPPort + 5,
		Category: flags.APICategory,
	}
	signerSecretFlag = &cli.StringFlag{
		Name:  "signersecret",
		Usage: "A file containing the (encrypted) master seed to encrypt Clef data, e.g. keystore credentials and ruleset hash",
	}
	customDBFlag = &cli.StringFlag{
		Name:  "4bytedb-custom",
		Usage: "File used for writing new 4byte-identifiers submitted via API",
		Value: "./4byte-custom.json",
	}
	auditLogFlag = &cli.StringFlag{
		Name:  "auditlog",
		Usage: "File used to emit audit logs. Set to \"\" to disable",
		Value: "audit.log",
	}
	ruleFlag = &cli.StringFlag{
		Name:  "rules",
		Usage: "Path to the rule file to auto-authorize requests with",
	}
	stdiouiFlag = &cli.BoolFlag{
		Name: "stdio-ui",
		Usage: "Use STDIN/STDOUT as a channel for an external UI. " +
			"This means that an STDIN/STDOUT is used for RPC-communication with a e.g. a graphical user " +
			"interface, and can be used when Clef is started by an external process.",
	}
	testFlag = &cli.BoolFlag{
		Name:  "stdio-ui-test",
		Usage: "Mechanism to test interface between Clef and UI. Requires 'stdio-ui'.",
	}
	initCommand = &cli.Command{
		Action:    initializeSecrets,
		Name:      "init",
		Usage:     "Initialize the signer, generate secret storage",
		ArgsUsage: "",
		Flags: []cli.Flag{
			logLevelFlag,
			configdirFlag,
		},
		Description: `
The init command generates a master seed which Clef can use to store credentials and data needed for
the rule-engine to work.`,
	}
	attestCommand = &cli.Command{
		Action:    attestFile,
		Name:      "attest",
		Usage:     "Attest that a js-file is to be used",
		ArgsUsage: "<sha256sum>",
		Flags: []cli.Flag{
			logLevelFlag,
			configdirFlag,
			signerSecretFlag,
		},
		Description: `
The attest command stores the sha256 of the rule.js-file that you want to use for automatic processing of
incoming requests.

Whenever you make an edit to the rule file, you need to use attestation to tell
Clef that the file is 'safe' to execute.`,
	}
	setCredentialCommand = &cli.Command{
		Action:    setCredential,
		Name:      "setpw",
		Usage:     "Store a credential for a keystore file",
		ArgsUsage: "<address>",
		Flags: []cli.Flag{
			logLevelFlag,
			configdirFlag,
			signerSecretFlag,
		},
		Description: `
The setpw command stores a password for a given address (keyfile).
`}
	delCredentialCommand = &cli.Command{
		Action:    removeCredential,
		Name:      "delpw",
		Usage:     "Remove a credential for a keystore file",
		ArgsUsage: "<address>",
		Flags: []cli.Flag{
			logLevelFlag,
			configdirFlag,
			signerSecretFlag,
		},
		Description: `
The delpw command removes a password for a given address (keyfile).
`}
	newAccountCommand = &cli.Command{
		Action:    newAccount,
		Name:      "newaccount",
		Usage:     "Create a new account",
		ArgsUsage: "",
		Flags: []cli.Flag{
			logLevelFlag,
			keystoreFlag,
			utils.LightKDFFlag,
			acceptFlag,
		},
		Description: `
The newaccount command creates a new keystore-backed account. It is a convenience-method
which can be used in lieu of an external UI.
`}
	gendocCommand = &cli.Command{
		Action: GenDoc,
		Name:   "gendoc",
		Usage:  "Generate documentation about json-rpc format",
		Description: `
The gendoc generates example structures of the json-rpc communication types.
`}
	listAccountsCommand = &cli.Command{
		Action: listAccounts,
		Name:   "list-accounts",
		Usage:  "List accounts in the keystore",
		Flags: []cli.Flag{
			logLevelFlag,
			keystoreFlag,
			utils.LightKDFFlag,
			acceptFlag,
		},
		Description: `
	Lists the accounts in the keystore.
	`}
	listWalletsCommand = &cli.Command{
		Action: listWallets,
		Name:   "list-wallets",
		Usage:  "List wallets known to Clef",
		Flags: []cli.Flag{
			logLevelFlag,
			keystoreFlag,
			utils.LightKDFFlag,
			acceptFlag,
		},
		Description: `
	Lists the wallets known to Clef.
	`}
	importRawCommand = &cli.Command{
		Action:    accountImport,
		Name:      "importraw",
		Usage:     "Import a hex-encoded private key.",
		ArgsUsage: "<keyfile>",
		Flags: []cli.Flag{
			logLevelFlag,
			keystoreFlag,
			utils.LightKDFFlag,
			acceptFlag,
		},
		Description: `
Imports an unencrypted private key from <keyfile> and creates a new account.
Prints the address.
The keyfile is assumed to contain an unencrypted private key in hexadecimal format.
The account is saved in encrypted format, you are prompted for a password.
`}
)

var app = flags.NewApp("Manage Ethereum account operations")

func init() {
	app.Name = "Clef"
	app.Flags = []cli.Flag{
		logLevelFlag,
		keystoreFlag,
		configdirFlag,
		chainIdFlag,
		utils.LightKDFFlag,
		utils.NoUSBFlag,
		utils.SmartCardDaemonPathFlag,
		utils.HTTPListenAddrFlag,
		utils.HTTPVirtualHostsFlag,
		utils.IPCDisabledFlag,
		utils.IPCPathFlag,
		utils.HTTPEnabledFlag,
		rpcPortFlag,
		signerSecretFlag,
		customDBFlag,
		auditLogFlag,
		ruleFlag,
		stdiouiFlag,
		testFlag,
		advancedMode,
		acceptFlag,
	}
	app.Action = signer
	app.Commands = []*cli.Command{initCommand,
		attestCommand,
		setCredentialCommand,
		delCredentialCommand,
		newAccountCommand,
		importRawCommand,
		gendocCommand,
		listAccountsCommand,
		listWalletsCommand,
	}
}

func main() {
	if err := app.Run(os.Args); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func initializeSecrets(c *cli.Context) error {
	// Get past the legal message
	if err := initialize(c); err != nil {
		return err
	}
	// Ensure the master key does not yet exist, we're not willing to overwrite
	configDir := c.String(configdirFlag.Name)
	if err := os.Mkdir(configDir, 0700); err != nil && !os.IsExist(err) {
		return err
	}
	location := filepath.Join(configDir, "masterseed.json")
	if _, err := os.Stat(location); err == nil {
		return fmt.Errorf("master key %v already exists, will not overwrite", location)
	}
	// Key file does not exist yet, generate a new one and encrypt it
	masterSeed := make([]byte, 256)
	num, err := io.ReadFull(rand.Reader, masterSeed)
	if err != nil {
		return err
	}
	if num != len(masterSeed) {
		return errors.New("failed to read enough random")
	}
	n, p := keystore.StandardScryptN, keystore.StandardScryptP
	if c.Bool(utils.LightKDFFlag.Name) {
		n, p = keystore.LightScryptN, keystore.LightScryptP
	}
	text := "The master seed of clef will be locked with a password.\nPlease specify a password. Do not forget this password!"
	var password string
	for {
		password = utils.GetPassPhrase(text, true)
		if err := core.ValidatePasswordFormat(password); err != nil {
			fmt.Printf("invalid password: %v\n", err)
		} else {
			fmt.Println()
			break
		}
	}
	cipherSeed, err := encryptSeed(masterSeed, []byte(password), n, p)
	if err != nil {
		return fmt.Errorf("failed to encrypt master seed: %v", err)
	}
	// Double check the master key path to ensure nothing wrote there in between
	if err = os.Mkdir(configDir, 0700); err != nil && !os.IsExist(err) {
		return err
	}
	if _, err := os.Stat(location); err == nil {
		return fmt.Errorf("master key %v already exists, will not overwrite", location)
	}
	// Write the file and print the usual warning message
	if err = os.WriteFile(location, cipherSeed, 0400); err != nil {
		return err
	}
	fmt.Printf("A master seed has been generated into %s\n", location)
	fmt.Printf(`
This is required to be able to store credentials, such as:
* Passwords for keystores (used by rule engine)
* Storage for JavaScript auto-signing rules
* Hash of JavaScript rule-file

You should treat 'masterseed.json' with utmost secrecy and make a backup of it!
* The password is necessary but not enough, you need to back up the master seed too!
* The master seed does not contain your accounts, those need to be backed up separately!

`)
	return nil
}

func attestFile(ctx *cli.Context) error {
	if ctx.NArg() < 1 {
		utils.Fatalf("This command requires an argument.")
	}
	if err := initialize(ctx); err != nil {
		return err
	}

	stretchedKey, err := readMasterKey(ctx, nil)
	if err != nil {
		utils.Fatalf(err.Error())
	}
	configDir := ctx.String(configdirFlag.Name)
	vaultLocation := filepath.Join(configDir, common.Bytes2Hex(crypto.Keccak256([]byte("vault"), stretchedKey)[:10]))
	confKey := crypto.Keccak256([]byte("config"), stretchedKey)

	// Initialize the encrypted storages
	configStorage := storage.NewAESEncryptedStorage(filepath.Join(vaultLocation, "config.json"), confKey)
	val := ctx.Args().First()
	configStorage.Put("ruleset_sha256", val)
	log.Info("Ruleset attestation updated", "sha256", val)
	return nil
}

func initInternalApi(c *cli.Context) (*core.UIServerAPI, core.UIClientAPI, error) {
	if err := initialize(c); err != nil {
		return nil, nil, err
	}
	var (
		ui                        = core.NewCommandlineUI()
		pwStorage storage.Storage = &storage.NoStorage{}
		ksLoc                     = c.String(keystoreFlag.Name)
		lightKdf                  = c.Bool(utils.LightKDFFlag.Name)
	)
	am := core.StartClefAccountManager(ksLoc, true, lightKdf, "")
	api := core.NewSignerAPI(am, 0, true, ui, nil, false, pwStorage)
	internalApi := core.NewUIServerAPI(api)
	return internalApi, ui, nil
}

func setCredential(ctx *cli.Context) error {
	if ctx.NArg() < 1 {
		utils.Fatalf("This command requires an address to be passed as an argument")
	}
	if err := initialize(ctx); err != nil {
		return err
	}
	addr := ctx.Args().First()
	if !common.IsHexAddress(addr) {
		utils.Fatalf("Invalid address specified: %s", addr)
	}
	address := common.HexToAddress(addr)
	password := utils.GetPassPhrase("Please enter a password to store for this address:", true)
	fmt.Println()

	stretchedKey, err := readMasterKey(ctx, nil)
	if err != nil {
		utils.Fatalf(err.Error())
	}
	configDir := ctx.String(configdirFlag.Name)
	vaultLocation := filepath.Join(configDir, common.Bytes2Hex(crypto.Keccak256([]byte("vault"), stretchedKey)[:10]))
	pwkey := crypto.Keccak256([]byte("credentials"), stretchedKey)

	pwStorage := storage.NewAESEncryptedStorage(filepath.Join(vaultLocation, "credentials.json"), pwkey)
	pwStorage.Put(address.Hex(), password)

	log.Info("Credential store updated", "set", address)
	return nil
}

func removeCredential(ctx *cli.Context) error {
	if ctx.NArg() < 1 {
		utils.Fatalf("This command requires an address to be passed as an argument")
	}
	if err := initialize(ctx); err != nil {
		return err
	}
	addr := ctx.Args().First()
	if !common.IsHexAddress(addr) {
		utils.Fatalf("Invalid address specified: %s", addr)
	}
	address := common.HexToAddress(addr)

	stretchedKey, err := readMasterKey(ctx, nil)
	if err != nil {
		utils.Fatalf(err.Error())
	}
	configDir := ctx.String(configdirFlag.Name)
	vaultLocation := filepath.Join(configDir, common.Bytes2Hex(crypto.Keccak256([]byte("vault"), stretchedKey)[:10]))
	pwkey := crypto.Keccak256([]byte("credentials"), stretchedKey)

	pwStorage := storage.NewAESEncryptedStorage(filepath.Join(vaultLocation, "credentials.json"), pwkey)
	pwStorage.Del(address.Hex())

	log.Info("Credential store updated", "unset", address)
	return nil
}

func initialize(c *cli.Context) error {
	// Set up the logger to print everything
	logOutput := os.Stdout
	if c.Bool(stdiouiFlag.Name) {
		logOutput = os.Stderr
		// If using the stdioui, we can't do the 'confirm'-flow
		if !c.Bool(acceptFlag.Name) {
			fmt.Fprint(logOutput, legalWarning)
		}
	} else if !c.Bool(acceptFlag.Name) {
		if !confirm(legalWarning) {
			return errors.New("aborted by user")
		}
		fmt.Println()
	}
	usecolor := (isatty.IsTerminal(os.Stderr.Fd()) || isatty.IsCygwinTerminal(os.Stderr.Fd())) && os.Getenv("TERM") != "dumb"
	output := io.Writer(logOutput)
	if usecolor {
		output = colorable.NewColorable(logOutput)
	}
	log.Root().SetHandler(log.LvlFilterHandler(log.Lvl(c.Int(logLevelFlag.Name)), log.StreamHandler(output, log.TerminalFormat(usecolor))))

	return nil
}

func newAccount(c *cli.Context) error {
	internalApi, _, err := initInternalApi(c)
	if err != nil {
		return err
	}
	addr, err := internalApi.New(context.Background())
	if err == nil {
		fmt.Printf("Generated account %v\n", addr.String())
	}
	return err
}

func listAccounts(c *cli.Context) error {
	internalApi, _, err := initInternalApi(c)
	if err != nil {
		return err
	}
	accs, err := internalApi.ListAccounts(context.Background())
	if err != nil {
		return err
	}
	if len(accs) == 0 {
		fmt.Println("\nThe keystore is empty.")
	}
	fmt.Println()
	for _, account := range accs {
		fmt.Printf("%v (%v)\n", account.Address, account.URL)
	}
	return err
}

func listWallets(c *cli.Context) error {
	internalApi, _, err := initInternalApi(c)
	if err != nil {
		return err
	}
	wallets := internalApi.ListWallets()
	if len(wallets) == 0 {
		fmt.Println("\nThere are no wallets.")
	}
	fmt.Println()
	for i, wallet := range wallets {
		fmt.Printf("- Wallet %d at %v (%v %v)\n", i, wallet.URL, wallet.Status, wallet.Failure)
		for j, acc := range wallet.Accounts {
			fmt.Printf("  -Account %d: %v (%v)\n", j, acc.Address, acc.URL)
		}
		fmt.Println()
	}
	return nil
}

// accountImport imports a raw hexadecimal private key via CLI.
func accountImport(c *cli.Context) error {
	if c.Args().Len() != 1 {
		return errors.New("<keyfile> must be given as first argument.")
	}
	internalApi, ui, err := initInternalApi(c)
	if err != nil {
		return err
	}
	pKey, err := crypto.LoadECDSA(c.Args().First())
	if err != nil {
		return err
	}
	readPw := func(prompt string) (string, error) {
		resp, err := ui.OnInputRequired(core.UserInputRequest{
			Title:      "Password",
			Prompt:     prompt,
			IsPassword: true,
		})
		if err != nil {
			return "", err
		}
		return resp.Text, nil
	}
	first, err := readPw("Please enter a password for the imported account")
	if err != nil {
		return err
	}
	second, err := readPw("Please repeat the password you just entered")
	if err != nil {
		return err
	}
	if first != second {
		return errors.New("Passwords do not match")
	}
	acc, err := internalApi.ImportRawKey(hex.EncodeToString(crypto.FromECDSA(pKey)), first)
	if err != nil {
		return err
	}
	ui.ShowInfo(fmt.Sprintf(`Key imported:
  Address %v
  Keystore file: %v

The key is now encrypted; losing the password will result in permanently losing
access to the key and all associated funds!

Make sure to backup keystore and passwords in a safe location.`,
		acc.Address, acc.URL.Path))
	return nil
}

// ipcEndpoint resolves an IPC endpoint based on a configured value, taking into
// account the set data folders as well as the designated platform we're currently
// running on.
func ipcEndpoint(ipcPath, datadir string) string {
	// On windows we can only use plain top-level pipes
	if runtime.GOOS == "windows" {
		if strings.HasPrefix(ipcPath, `\\.\pipe\`) {
			return ipcPath
		}
		return `\\.\pipe\` + ipcPath
	}
	// Resolve names into the data directory full paths otherwise
	if filepath.Base(ipcPath) == ipcPath {
		if datadir == "" {
			return filepath.Join(os.TempDir(), ipcPath)
		}
		return filepath.Join(datadir, ipcPath)
	}
	return ipcPath
}

func signer(c *cli.Context) error {
	// If we have some unrecognized command, bail out
	if c.NArg() > 0 {
		return fmt.Errorf("invalid command: %q", c.Args().First())
	}
	if err := initialize(c); err != nil {
		return err
	}
	var (
		ui core.UIClientAPI
	)
	if c.Bool(stdiouiFlag.Name) {
		log.Info("Using stdin/stdout as UI-channel")
		ui = core.NewStdIOUI()
	} else {
		log.Info("Using CLI as UI-channel")
		ui = core.NewCommandlineUI()
	}
	// 4bytedb data
	fourByteLocal := c.String(customDBFlag.Name)
	db, err := fourbyte.NewWithFile(fourByteLocal)
	if err != nil {
		utils.Fatalf(err.Error())
	}
	embeds, locals := db.Size()
	log.Info("Loaded 4byte database", "embeds", embeds, "locals", locals, "local", fourByteLocal)

	var (
		api       core.ExternalAPI
		pwStorage storage.Storage = &storage.NoStorage{}
	)
	configDir := c.String(configdirFlag.Name)
	if stretchedKey, err := readMasterKey(c, ui); err != nil {
		log.Warn("Failed to open master, rules disabled", "err", err)
	} else {
		vaultLocation := filepath.Join(configDir, common.Bytes2Hex(crypto.Keccak256([]byte("vault"), stretchedKey)[:10]))

		// Generate domain specific keys
		pwkey := crypto.Keccak256([]byte("credentials"), stretchedKey)
		jskey := crypto.Keccak256([]byte("jsstorage"), stretchedKey)
		confkey := crypto.Keccak256([]byte("config"), stretchedKey)

		// Initialize the encrypted storages
		pwStorage = storage.NewAESEncryptedStorage(filepath.Join(vaultLocation, "credentials.json"), pwkey)
		jsStorage := storage.NewAESEncryptedStorage(filepath.Join(vaultLocation, "jsstorage.json"), jskey)
		configStorage := storage.NewAESEncryptedStorage(filepath.Join(vaultLocation, "config.json"), confkey)

		// Do we have a rule-file?
		if ruleFile := c.String(ruleFlag.Name); ruleFile != "" {
			ruleJS, err := os.ReadFile(ruleFile)
			if err != nil {
				log.Warn("Could not load rules, disabling", "file", ruleFile, "err", err)
			} else {
				shasum := sha256.Sum256(ruleJS)
				foundShaSum := hex.EncodeToString(shasum[:])
				storedShasum, _ := configStorage.Get("ruleset_sha256")
				if storedShasum != foundShaSum {
					log.Warn("Rule hash not attested, disabling", "hash", foundShaSum, "attested", storedShasum)
				} else {
					// Initialize rules
					ruleEngine, err := rules.NewRuleEvaluator(ui, jsStorage)
					if err != nil {
						utils.Fatalf(err.Error())
					}
					ruleEngine.Init(string(ruleJS))
					ui = ruleEngine
					log.Info("Rule engine configured", "file", c.String(ruleFlag.Name))
				}
			}
		}
	}
	var (
		chainId  = c.Int64(chainIdFlag.Name)
		ksLoc    = c.String(keystoreFlag.Name)
		lightKdf = c.Bool(utils.LightKDFFlag.Name)
		advanced = c.Bool(advancedMode.Name)
		nousb    = c.Bool(utils.NoUSBFlag.Name)
		scpath   = c.String(utils.SmartCardDaemonPathFlag.Name)
	)
	log.Info("Starting signer", "chainid", chainId, "keystore", ksLoc,
		"light-kdf", lightKdf, "advanced", advanced)
	am := core.StartClefAccountManager(ksLoc, nousb, lightKdf, scpath)
	apiImpl := core.NewSignerAPI(am, chainId, nousb, ui, db, advanced, pwStorage)

	// Establish the bidirectional communication, by creating a new UI backend and registering
	// it with the UI.
	ui.RegisterUIServer(core.NewUIServerAPI(apiImpl))
	api = apiImpl

	// Audit logging
	if logfile := c.String(auditLogFlag.Name); logfile != "" {
		api, err = core.NewAuditLogger(logfile, api)
		if err != nil {
			utils.Fatalf(err.Error())
		}
		log.Info("Audit logs configured", "file", logfile)
	}
	// register signer API with server
	var (
		extapiURL = "n/a"
		ipcapiURL = "n/a"
	)
	rpcAPI := []rpc.API{
		{
			Namespace: "account",
			Service:   api,
		},
	}
	if c.Bool(utils.HTTPEnabledFlag.Name) {
		vhosts := utils.SplitAndTrim(c.String(utils.HTTPVirtualHostsFlag.Name))
		cors := utils.SplitAndTrim(c.String(utils.HTTPCORSDomainFlag.Name))

		srv := rpc.NewServer()
		srv.SetBatchLimits(node.DefaultConfig.BatchRequestLimit, node.DefaultConfig.BatchResponseMaxSize)
		err := node.RegisterApis(rpcAPI, []string{"account"}, srv)
		if err != nil {
			utils.Fatalf("Could not register API: %w", err)
		}
		handler := node.NewHTTPHandlerStack(srv, cors, vhosts, nil)

		// set port
		port := c.Int(rpcPortFlag.Name)

		// start http server
		httpEndpoint := net.JoinHostPort(c.String(utils.HTTPListenAddrFlag.Name), fmt.Sprintf("%d", port))
		httpServer, addr, err := node.StartHTTPEndpoint(httpEndpoint, rpc.DefaultHTTPTimeouts, handler)
		if err != nil {
			utils.Fatalf("Could not start RPC api: %v", err)
		}
		extapiURL = fmt.Sprintf("http://%v/", addr)
		log.Info("HTTP endpoint opened", "url", extapiURL)

		defer func() {
			// Don't bother imposing a timeout here.
			httpServer.Shutdown(context.Background())
			log.Info("HTTP endpoint closed", "url", extapiURL)
		}()
	}
	if !c.Bool(utils.IPCDisabledFlag.Name) {
		givenPath := c.String(utils.IPCPathFlag.Name)
		ipcapiURL = ipcEndpoint(filepath.Join(givenPath, "clef.ipc"), configDir)
		listener, _, err := rpc.StartIPCEndpoint(ipcapiURL, rpcAPI)
		if err != nil {
			utils.Fatalf("Could not start IPC api: %v", err)
		}
		log.Info("IPC endpoint opened", "url", ipcapiURL)
		defer func() {
			listener.Close()
			log.Info("IPC endpoint closed", "url", ipcapiURL)
		}()
	}
	if c.Bool(testFlag.Name) {
		log.Info("Performing UI test")
		go testExternalUI(apiImpl)
	}
	ui.OnSignerStartup(core.StartupInfo{
		Info: map[string]interface{}{
			"intapi_version": core.InternalAPIVersion,
			"extapi_version": core.ExternalAPIVersion,
			"extapi_http":    extapiURL,
			"extapi_ipc":     ipcapiURL,
		}})

	abortChan := make(chan os.Signal, 1)
	signal.Notify(abortChan, os.Interrupt)

	sig := <-abortChan
	log.Info("Exiting...", "signal", sig)

	return nil
}

// DefaultConfigDir is the default config directory to use for the vaults and other
// persistence requirements.
func DefaultConfigDir() string {
	// Try to place the data folder in the user's home dir
	home := flags.HomeDir()
	if home != "" {
		if runtime.GOOS == "darwin" {
			return filepath.Join(home, "Library", "Signer")
		} else if runtime.GOOS == "windows" {
			appdata := os.Getenv("APPDATA")
			if appdata != "" {
				return filepath.Join(appdata, "Signer")
			}
			return filepath.Join(home, "AppData", "Roaming", "Signer")
		}
		return filepath.Join(home, ".clef")
	}
	// As we cannot guess a stable location, return empty and handle later
	return ""
}

func readMasterKey(ctx *cli.Context, ui core.UIClientAPI) ([]byte, error) {
	var (
		file      string
		configDir = ctx.String(configdirFlag.Name)
	)
	if ctx.IsSet(signerSecretFlag.Name) {
		file = ctx.String(signerSecretFlag.Name)
	} else {
		file = filepath.Join(configDir, "masterseed.json")
	}
	if err := checkFile(file); err != nil {
		return nil, err
	}
	cipherKey, err := os.ReadFile(file)
	if err != nil {
		return nil, err
	}
	var password string
	// If ui is not nil, get the password from ui.
	if ui != nil {
		resp, err := ui.OnInputRequired(core.UserInputRequest{
			Title:      "Master Password",
			Prompt:     "Please enter the password to decrypt the master seed",
			IsPassword: true})
		if err != nil {
			return nil, err
		}
		password = resp.Text
	} else {
		password = utils.GetPassPhrase("Decrypt master seed of clef", false)
	}
	masterSeed, err := decryptSeed(cipherKey, password)
	if err != nil {
		return nil, errors.New("failed to decrypt the master seed of clef")
	}
	if len(masterSeed) < 256 {
		return nil, fmt.Errorf("master seed of insufficient length, expected >255 bytes, got %d", len(masterSeed))
	}
	// Create vault location
	vaultLocation := filepath.Join(configDir, common.Bytes2Hex(crypto.Keccak256([]byte("vault"), masterSeed)[:10]))
	err = os.Mkdir(vaultLocation, 0700)
	if err != nil && !os.IsExist(err) {
		return nil, err
	}
	return masterSeed, nil
}

// checkFile is a convenience function to check if a file
// * exists
// * is mode 0400 (unix only)
func checkFile(filename string) error {
	info, err := os.Stat(filename)
	if err != nil {
		return fmt.Errorf("failed stat on %s: %v", filename, err)
	}
	// Check the unix permission bits
	// However, on windows, we cannot use the unix perm-bits, see
	// https://github.com/ethereum/go-ethereum/issues/20123
	if runtime.GOOS != "windows" && info.Mode().Perm()&0377 != 0 {
		return fmt.Errorf("file (%v) has insecure file permissions (%v)", filename, info.Mode().String())
	}
	return nil
}

// confirm displays a text and asks for user confirmation
func confirm(text string) bool {
	fmt.Print(text)
	fmt.Printf("\nEnter 'ok' to proceed:\n> ")

	text, err := bufio.NewReader(os.Stdin).ReadString('\n')
	if err != nil {
		log.Crit("Failed to read user input", "err", err)
	}
	if text := strings.TrimSpace(text); text == "ok" {
		return true
	}
	return false
}

func testExternalUI(api *core.SignerAPI) {
	ctx := context.WithValue(context.Background(), "remote", "clef binary")
	ctx = context.WithValue(ctx, "scheme", "in-proc")
	ctx = context.WithValue(ctx, "local", "main")
	errs := make([]string, 0)

	a := common.HexToAddress("0xdeadbeef000000000000000000000000deadbeef")
	addErr := func(errStr string) {
		log.Info("Test error", "err", errStr)
		errs = append(errs, errStr)
	}

	queryUser := func(q string) string {
		resp, err := api.UI.OnInputRequired(core.UserInputRequest{
			Title:  "Testing",
			Prompt: q,
		})
		if err != nil {
			addErr(err.Error())
		}
		return resp.Text
	}
	expectResponse := func(testcase, question, expect string) {
		if got := queryUser(question); got != expect {
			addErr(fmt.Sprintf("%s: got %v, expected %v", testcase, got, expect))
		}
	}
	expectApprove := func(testcase string, err error) {
		if err == nil || err == accounts.ErrUnknownAccount {
			return
		}
		addErr(fmt.Sprintf("%v: expected no error, got %v", testcase, err.Error()))
	}
	expectDeny := func(testcase string, err error) {
		if err == nil || err != core.ErrRequestDenied {
			addErr(fmt.Sprintf("%v: expected ErrRequestDenied, got %v", testcase, err))
		}
	}
	var delay = 1 * time.Second
	// Test display of info and error
	{
		api.UI.ShowInfo("If you see this message, enter 'yes' to next question")
		time.Sleep(delay)
		expectResponse("showinfo", "Did you see the message? [yes/no]", "yes")
		api.UI.ShowError("If you see this message, enter 'yes' to the next question")
		time.Sleep(delay)
		expectResponse("showerror", "Did you see the message? [yes/no]", "yes")
	}
	{ // Sign data test - clique header
		api.UI.ShowInfo("Please approve the next request for signing a clique header")
		time.Sleep(delay)
		cliqueHeader := types.Header{
			ParentHash:  common.HexToHash("0000H45H"),
			UncleHash:   common.HexToHash("0000H45H"),
			Coinbase:    common.HexToAddress("0000H45H"),
			Root:        common.HexToHash("0000H00H"),
			TxHash:      common.HexToHash("0000H45H"),
			ReceiptHash: common.HexToHash("0000H45H"),
			Difficulty:  big.NewInt(1337),
			Number:      big.NewInt(1337),
			GasLimit:    1338,
			GasUsed:     1338,
			Time:        1338,
			Extra:       []byte("Extra data Extra data Extra data  Extra data  Extra data  Extra data  Extra data Extra data"),
			MixDigest:   common.HexToHash("0x0000H45H"),
		}
		cliqueRlp, err := rlp.EncodeToBytes(cliqueHeader)
		if err != nil {
			utils.Fatalf("Should not error: %v", err)
		}
		addr, _ := common.NewMixedcaseAddressFromString("0x0011223344556677889900112233445566778899")
		_, err = api.SignData(ctx, accounts.MimetypeClique, *addr, hexutil.Encode(cliqueRlp))
		expectApprove("signdata - clique header", err)
	}
	{ // Sign data test - typed data
		api.UI.ShowInfo("Please approve the next request for signing EIP-712 typed data")
		time.Sleep(delay)
		addr, _ := common.NewMixedcaseAddressFromString("0x0011223344556677889900112233445566778899")
		data := `{"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"version","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"Person":[{"name":"name","type":"string"},{"name":"test","type":"uint8"},{"name":"wallet","type":"address"}],"Mail":[{"name":"from","type":"Person"},{"name":"to","type":"Person"},{"name":"contents","type":"string"}]},"primaryType":"Mail","domain":{"name":"Ether Mail","version":"1","chainId":"1","verifyingContract":"0xCCCcccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"},"message":{"from":{"name":"Cow","test":"3","wallet":"0xcD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"},"to":{"name":"Bob","wallet":"0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB","test":"2"},"contents":"Hello, Bob!"}}`
		//_, err := api.SignData(ctx, accounts.MimetypeTypedData, *addr, hexutil.Encode([]byte(data)))
		var typedData apitypes.TypedData
		json.Unmarshal([]byte(data), &typedData)
		_, err := api.SignTypedData(ctx, *addr, typedData)
		expectApprove("sign 712 typed data", err)
	}
	{ // Sign data test - plain text
		api.UI.ShowInfo("Please approve the next request for signing text")
		time.Sleep(delay)
		addr, _ := common.NewMixedcaseAddressFromString("0x0011223344556677889900112233445566778899")
		_, err := api.SignData(ctx, accounts.MimetypeTextPlain, *addr, hexutil.Encode([]byte("hello world")))
		expectApprove("signdata - text", err)
	}
	{ // Sign data test - plain text reject
		api.UI.ShowInfo("Please deny the next request for signing text")
		time.Sleep(delay)
		addr, _ := common.NewMixedcaseAddressFromString("0x0011223344556677889900112233445566778899")
		_, err := api.SignData(ctx, accounts.MimetypeTextPlain, *addr, hexutil.Encode([]byte("hello world")))
		expectDeny("signdata - text", err)
	}
	{ // Sign transaction
		api.UI.ShowInfo("Please reject next transaction")
		time.Sleep(delay)
		data := hexutil.Bytes([]byte{})
		to := common.NewMixedcaseAddress(a)
		tx := apitypes.SendTxArgs{
			Data:     &data,
			Nonce:    0x1,
			Value:    hexutil.Big(*big.NewInt(6)),
			From:     common.NewMixedcaseAddress(a),
			To:       &to,
			GasPrice: (*hexutil.Big)(big.NewInt(5)),
			Gas:      1000,
			Input:    nil,
		}
		_, err := api.SignTransaction(ctx, tx, nil)
		expectDeny("signtransaction [1]", err)
		expectResponse("signtransaction [2]", "Did you see any warnings for the last transaction? (yes/no)", "no")
	}
	{ // Listing
		api.UI.ShowInfo("Please reject listing-request")
		time.Sleep(delay)
		_, err := api.List(ctx)
		expectDeny("list", err)
	}
	{ // Import
		api.UI.ShowInfo("Please reject new account-request")
		time.Sleep(delay)
		_, err := api.New(ctx)
		expectDeny("newaccount", err)
	}
	{ // Metadata
		api.UI.ShowInfo("Please check if you see the Origin in next listing (approve or deny)")
		time.Sleep(delay)
		api.List(context.WithValue(ctx, "Origin", "origin.com"))
		expectResponse("metadata - origin", "Did you see origin (origin.com)? [yes/no] ", "yes")
	}

	for _, e := range errs {
		log.Error(e)
	}
	result := fmt.Sprintf("Tests completed. %d errors:\n%s\n", len(errs), strings.Join(errs, "\n"))
	api.UI.ShowInfo(result)
}

type encryptedSeedStorage struct {
	Description string              `json:"description"`
	Version     int                 `json:"version"`
	Params      keystore.CryptoJSON `json:"params"`
}

// encryptSeed uses a similar scheme as the keystore uses, but with a different wrapping,
// to encrypt the master seed
func encryptSeed(seed []byte, auth []byte, scryptN, scryptP int) ([]byte, error) {
	cryptoStruct, err := keystore.EncryptDataV3(seed, auth, scryptN, scryptP)
	if err != nil {
		return nil, err
	}
	return json.Marshal(&encryptedSeedStorage{"Clef seed", 1, cryptoStruct})
}

// decryptSeed decrypts the master seed
func decryptSeed(keyjson []byte, auth string) ([]byte, error) {
	var encSeed encryptedSeedStorage
	if err := json.Unmarshal(keyjson, &encSeed); err != nil {
		return nil, err
	}
	if encSeed.Version != 1 {
		log.Warn(fmt.Sprintf("unsupported encryption format of seed: %d, operation will likely fail", encSeed.Version))
	}
	seed, err := keystore.DecryptDataV3(encSeed.Params, auth)
	if err != nil {
		return nil, err
	}
	return seed, err
}

// GenDoc outputs examples of all structures used in json-rpc communication
func GenDoc(ctx *cli.Context) error {
	var (
		a    = common.HexToAddress("0xdeadbeef000000000000000000000000deadbeef")
		b    = common.HexToAddress("0x1111111122222222222233333333334444444444")
		meta = core.Metadata{
			Scheme:    "http",
			Local:     "localhost:8545",
			Origin:    "www.malicious.ru",
			Remote:    "localhost:9999",
			UserAgent: "Firefox 3.2",
		}
		output []string
		add    = func(name, desc string, v interface{}) {
			if data, err := json.MarshalIndent(v, "", "  "); err == nil {
				output = append(output, fmt.Sprintf("### %s\n\n%s\n\nExample:\n```json\n%s\n```", name, desc, data))
			} else {
				log.Error("Error generating output", "err", err)
			}
		}
	)

	{ // Sign plain text request
		desc := "SignDataRequest contains information about a pending request to sign some data. " +
			"The data to be signed can be of various types, defined by content-type. Clef has done most " +
			"of the work in canonicalizing and making sense of the data, and it's up to the UI to present" +
			"the user with the contents of the `message`"
		sighash, msg := accounts.TextAndHash([]byte("hello world"))
		messages := []*apitypes.NameValueType{{Name: "message", Value: msg, Typ: accounts.MimetypeTextPlain}}

		add("SignDataRequest", desc, &core.SignDataRequest{
			Address:     common.NewMixedcaseAddress(a),
			Meta:        meta,
			ContentType: accounts.MimetypeTextPlain,
			Rawdata:     []byte(msg),
			Messages:    messages,
			Hash:        sighash})
	}
	{ // Sign plain text response
		add("SignDataResponse - approve", "Response to SignDataRequest",
			&core.SignDataResponse{Approved: true})
		add("SignDataResponse - deny", "Response to SignDataRequest",
			&core.SignDataResponse{})
	}
	{ // Sign transaction request
		desc := "SignTxRequest contains information about a pending request to sign a transaction. " +
			"Aside from the transaction itself, there is also a `call_info`-struct. That struct contains " +
			"messages of various types, that the user should be informed of." +
			"\n\n" +
			"As in any request, it's important to consider that the `meta` info also contains untrusted data." +
			"\n\n" +
			"The `transaction` (on input into clef) can have either `data` or `input` -- if both are set, " +
			"they must be identical, otherwise an error is generated. " +
			"However, Clef will always use `data` when passing this struct on (if Clef does otherwise, please file a ticket)"

		data := hexutil.Bytes([]byte{0x01, 0x02, 0x03, 0x04})
		add("SignTxRequest", desc, &core.SignTxRequest{
			Meta: meta,
			Callinfo: []apitypes.ValidationInfo{
				{Typ: "Warning", Message: "Something looks odd, show this message as a warning"},
				{Typ: "Info", Message: "User should see this as well"},
			},
			Transaction: apitypes.SendTxArgs{
				Data:     &data,
				Nonce:    0x1,
				Value:    hexutil.Big(*big.NewInt(6)),
				From:     common.NewMixedcaseAddress(a),
				To:       nil,
				GasPrice: (*hexutil.Big)(big.NewInt(5)),
				Gas:      1000,
				Input:    nil,
			}})
	}
	{ // Sign tx response
		data := hexutil.Bytes([]byte{0x04, 0x03, 0x02, 0x01})
		add("SignTxResponse - approve", "Response to request to sign a transaction. This response needs to contain the `transaction`"+
			", because the UI is free to make modifications to the transaction.",
			&core.SignTxResponse{Approved: true,
				Transaction: apitypes.SendTxArgs{
					Data:     &data,
					Nonce:    0x4,
					Value:    hexutil.Big(*big.NewInt(6)),
					From:     common.NewMixedcaseAddress(a),
					To:       nil,
					GasPrice: (*hexutil.Big)(big.NewInt(5)),
					Gas:      1000,
					Input:    nil,
				}})
		add("SignTxResponse - deny", "Response to SignTxRequest. When denying a request, there's no need to "+
			"provide the transaction in return",
			&core.SignTxResponse{})
	}
	{ // WHen a signed tx is ready to go out
		desc := "SignTransactionResult is used in the call `clef` -> `OnApprovedTx(result)`" +
			"\n\n" +
			"This occurs _after_ successful completion of the entire signing procedure, but right before the signed " +
			"transaction is passed to the external caller. This method (and data) can be used by the UI to signal " +
			"to the user that the transaction was signed, but it is primarily useful for ruleset implementations." +
			"\n\n" +
			"A ruleset that implements a rate limitation needs to know what transactions are sent out to the external " +
			"interface. By hooking into this methods, the ruleset can maintain track of that count." +
			"\n\n" +
			"**OBS:** Note that if an attacker can restore your `clef` data to a previous point in time" +
			" (e.g through a backup), the attacker can reset such windows, even if he/she is unable to decrypt the content. " +
			"\n\n" +
			"The `OnApproved` method cannot be responded to, it's purely informative"

		rlpdata := common.FromHex("0xf85d640101948a8eafb1cf62bfbeb1741769dae1a9dd47996192018026a0716bd90515acb1e68e5ac5867aa11a1e65399c3349d479f5fb698554ebc6f293a04e8a4ebfff434e971e0ef12c5bf3a881b06fd04fc3f8b8a7291fb67a26a1d4ed")
		var tx types.Transaction
		tx.UnmarshalBinary(rlpdata)
		add("OnApproved - SignTransactionResult", desc, &ethapi.SignTransactionResult{Raw: rlpdata, Tx: &tx})
	}
	{ // User input
		add("UserInputRequest", "Sent when clef needs the user to provide data. If 'password' is true, the input field should be treated accordingly (echo-free)",
			&core.UserInputRequest{IsPassword: true, Title: "The title here", Prompt: "The question to ask the user"})
		add("UserInputResponse", "Response to UserInputRequest",
			&core.UserInputResponse{Text: "The textual response from user"})
	}
	{ // List request
		add("ListRequest", "Sent when a request has been made to list addresses. The UI is provided with the "+
			"full `account`s, including local directory names. Note: this information is not passed back to the external caller, "+
			"who only sees the `address`es. ",
			&core.ListRequest{
				Meta: meta,
				Accounts: []accounts.Account{
					{Address: a, URL: accounts.URL{Scheme: "keystore", Path: "/path/to/keyfile/a"}},
					{Address: b, URL: accounts.URL{Scheme: "keystore", Path: "/path/to/keyfile/b"}}},
			})

		add("ListResponse", "Response to list request. The response contains a list of all addresses to show to the caller. "+
			"Note: the UI is free to respond with any address the caller, regardless of whether it exists or not",
			&core.ListResponse{
				Accounts: []accounts.Account{
					{
						Address: common.HexToAddress("0xcowbeef000000cowbeef00000000000000000c0w"),
						URL:     accounts.URL{Path: ".. ignored .."},
					},
					{
						Address: common.MaxAddress,
					},
				}})
	}

	fmt.Println(`## UI Client interface

These data types are defined in the channel between clef and the UI`)
	for _, elem := range output {
		fmt.Println(elem)
	}
	return nil
}
