// Copyright 2016 The go-ethereum Authors
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
	"fmt"
	"strings"

	"github.com/ethereum/go-ethereum/cmd/utils"
	"github.com/ethereum/go-ethereum/console"
	"github.com/ethereum/go-ethereum/internal/flags"
	"github.com/urfave/cli/v2"
)

var (
	consoleFlags = []cli.Flag{utils.JSpathFlag, utils.ExecFlag, utils.PreloadJSFlag}

	consoleCommand = &cli.Command{
		Action: localConsole,
		Name:   "console",
		Usage:  "Start an interactive JavaScript environment",
		Flags:  flags.Merge(nodeFlags, rpcFlags, consoleFlags),
		Description: `
The Geth console is an interactive shell for the JavaScript runtime environment
which exposes a node admin interface as well as the Ðapp JavaScript API.
See https://geth.ethereum.org/docs/interacting-with-geth/javascript-console.`,
	}

	attachCommand = &cli.Command{
		Action:    remoteConsole,
		Name:      "attach",
		Usage:     "Start an interactive JavaScript environment (connect to node)",
		ArgsUsage: "[endpoint]",
		Flags:     flags.Merge([]cli.Flag{utils.DataDirFlag, utils.HttpHeaderFlag}, consoleFlags),
		Description: `
The Geth console is an interactive shell for the JavaScript runtime environment
which exposes a node admin interface as well as the Ðapp JavaScript API.
See https://geth.ethereum.org/docs/interacting-with-geth/javascript-console.
This command allows to open a console on a running geth node.`,
	}

	javascriptCommand = &cli.Command{
		Action:    ephemeralConsole,
		Name:      "js",
		Usage:     "(DEPRECATED) Execute the specified JavaScript files",
		ArgsUsage: "<jsfile> [jsfile...]",
		Flags:     flags.Merge(nodeFlags, consoleFlags),
		Description: `
The JavaScript VM exposes a node admin interface as well as the Ðapp
JavaScript API. See https://geth.ethereum.org/docs/interacting-with-geth/javascript-console`,
	}
)

// localConsole starts a new geth node, attaching a JavaScript console to it at the
// same time.
func localConsole(ctx *cli.Context) error {
	// Create and start the node based on the CLI flags
	prepare(ctx)
	stack, backend := makeFullNode(ctx)
	startNode(ctx, stack, backend, true)
	defer stack.Close()

	// Attach to the newly started node and create the JavaScript console.
	client := stack.Attach()
	config := console.Config{
		DataDir: utils.MakeDataDir(ctx),
		DocRoot: ctx.String(utils.JSpathFlag.Name),
		Client:  client,
		Preload: utils.MakeConsolePreloads(ctx),
	}
	console, err := console.New(config)
	if err != nil {
		return fmt.Errorf("failed to start the JavaScript console: %v", err)
	}
	defer console.Stop(false)

	// If only a short execution was requested, evaluate and return.
	if script := ctx.String(utils.ExecFlag.Name); script != "" {
		console.Evaluate(script)
		return nil
	}

	// Track node shutdown and stop the console when it goes down.
	// This happens when SIGTERM is sent to the process.
	go func() {
		stack.Wait()
		console.StopInteractive()
	}()

	// Print the welcome screen and enter interactive mode.
	console.Welcome()
	console.Interactive()
	return nil
}

// remoteConsole will connect to a remote geth instance, attaching a JavaScript
// console to it.
func remoteConsole(ctx *cli.Context) error {
	if ctx.Args().Len() > 1 {
		utils.Fatalf("invalid command-line: too many arguments")
	}
	endpoint := ctx.Args().First()
	if endpoint == "" {
		cfg := defaultNodeConfig()
		utils.SetDataDir(ctx, &cfg)
		endpoint = cfg.IPCEndpoint()
	}
	client, err := utils.DialRPCWithHeaders(endpoint, ctx.StringSlice(utils.HttpHeaderFlag.Name))
	if err != nil {
		utils.Fatalf("Unable to attach to remote geth: %v", err)
	}
	config := console.Config{
		DataDir: utils.MakeDataDir(ctx),
		DocRoot: ctx.String(utils.JSpathFlag.Name),
		Client:  client,
		Preload: utils.MakeConsolePreloads(ctx),
	}
	console, err := console.New(config)
	if err != nil {
		utils.Fatalf("Failed to start the JavaScript console: %v", err)
	}
	defer console.Stop(false)

	if script := ctx.String(utils.ExecFlag.Name); script != "" {
		console.Evaluate(script)
		return nil
	}

	// Otherwise print the welcome screen and enter interactive mode
	console.Welcome()
	console.Interactive()
	return nil
}

// ephemeralConsole starts a new geth node, attaches an ephemeral JavaScript
// console to it, executes each of the files specified as arguments and tears
// everything down.
func ephemeralConsole(ctx *cli.Context) error {
	var b strings.Builder
	for _, file := range ctx.Args().Slice() {
		b.Write([]byte(fmt.Sprintf("loadScript('%s');", file)))
	}
	utils.Fatalf(`The "js" command is deprecated. Please use the following instead:
geth --exec "%s" console`, b.String())
	return nil
}
