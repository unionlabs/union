// Copyright 2017 The go-ethereum Authors
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

// p2psim provides a command-line client for a simulation HTTP API.
//
// Here is an example of creating a 2 node network with the first node
// connected to the second:
//
//	$ p2psim node create
//	Created node01
//
//	$ p2psim node start node01
//	Started node01
//
//	$ p2psim node create
//	Created node02
//
//	$ p2psim node start node02
//	Started node02
//
//	$ p2psim node connect node01 node02
//	Connected node01 to node02
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"
	"text/tabwriter"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/internal/flags"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/simulations"
	"github.com/ethereum/go-ethereum/p2p/simulations/adapters"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/urfave/cli/v2"
)

var client *simulations.Client

var (
	// global command flags
	apiFlag = &cli.StringFlag{
		Name:    "api",
		Value:   "http://localhost:8888",
		Usage:   "simulation API URL",
		EnvVars: []string{"P2PSIM_API_URL"},
	}

	// events subcommand flags
	currentFlag = &cli.BoolFlag{
		Name:  "current",
		Usage: "get existing nodes and conns first",
	}
	filterFlag = &cli.StringFlag{
		Name:  "filter",
		Value: "",
		Usage: "message filter",
	}

	// node create subcommand flags
	nameFlag = &cli.StringFlag{
		Name:  "name",
		Value: "",
		Usage: "node name",
	}
	servicesFlag = &cli.StringFlag{
		Name:  "services",
		Value: "",
		Usage: "node services (comma separated)",
	}
	keyFlag = &cli.StringFlag{
		Name:  "key",
		Value: "",
		Usage: "node private key (hex encoded)",
	}

	// node rpc subcommand flags
	subscribeFlag = &cli.BoolFlag{
		Name:  "subscribe",
		Usage: "method is a subscription",
	}
)

