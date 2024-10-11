// This binary is a simple rest API endpoint to calculate
// the ENR value of a node given its private key,ip address and port.
package main

import (
	"encoding/hex"
	"flag"
	"net"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/crypto"
	ecdsaprysm "github.com/prysmaticlabs/prysm/v4/crypto/ecdsa"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	_ "github.com/prysmaticlabs/prysm/v4/runtime/maxprocs"
	log "github.com/sirupsen/logrus"
)

var (
	privateKey = flag.String("private", "", "Hex encoded Private key to use for calculation of ENR")
	udpPort    = flag.Int("udp-port", 0, "UDP Port to use for calculation of ENR")
	tcpPort    = flag.Int("tcp-port", 0, "TCP Port to use for calculation of ENR")
	ipAddr     = flag.String("ipAddress", "", "IP to use in calculation of ENR")
	outfile    = flag.String("out", "", "Filepath to write ENR")
)

func main() {
	flag.Parse()

	if *privateKey == "" {
		log.Fatal("No private key given")
	}
	dst, err := hex.DecodeString(*privateKey)
	if err != nil {
		panic(err)
	}
	unmarshalledKey, err := crypto.UnmarshalSecp256k1PrivateKey(dst)
	if err != nil {
		panic(err)
	}
	ecdsaPrivKey, err := ecdsaprysm.ConvertFromInterfacePrivKey(unmarshalledKey)
	if err != nil {
		panic(err)
	}

	if net.ParseIP(*ipAddr).To4() == nil {
		log.WithField("address", *ipAddr).Fatal("Invalid ipv4 address given")
	}

	if *udpPort == 0 {
		log.WithField("port", *udpPort).Fatal("Invalid udp port given")
		return
	}

	db, err := enode.OpenDB("")
	if err != nil {
		log.WithError(err).Fatal("Could not open node's peer database")
		return
	}
	defer db.Close()

	localNode := enode.NewLocalNode(db, ecdsaPrivKey)
	ipEntry := enr.IP(net.ParseIP(*ipAddr))
	udpEntry := enr.UDP(*udpPort)
	localNode.Set(ipEntry)
	localNode.Set(udpEntry)
	if *tcpPort != 0 {
		tcpEntry := enr.TCP(*tcpPort)
		localNode.Set(tcpEntry)
	}
	log.Info(localNode.Node().String())

	if *outfile != "" {
		err := file.WriteFile(*outfile, []byte(localNode.Node().String()))
		if err != nil {
			panic(err)
		}
		log.Infof("Wrote to %s", *outfile)
	}
}
