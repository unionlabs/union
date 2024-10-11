// Copyright 2020 The go-ethereum Authors
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
	"time"

	"github.com/ethereum/go-ethereum/cmd/devp2p/internal/v5test"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/internal/flags"
	"github.com/ethereum/go-ethereum/p2p/discover"
	"github.com/urfave/cli/v2"
)

var (
	discv5Command = &cli.Command{
		Name:  "discv5",
		Usage: "Node Discovery v5 tools",
		Subcommands: []*cli.Command{
			discv5PingCommand,
			discv5ResolveCommand,
			discv5CrawlCommand,
			discv5TestCommand,
			discv5ListenCommand,
		},
	}
	discv5PingCommand = &cli.Command{
		Name:   "ping",
		Usage:  "Sends ping to a node",
		Action: discv5Ping,
		Flags:  discoveryNodeFlags,
	}
	discv5ResolveCommand = &cli.Command{
		Name:   "resolve",
		Usage:  "Finds a node in the DHT",
		Action: discv5Resolve,
		Flags:  discoveryNodeFlags,
	}
	discv5CrawlCommand = &cli.Command{
		Name:   "crawl",
		Usage:  "Updates a nodes.json file with random nodes found in the DHT",
		Action: discv5Crawl,
		Flags: flags.Merge(discoveryNodeFlags, []cli.Flag{
			crawlTimeoutFlag,
		}),
	}
	discv5TestCommand = &cli.Command{
		Name:   "test",
		Usage:  "Runs protocol tests against a node",
		Action: discv5Test,
		Flags: []cli.Flag{
			testPatternFlag,
			testTAPFlag,
			testListen1Flag,
			testListen2Flag,
		},
	}
	discv5ListenCommand = &cli.Command{
		Name:   "listen",
		Usage:  "Runs a node",
		Action: discv5Listen,
		Flags:  discoveryNodeFlags,
	}
)

func discv5Ping(ctx *cli.Context) error {
	n := getNodeArg(ctx)
	disc, _ := startV5(ctx)
	defer disc.Close()

	fmt.Println(disc.Ping(n))
	return nil
}

func discv5Resolve(ctx *cli.Context) error {
	n := getNodeArg(ctx)
	disc, _ := startV5(ctx)
	defer disc.Close()

	fmt.Println(disc.Resolve(n))
	return nil
}

func discv5Crawl(ctx *cli.Context) error {
	if ctx.NArg() < 1 {
		return errors.New("need nodes file as argument")
	}
	nodesFile := ctx.Args().First()
	inputSet := make(nodeSet)
	if common.FileExist(nodesFile) {
		inputSet = loadNodesJSON(nodesFile)
	}

	disc, config := startV5(ctx)
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

// discv5Test runs the protocol test suite.
func discv5Test(ctx *cli.Context) error {
	suite := &v5test.Suite{
		Dest:    getNodeArg(ctx),
		Listen1: ctx.String(testListen1Flag.Name),
		Listen2: ctx.String(testListen2Flag.Name),
	}
	return runTests(ctx, suite.AllTests())
}

func discv5Listen(ctx *cli.Context) error {
	disc, _ := startV5(ctx)
	defer disc.Close()

	fmt.Println(disc.Self())
	select {}
}

// startV5 starts an ephemeral discovery v5 node.
func startV5(ctx *cli.Context) (*discover.UDPv5, discover.Config) {
	ln, config := makeDiscoveryConfig(ctx)
	socket := listen(ctx, ln)
	disc, err := discover.ListenV5(socket, ln, config)
	if err != nil {
		exit(err)
	}
	return disc, config
}
