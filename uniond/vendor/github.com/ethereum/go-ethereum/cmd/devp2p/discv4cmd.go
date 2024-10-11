// Copyright 2019 The go-ethereum Authors
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
	"errors"
	"fmt"
	"net"
	"strconv"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/cmd/devp2p/internal/v4test"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/internal/flags"
	"github.com/ethereum/go-ethereum/p2p/discover"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/params"
	"github.com/urfave/cli/v2"
)

var (
	discv4Command = &cli.Command{
		Name:  "discv4",
		Usage: "Node Discovery v4 tools",
		Subcommands: []*cli.Command{
			discv4PingCommand,
			discv4RequestRecordCommand,
			discv4ResolveCommand,
			discv4ResolveJSONCommand,
			discv4CrawlCommand,
			discv4TestCommand,
		},
	}
	discv4PingCommand = &cli.Command{
		Name:      "ping",
		Usage:     "Sends ping to a node",
		Action:    discv4Ping,
		ArgsUsage: "<node>",
		Flags:     discoveryNodeFlags,
	}
	discv4RequestRecordCommand = &cli.Command{
		Name:      "requestenr",
		Usage:     "Requests a node record using EIP-868 enrRequest",
		Action:    discv4RequestRecord,
		ArgsUsage: "<node>",
		Flags:     discoveryNodeFlags,
	}
	discv4ResolveCommand = &cli.Command{
		Name:      "resolve",
		Usage:     "Finds a node in the DHT",
		Action:    discv4Resolve,
		ArgsUsage: "<node>",
		Flags:     discoveryNodeFlags,
	}
	discv4ResolveJSONCommand = &cli.Command{
		Name:      "resolve-json",
		Usage:     "Re-resolves nodes in a nodes.json file",
		Action:    discv4ResolveJSON,
		Flags:     discoveryNodeFlags,
		ArgsUsage: "<nodes.json file>",
	}
	discv4CrawlCommand = &cli.Command{
		Name:   "crawl",
		Usage:  "Updates a nodes.json file with random nodes found in the DHT",
		Action: discv4Crawl,
		Flags:  flags.Merge(discoveryNodeFlags, []cli.Flag{crawlTimeoutFlag, crawlParallelismFlag}),
	}
	discv4TestCommand = &cli.Command{
		Name:   "test",
		Usage:  "Runs tests against a node",
		Action: discv4Test,
		Flags: []cli.Flag{
			remoteEnodeFlag,
			testPatternFlag,
			testTAPFlag,
			testListen1Flag,
			testListen2Flag,
		},
	}
)

var (
	bootnodesFlag = &cli.StringFlag{
		Name:  "bootnodes",
		Usage: "Comma separated nodes used for bootstrapping",
	}
	nodekeyFlag = &cli.StringFlag{
		Name:  "nodekey",
		Usage: "Hex-encoded node key",
	}
	nodedbFlag = &cli.StringFlag{
		Name:  "nodedb",
		Usage: "Nodes database location",
	}
	listenAddrFlag = &cli.StringFlag{
		Name:  "addr",
		Usage: "Listening address",
	}
	extAddrFlag = &cli.StringFlag{
		Name:  "extaddr",
		Usage: "UDP endpoint announced in ENR. You can provide a bare IP address or IP:port as the value of this flag.",
	}
	crawlTimeoutFlag = &cli.DurationFlag{
		Name:  "timeout",
		Usage: "Time limit for the crawl.",
		Value: 30 * time.Minute,
	}
	crawlParallelismFlag = &cli.IntFlag{
		Name:  "parallel",
		Usage: "How many parallel discoveries to attempt.",
		Value: 16,
	}
	remoteEnodeFlag = &cli.StringFlag{
		Name:    "remote",
		Usage:   "Enode of the remote node under test",
		EnvVars: []string{"REMOTE_ENODE"},
	}
)

var discoveryNodeFlags = []cli.Flag{
	bootnodesFlag,
	nodekeyFlag,
	nodedbFlag,
	listenAddrFlag,
	extAddrFlag,
}

func discv4Ping(ctx *cli.Context) error {
	n := getNodeArg(ctx)
	disc, _ := startV4(ctx)
	defer disc.Close()

	start := time.Now()
	if err := disc.Ping(n); err != nil {
		return fmt.Errorf("node didn't respond: %v", err)
	}
	fmt.Printf("node responded to ping (RTT %v).\n", time.Since(start))
	return nil
}

func discv4RequestRecord(ctx *cli.Context) error {
	n := getNodeArg(ctx)
	disc, _ := startV4(ctx)
	defer disc.Close()

	respN, err := disc.RequestENR(n)
	if err != nil {
		return fmt.Errorf("can't retrieve record: %v", err)
	}
	fmt.Println(respN.String())
	return nil
}

func discv4Resolve(ctx *cli.Context) error {
	n := getNodeArg(ctx)
	disc, _ := startV4(ctx)
	defer disc.Close()

	fmt.Println(disc.Resolve(n).String())
	return nil
}

func discv4ResolveJSON(ctx *cli.Context) error {
	if ctx.NArg() < 1 {
		return errors.New("need nodes file as argument")
	}
	nodesFile := ctx.Args().Get(0)
	inputSet := make(nodeSet)
	if common.FileExist(nodesFile) {
		inputSet = loadNodesJSON(nodesFile)
	}

	// Add extra nodes from command line arguments.
	var nodeargs []*enode.Node
	for i := 1; i < ctx.NArg(); i++ {
		n, err := parseNode(ctx.Args().Get(i))
		if err != nil {
			exit(err)
		}
		nodeargs = append(nodeargs, n)
	}

	disc, config := startV4(ctx)
	defer disc.Close()

	c, err := newCrawler(inputSet, config.Bootnodes, disc, enode.IterNodes(nodeargs))
	if err != nil {
		return err
	}
	c.revalidateInterval = 0
	output := c.run(0, 1)
	writeNodesJSON(nodesFile, output)
	return nil
}

