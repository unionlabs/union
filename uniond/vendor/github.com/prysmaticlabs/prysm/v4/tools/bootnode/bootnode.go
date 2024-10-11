/**
 * Bootnode
 *
 * A node which implements the DiscoveryV5 protocol for peer
 * discovery. The purpose of this service is to provide a starting point for
 * newly connected services to find other peers outside of their network.
 *
 * Usage: Run bootnode --help for flag options.
 */
package main

import (
	"context"
	"crypto/ecdsa"
	"crypto/rand"
	"encoding/hex"
	"flag"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"time"

	gcrypto "github.com/ethereum/go-ethereum/crypto"
	gethlog "github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/discover"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/crypto"
	"github.com/pkg/errors"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/async"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ecdsaprysm "github.com/prysmaticlabs/prysm/v4/crypto/ecdsa"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/logs"
	"github.com/prysmaticlabs/prysm/v4/network"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	_ "github.com/prysmaticlabs/prysm/v4/runtime/maxprocs"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/sirupsen/logrus"
)

var (
	debug                 = flag.Bool("debug", false, "Enable debug logging")
	logFileName           = flag.String("log-file", "", "Specify log filename, relative or absolute")
	privateKey            = flag.String("private", "", "Private key to use for peer ID")
	discv5port            = flag.Int("discv5-port", 4000, "Port to listen for discv5 connections")
	metricsPort           = flag.Int("metrics-port", 5000, "Port to listen for connections")
	externalIP            = flag.String("external-ip", "", "External IP for the bootnode")
	forkVersion           = flag.String("fork-version", "", "Fork Version that the bootnode uses")
	genesisValidatorsRoot = flag.String("genesis-root", "", "Genesis Validators Root the beacon node uses")
	seedNode              = flag.String("seed-node", "", "External node to connect to")
	log                   = logrus.WithField("prefix", "bootnode")
	discv5PeersCount      = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "bootstrap_node_discv5_peers",
		Help: "The current number of discv5 peers of the bootstrap node",
	})
)

type handler struct {
	listener *discover.UDPv5
}

func main() {
	flag.Parse()

	if *logFileName != "" {
		if err := logs.ConfigurePersistentLogging(*logFileName); err != nil {
			log.WithError(err).Error("Failed to configuring logging to disk.")
		}
	}

	fmt.Printf("Starting bootnode. Version: %s\n", version.Version())

	if *debug {
		logrus.SetLevel(logrus.DebugLevel)

		// Geth specific logging.
		glogger := gethlog.NewGlogHandler(gethlog.StreamHandler(os.Stderr, gethlog.TerminalFormat(false)))
		glogger.Verbosity(gethlog.LvlTrace)
		gethlog.Root().SetHandler(glogger)

		log.Debug("Debug logging enabled.")
	}
	privKey := extractPrivateKey()
	cfg := discover.Config{
		PrivateKey: privKey,
	}
	if *seedNode != "" {
		log.Debugf("Adding seed node %s", *seedNode)
		node, err := enode.Parse(enode.ValidSchemes, *seedNode)
		if err != nil {
			log.Fatal(err)
		}
		cfg.Bootnodes = []*enode.Node{node}
	}
	ipAddr, err := network.ExternalIP()
	if err != nil {
		log.Fatal(err)
	}
	listener := createListener(ipAddr, *discv5port, cfg)

	node := listener.Self()
	log.Infof("Running bootnode: %s", node.String())

	handler := &handler{
		listener: listener,
	}
	mux := http.NewServeMux()
	mux.HandleFunc("/p2p", handler.httpHandler)

	srv := &http.Server{
		Addr:              fmt.Sprintf(":%d", *metricsPort),
		ReadHeaderTimeout: 3 * time.Second,
		Handler:           mux,
	}

	if err := srv.ListenAndServe(); err != nil {
		log.WithError(err).Fatal("Failed to start server")
	}

	// Update metrics once per slot.
	slotDuration := time.Duration(params.BeaconConfig().SecondsPerSlot)
	async.RunEvery(context.Background(), slotDuration*time.Second, func() {
		updateMetrics(listener)
	})

	select {}
}

func createListener(ipAddr string, port int, cfg discover.Config) *discover.UDPv5 {
	ip := net.ParseIP(ipAddr)
	if ip.To4() == nil {
		log.Fatalf("IPV4 address not provided instead %s was provided", ipAddr)
	}
	var bindIP net.IP
	var networkVersion string
	switch {
	case ip.To16() != nil && ip.To4() == nil:
		bindIP = net.IPv6zero
		networkVersion = "udp6"
	case ip.To4() != nil:
		bindIP = net.IPv4zero
		networkVersion = "udp4"
	default:
		log.Fatalf("Valid ip address not provided instead %s was provided", ipAddr)
	}
	udpAddr := &net.UDPAddr{
		IP:   bindIP,
		Port: port,
	}
	conn, err := net.ListenUDP(networkVersion, udpAddr)
	if err != nil {
		log.Fatal(err)
	}
	localNode, err := createLocalNode(cfg.PrivateKey, ip, port)
	if err != nil {
		log.Fatal(err)
	}

	net, err := discover.ListenV5(conn, localNode, cfg)
	if err != nil {
		log.Fatal(err)
	}
	return net
}

