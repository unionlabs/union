// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"strconv"

	"github.com/hashicorp/go-plugin"
	"github.com/hashicorp/go-plugin/examples/bidirectional/shared"
)

type addHelper struct{}

func (*addHelper) Sum(a, b int64) (int64, error) {
	return a + b, nil
}

func main() {
	// We don't want to see the plugin logs.
	log.SetOutput(ioutil.Discard)

	// We're a host. Start by launching the plugin process.
	client := plugin.NewClient(&plugin.ClientConfig{
		HandshakeConfig: shared.Handshake,
		Plugins:         shared.PluginMap,
		Cmd:             exec.Command("sh", "-c", os.Getenv("COUNTER_PLUGIN")),
		AllowedProtocols: []plugin.Protocol{
			plugin.ProtocolNetRPC, plugin.ProtocolGRPC},
	})
	defer client.Kill()

	// Connect via RPC
	rpcClient, err := client.Client()
	if err != nil {
		fmt.Println("Error:", err.Error())
		os.Exit(1)
	}

	// Request the plugin
	raw, err := rpcClient.Dispense("counter")
	if err != nil {
		fmt.Println("Error:", err.Error())
		os.Exit(1)
	}

	// We should have a Counter store now! This feels like a normal interface
	// implementation but is in fact over an RPC connection.
	counter := raw.(shared.Counter)

	os.Args = os.Args[1:]
	switch os.Args[0] {
	case "get":
		result, err := counter.Get(os.Args[1])
		if err != nil {
			fmt.Println("Error:", err.Error())
			os.Exit(1)
		}

		fmt.Println(result)

	case "put":
		i, err := strconv.Atoi(os.Args[2])
		if err != nil {
			fmt.Println("Error:", err.Error())
			os.Exit(1)
		}

		err = counter.Put(os.Args[1], int64(i), &addHelper{})
		if err != nil {
			fmt.Println("Error:", err.Error())
			os.Exit(1)
		}

	default:
		fmt.Println("Please only use 'get' or 'put'")
		os.Exit(1)
	}
}
