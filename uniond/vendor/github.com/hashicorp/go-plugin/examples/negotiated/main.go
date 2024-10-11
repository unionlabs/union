// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"

	"github.com/hashicorp/go-plugin"
	"github.com/hashicorp/go-plugin/examples/grpc/shared"
)

func main() {
	// We don't want to see the plugin logs.
	log.SetOutput(ioutil.Discard)

	plugins := map[int]plugin.PluginSet{}

	// Both version can be supported, but switch the implementation to
	// demonstrate version negoation.
	switch os.Getenv("KV_PROTO") {
	case "netrpc":
		plugins[2] = plugin.PluginSet{
			"kv": &shared.KVPlugin{},
		}
	case "grpc":
		plugins[3] = plugin.PluginSet{
			"kv": &shared.KVGRPCPlugin{},
		}
	default:
		fmt.Println("must set KV_PROTO to netrpc or grpc")
		os.Exit(1)
	}

	// We're a host. Start by launching the plugin process.
	client := plugin.NewClient(&plugin.ClientConfig{
		HandshakeConfig:  shared.Handshake,
		VersionedPlugins: plugins,
		Cmd:              exec.Command("./kv-plugin"),
		AllowedProtocols: []plugin.Protocol{
			plugin.ProtocolNetRPC, plugin.ProtocolGRPC},
	})
	defer client.Kill()

	rpcClient, err := client.Client()
	if err != nil {
		fmt.Println("Error:", err.Error())
		os.Exit(1)
	}

	// Request the plugin
	raw, err := rpcClient.Dispense("kv")
	if err != nil {
		fmt.Println("Error:", err.Error())
		os.Exit(1)
	}

	// We should have a KV store now! This feels like a normal interface
	// implementation but is in fact over an RPC connection.
	kv := raw.(shared.KV)
	os.Args = os.Args[1:]
	switch os.Args[0] {
	case "get":
		result, err := kv.Get(os.Args[1])
		if err != nil {
			fmt.Println("Error:", err.Error())
			os.Exit(1)
		}

		fmt.Println(string(result))

	case "put":
		err := kv.Put(os.Args[1], []byte(os.Args[2]))
		if err != nil {
			fmt.Println("Error:", err.Error())
			os.Exit(1)
		}

	default:
		fmt.Println("Please only use 'get' or 'put'")
		os.Exit(1)
	}
}