func main() {
	app := flags.NewApp("devp2p simulation command-line client")
	app.Flags = []cli.Flag{
		apiFlag,
	}
	app.Before = func(ctx *cli.Context) error {
		client = simulations.NewClient(ctx.String(apiFlag.Name))
		return nil
	}
	app.Commands = []*cli.Command{
		{
			Name:   "show",
			Usage:  "show network information",
			Action: showNetwork,
		},
		{
			Name:   "events",
			Usage:  "stream network events",
			Action: streamNetwork,
			Flags: []cli.Flag{
				currentFlag,
				filterFlag,
			},
		},
		{
			Name:   "snapshot",
			Usage:  "create a network snapshot to stdout",
			Action: createSnapshot,
		},
		{
			Name:   "load",
			Usage:  "load a network snapshot from stdin",
			Action: loadSnapshot,
		},
		{
			Name:   "node",
			Usage:  "manage simulation nodes",
			Action: listNodes,
			Subcommands: []*cli.Command{
				{
					Name:   "list",
					Usage:  "list nodes",
					Action: listNodes,
				},
				{
					Name:   "create",
					Usage:  "create a node",
					Action: createNode,
					Flags: []cli.Flag{
						nameFlag,
						servicesFlag,
						keyFlag,
					},
				},
				{
					Name:      "show",
					ArgsUsage: "<node>",
					Usage:     "show node information",
					Action:    showNode,
				},
				{
					Name:      "start",
					ArgsUsage: "<node>",
					Usage:     "start a node",
					Action:    startNode,
				},
				{
					Name:      "stop",
					ArgsUsage: "<node>",
					Usage:     "stop a node",
					Action:    stopNode,
				},
				{
					Name:      "connect",
					ArgsUsage: "<node> <peer>",
					Usage:     "connect a node to a peer node",
					Action:    connectNode,
				},
				{
					Name:      "disconnect",
					ArgsUsage: "<node> <peer>",
					Usage:     "disconnect a node from a peer node",
					Action:    disconnectNode,
				},
				{
					Name:      "rpc",
					ArgsUsage: "<node> <method> [<args>]",
					Usage:     "call a node RPC method",
					Action:    rpcNode,
					Flags: []cli.Flag{
						subscribeFlag,
					},
				},
			},
		},
	}
	if err := app.Run(os.Args); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func showNetwork(ctx *cli.Context) error {
	if ctx.NArg() != 0 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	network, err := client.GetNetwork()
	if err != nil {
		return err
	}
	w := tabwriter.NewWriter(ctx.App.Writer, 1, 2, 2, ' ', 0)
	defer w.Flush()
	fmt.Fprintf(w, "NODES\t%d\n", len(network.Nodes))
	fmt.Fprintf(w, "CONNS\t%d\n", len(network.Conns))
	return nil
}

func streamNetwork(ctx *cli.Context) error {
	if ctx.NArg() != 0 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	events := make(chan *simulations.Event)
	sub, err := client.SubscribeNetwork(events, simulations.SubscribeOpts{
		Current: ctx.Bool(currentFlag.Name),
		Filter:  ctx.String(filterFlag.Name),
	})
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()
	enc := json.NewEncoder(ctx.App.Writer)
	for {
		select {
		case event := <-events:
			if err := enc.Encode(event); err != nil {
				return err
			}
		case err := <-sub.Err():
			return err
		}
	}
}

func createSnapshot(ctx *cli.Context) error {
	if ctx.NArg() != 0 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	snap, err := client.CreateSnapshot()
	if err != nil {
		return err
	}
	return json.NewEncoder(os.Stdout).Encode(snap)
}

func loadSnapshot(ctx *cli.Context) error {
	if ctx.NArg() != 0 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	snap := &simulations.Snapshot{}
	if err := json.NewDecoder(os.Stdin).Decode(snap); err != nil {
		return err
	}
	return client.LoadSnapshot(snap)
}

func listNodes(ctx *cli.Context) error {
	if ctx.NArg() != 0 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	nodes, err := client.GetNodes()
	if err != nil {
		return err
	}
	w := tabwriter.NewWriter(ctx.App.Writer, 1, 2, 2, ' ', 0)
	defer w.Flush()
	fmt.Fprintf(w, "NAME\tPROTOCOLS\tID\n")
	for _, node := range nodes {
		fmt.Fprintf(w, "%s\t%s\t%s\n", node.Name, strings.Join(protocolList(node), ","), node.ID)
	}
	return nil
}

func protocolList(node *p2p.NodeInfo) []string {
	protos := make([]string, 0, len(node.Protocols))
	for name := range node.Protocols {
		protos = append(protos, name)
	}
	return protos
}

func createNode(ctx *cli.Context) error {
	if ctx.NArg() != 0 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	config := adapters.RandomNodeConfig()
	config.Name = ctx.String(nameFlag.Name)
	if key := ctx.String(keyFlag.Name); key != "" {
		privKey, err := crypto.HexToECDSA(key)
		if err != nil {
			return err
		}
		config.ID = enode.PubkeyToIDV4(&privKey.PublicKey)
		config.PrivateKey = privKey
	}
	if services := ctx.String(servicesFlag.Name); services != "" {
		config.Lifecycles = strings.Split(services, ",")
	}
	node, err := client.CreateNode(config)
	if err != nil {
		return err
	}
	fmt.Fprintln(ctx.App.Writer, "Created", node.Name)
	return nil
}

func showNode(ctx *cli.Context) error {
	if ctx.NArg() != 1 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	nodeName := ctx.Args().First()
	node, err := client.GetNode(nodeName)
	if err != nil {
		return err
	}
	w := tabwriter.NewWriter(ctx.App.Writer, 1, 2, 2, ' ', 0)
	defer w.Flush()
	fmt.Fprintf(w, "NAME\t%s\n", node.Name)
	fmt.Fprintf(w, "PROTOCOLS\t%s\n", strings.Join(protocolList(node), ","))
	fmt.Fprintf(w, "ID\t%s\n", node.ID)
	fmt.Fprintf(w, "ENODE\t%s\n", node.Enode)
	for name, proto := range node.Protocols {
		fmt.Fprintln(w)
		fmt.Fprintf(w, "--- PROTOCOL INFO: %s\n", name)
		fmt.Fprintf(w, "%v\n", proto)
		fmt.Fprintf(w, "---\n")
	}
	return nil
}

func startNode(ctx *cli.Context) error {
	if ctx.NArg() != 1 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	nodeName := ctx.Args().First()
	if err := client.StartNode(nodeName); err != nil {
		return err
	}
	fmt.Fprintln(ctx.App.Writer, "Started", nodeName)
	return nil
}

func stopNode(ctx *cli.Context) error {
	if ctx.NArg() != 1 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	nodeName := ctx.Args().First()
	if err := client.StopNode(nodeName); err != nil {
		return err
	}
	fmt.Fprintln(ctx.App.Writer, "Stopped", nodeName)
	return nil
}

func connectNode(ctx *cli.Context) error {
	if ctx.NArg() != 2 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	args := ctx.Args()
	nodeName := args.Get(0)
	peerName := args.Get(1)
	if err := client.ConnectNode(nodeName, peerName); err != nil {
		return err
	}
	fmt.Fprintln(ctx.App.Writer, "Connected", nodeName, "to", peerName)
	return nil
}

func disconnectNode(ctx *cli.Context) error {
	args := ctx.Args()
	if args.Len() != 2 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	nodeName := args.Get(0)
	peerName := args.Get(1)
	if err := client.DisconnectNode(nodeName, peerName); err != nil {
		return err
	}
	fmt.Fprintln(ctx.App.Writer, "Disconnected", nodeName, "from", peerName)
	return nil
}

func rpcNode(ctx *cli.Context) error {
	args := ctx.Args()
	if args.Len() < 2 {
		return cli.ShowCommandHelp(ctx, ctx.Command.Name)
	}
	nodeName := args.Get(0)
	method := args.Get(1)
	rpcClient, err := client.RPCClient(context.Background(), nodeName)
	if err != nil {
		return err
	}
	if ctx.Bool(subscribeFlag.Name) {
		return rpcSubscribe(rpcClient, ctx.App.Writer, method, args.Slice()[3:]...)
	}
	var result interface{}
	params := make([]interface{}, len(args.Slice()[3:]))
	for i, v := range args.Slice()[3:] {
		params[i] = v
	}
	if err := rpcClient.Call(&result, method, params...); err != nil {
		return err
	}
	return json.NewEncoder(ctx.App.Writer).Encode(result)
}

func rpcSubscribe(client *rpc.Client, out io.Writer, method string, args ...string) error {
	parts := strings.SplitN(method, "_", 2)
	namespace := parts[0]
	method = parts[1]
	ch := make(chan interface{})
	subArgs := make([]interface{}, len(args)+1)
	subArgs[0] = method
	for i, v := range args {
		subArgs[i+1] = v
	}
	sub, err := client.Subscribe(context.Background(), namespace, ch, subArgs...)
	if err != nil {
		return err
	}
	defer sub.Unsubscribe()
	enc := json.NewEncoder(out)
	for {
		select {
		case v := <-ch:
			if err := enc.Encode(v); err != nil {
				return err
			}
		case err := <-sub.Err():
			return err
		}
	}
}