func discv4Crawl(ctx *cli.Context) error {
	if ctx.NArg() < 1 {
		return errors.New("need nodes file as argument")
	}
	nodesFile := ctx.Args().First()
	inputSet := make(nodeSet)
	if common.FileExist(nodesFile) {
		inputSet = loadNodesJSON(nodesFile)
	}

	disc, config := startV4(ctx)
	defer disc.Close()

	c, err := newCrawler(inputSet, config.Bootnodes, disc, disc.RandomNodes())
	if err != nil {
		return err
	}
	c.revalidateInterval = 10 * time.Minute
	output := c.run(ctx.Duration(crawlTimeoutFlag.Name), ctx.Int(crawlParallelismFlag.Name))
	writeNodesJSON(nodesFile, output)
	return nil
}

// discv4Test runs the protocol test suite.
func discv4Test(ctx *cli.Context) error {
	// Configure test package globals.
	if !ctx.IsSet(remoteEnodeFlag.Name) {
		return fmt.Errorf("Missing -%v", remoteEnodeFlag.Name)
	}
	v4test.Remote = ctx.String(remoteEnodeFlag.Name)
	v4test.Listen1 = ctx.String(testListen1Flag.Name)
	v4test.Listen2 = ctx.String(testListen2Flag.Name)
	return runTests(ctx, v4test.AllTests)
}

// startV4 starts an ephemeral discovery V4 node.
func startV4(ctx *cli.Context) (*discover.UDPv4, discover.Config) {
	ln, config := makeDiscoveryConfig(ctx)
	socket := listen(ctx, ln)
	disc, err := discover.ListenV4(socket, ln, config)
	if err != nil {
		exit(err)
	}
	return disc, config
}

func makeDiscoveryConfig(ctx *cli.Context) (*enode.LocalNode, discover.Config) {
	var cfg discover.Config

	if ctx.IsSet(nodekeyFlag.Name) {
		key, err := crypto.HexToECDSA(ctx.String(nodekeyFlag.Name))
		if err != nil {
			exit(fmt.Errorf("-%s: %v", nodekeyFlag.Name, err))
		}
		cfg.PrivateKey = key
	} else {
		cfg.PrivateKey, _ = crypto.GenerateKey()
	}

	if commandHasFlag(ctx, bootnodesFlag) {
		bn, err := parseBootnodes(ctx)
		if err != nil {
			exit(err)
		}
		cfg.Bootnodes = bn
	}

	dbpath := ctx.String(nodedbFlag.Name)
	db, err := enode.OpenDB(dbpath)
	if err != nil {
		exit(err)
	}
	ln := enode.NewLocalNode(db, cfg.PrivateKey)
	return ln, cfg
}

func parseExtAddr(spec string) (ip net.IP, port int, ok bool) {
	ip = net.ParseIP(spec)
	if ip != nil {
		return ip, 0, true
	}
	host, portstr, err := net.SplitHostPort(spec)
	if err != nil {
		return nil, 0, false
	}
	ip = net.ParseIP(host)
	if ip == nil {
		return nil, 0, false
	}
	port, err = strconv.Atoi(portstr)
	if err != nil {
		return nil, 0, false
	}
	return ip, port, true
}

func listen(ctx *cli.Context, ln *enode.LocalNode) *net.UDPConn {
	addr := ctx.String(listenAddrFlag.Name)
	if addr == "" {
		addr = "0.0.0.0:0"
	}
	socket, err := net.ListenPacket("udp4", addr)
	if err != nil {
		exit(err)
	}

	// Configure UDP endpoint in ENR from listener address.
	usocket := socket.(*net.UDPConn)
	uaddr := socket.LocalAddr().(*net.UDPAddr)
	if uaddr.IP.IsUnspecified() {
		ln.SetFallbackIP(net.IP{127, 0, 0, 1})
	} else {
		ln.SetFallbackIP(uaddr.IP)
	}
	ln.SetFallbackUDP(uaddr.Port)

	// If an ENR endpoint is set explicitly on the command-line, override
	// the information from the listening address. Note this is careful not
	// to set the UDP port if the external address doesn't have it.
	extAddr := ctx.String(extAddrFlag.Name)
	if extAddr != "" {
		ip, port, ok := parseExtAddr(extAddr)
		if !ok {
			exit(fmt.Errorf("-%s: invalid external address %q", extAddrFlag.Name, extAddr))
		}
		ln.SetStaticIP(ip)
		if port != 0 {
			ln.SetFallbackUDP(port)
		}
	}

	return usocket
}

func parseBootnodes(ctx *cli.Context) ([]*enode.Node, error) {
	s := params.MainnetBootnodes
	if ctx.IsSet(bootnodesFlag.Name) {
		input := ctx.String(bootnodesFlag.Name)
		if input == "" {
			return nil, nil
		}
		s = strings.Split(input, ",")
	}
	nodes := make([]*enode.Node, len(s))
	var err error
	for i, record := range s {
		nodes[i], err = parseNode(record)
		if err != nil {
			return nil, fmt.Errorf("invalid bootstrap node: %v", err)
		}
	}
	return nodes, nil
}