func (h *handler) httpHandler(w http.ResponseWriter, _ *http.Request) {
	w.WriteHeader(http.StatusOK)
	write := func(w io.Writer, b []byte) {
		if _, err := w.Write(b); err != nil {
			log.WithError(err).Error("Failed to write to http response")
		}
	}
	allNodes := h.listener.AllNodes()
	write(w, []byte("Nodes stored in the table:\n"))
	for i, n := range allNodes {
		write(w, []byte(fmt.Sprintf("Node %d\n", i)))
		write(w, []byte(n.String()+"\n"))
		write(w, []byte("Node ID: "+n.ID().String()+"\n"))
		write(w, []byte("IP: "+n.IP().String()+"\n"))
		write(w, []byte(fmt.Sprintf("UDP Port: %d", n.UDP())+"\n"))
		write(w, []byte(fmt.Sprintf("TCP Port: %d", n.TCP())+"\n\n"))
	}
}

func createLocalNode(privKey *ecdsa.PrivateKey, ipAddr net.IP, port int) (*enode.LocalNode, error) {
	db, err := enode.OpenDB("")
	if err != nil {
		return nil, errors.Wrap(err, "Could not open node's peer database")
	}
	external := net.ParseIP(*externalIP)
	if *externalIP == "" {
		external = ipAddr
	}
	fVersion := params.BeaconConfig().GenesisForkVersion
	if *forkVersion != "" {
		fVersion, err = hex.DecodeString(*forkVersion)
		if err != nil {
			return nil, errors.Wrap(err, "Could not retrieve fork version")
		}
		if len(fVersion) != 4 {
			return nil, errors.Errorf("Invalid fork version size expected %d but got %d", 4, len(fVersion))
		}
	}
	genRoot := params.BeaconConfig().ZeroHash
	if *genesisValidatorsRoot != "" {
		retRoot, err := hex.DecodeString(*genesisValidatorsRoot)
		if err != nil {
			return nil, errors.Wrap(err, "Could not retrieve genesis validators root")
		}
		if len(retRoot) != 32 {
			return nil, errors.Errorf("Invalid root size, expected 32 but got %d", len(retRoot))
		}
		genRoot = bytesutil.ToBytes32(retRoot)
	}
	digest, err := signing.ComputeForkDigest(fVersion, genRoot[:])
	if err != nil {
		return nil, errors.Wrap(err, "Could not compute fork digest")
	}

	forkID := &pb.ENRForkID{
		CurrentForkDigest: digest[:],
		NextForkVersion:   fVersion,
		NextForkEpoch:     params.BeaconConfig().FarFutureEpoch,
	}
	forkEntry, err := forkID.MarshalSSZ()
	if err != nil {
		return nil, errors.Wrap(err, "Could not marshal fork id")
	}

	localNode := enode.NewLocalNode(db, privKey)
	localNode.Set(enr.WithEntry("eth2", forkEntry))
	localNode.Set(enr.WithEntry("attnets", bitfield.NewBitvector64()))
	localNode.SetFallbackIP(external)
	localNode.SetFallbackUDP(port)

	return localNode, nil
}

func extractPrivateKey() *ecdsa.PrivateKey {
	var privKey *ecdsa.PrivateKey
	if *privateKey != "" {
		dst, err := hex.DecodeString(*privateKey)
		if err != nil {
			panic(err)
		}
		unmarshalledKey, err := crypto.UnmarshalSecp256k1PrivateKey(dst)
		if err != nil {
			panic(err)
		}

		privKey, err = ecdsaprysm.ConvertFromInterfacePrivKey(unmarshalledKey)
		if err != nil {
			panic(err)
		}
	} else {
		privInterfaceKey, _, err := crypto.GenerateSecp256k1Key(rand.Reader)
		if err != nil {
			panic(err)
		}
		privKey, err = ecdsaprysm.ConvertFromInterfacePrivKey(privInterfaceKey)
		if err != nil {
			panic(err)
		}
		log.Warning("No private key was provided. Using default/random private key")
		b, err := privInterfaceKey.Raw()
		if err != nil {
			panic(err)
		}
		log.Debugf("Private key %x", b)
	}
	privKey.Curve = gcrypto.S256()

	return privKey
}

func updateMetrics(listener *discover.UDPv5) {
	if listener != nil {
		discv5PeersCount.Set(float64(len(listener.AllNodes())))
	}
}
