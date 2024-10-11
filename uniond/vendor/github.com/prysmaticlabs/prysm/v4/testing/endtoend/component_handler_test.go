package endtoend

import (
	"context"
	"fmt"
	"os"
	"strconv"
	"testing"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/components"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/components/eth1"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"golang.org/x/sync/errgroup"
)

type componentHandler struct {
	t                        *testing.T
	cfg                      *e2etypes.E2EConfig
	ctx                      context.Context
	done                     func()
	group                    *errgroup.Group
	keygen                   e2etypes.ComponentRunner
	tracingSink              e2etypes.ComponentRunner
	web3Signer               e2etypes.ComponentRunner
	bootnode                 e2etypes.ComponentRunner
	eth1Miner                e2etypes.ComponentRunner
	builders                 e2etypes.MultipleComponentRunners
	eth1Proxy                e2etypes.MultipleComponentRunners
	eth1Nodes                e2etypes.MultipleComponentRunners
	beaconNodes              e2etypes.MultipleComponentRunners
	validatorNodes           e2etypes.MultipleComponentRunners
	lighthouseBeaconNodes    e2etypes.MultipleComponentRunners
	lighthouseValidatorNodes e2etypes.MultipleComponentRunners
}

func NewComponentHandler(cfg *e2etypes.E2EConfig, t *testing.T) *componentHandler {
	ctx, done := context.WithCancel(context.Background())
	g, ctx := errgroup.WithContext(ctx)

	return &componentHandler{
		ctx:       ctx,
		done:      done,
		group:     g,
		cfg:       cfg,
		t:         t,
		eth1Miner: eth1.NewMiner(),
	}
}

func (c *componentHandler) setup() {
	t, config := c.t, c.cfg
	ctx, g := c.ctx, c.group
	t.Logf("Shard index: %d\n", e2e.TestParams.TestShardIndex)
	t.Logf("Starting time: %s\n", time.Now().String())
	t.Logf("Log Path: %s\n", e2e.TestParams.LogPath)

	multiClientActive := e2e.TestParams.LighthouseBeaconNodeCount > 0
	var keyGen e2etypes.ComponentRunner
	var lighthouseValidatorNodes e2etypes.MultipleComponentRunners
	var lighthouseNodes *components.LighthouseBeaconNodeSet

	tracingSink := components.NewTracingSink(config.TracingSinkEndpoint)
	g.Go(func() error {
		return tracingSink.Start(ctx)
	})
	c.tracingSink = tracingSink

	if multiClientActive {
		keyGen = components.NewKeystoreGenerator()

		// Generate lighthouse keystores.
		g.Go(func() error {
			return keyGen.Start(ctx)
		})
		c.keygen = keyGen
	}

	var web3RemoteSigner *components.Web3RemoteSigner
	if config.UseWeb3RemoteSigner {
		web3RemoteSigner = components.NewWeb3RemoteSigner()
		g.Go(func() error {
			if err := web3RemoteSigner.Start(ctx); err != nil {
				return errors.Wrap(err, "failed to start web3 remote signer")
			}
			return nil
		})
		c.web3Signer = web3RemoteSigner

	}

	// Boot node.
	bootNode := components.NewBootNode()
	g.Go(func() error {
		if err := bootNode.Start(ctx); err != nil {
			return errors.Wrap(err, "failed to start bootnode")
		}
		return nil
	})
	c.bootnode = bootNode

	miner, ok := c.eth1Miner.(*eth1.Miner)
	if !ok {
		g.Go(func() error {
			return errors.New("c.eth1Miner fails type assertion to *eth1.Miner")
		})
		return
	}
	// ETH1 miner.
	g.Go(func() error {
		if err := helpers.ComponentsStarted(ctx, []e2etypes.ComponentRunner{bootNode}); err != nil {
			return errors.Wrap(err, "miner require boot node to run")
		}
		miner.SetBootstrapENR(bootNode.ENR())
		if err := miner.Start(ctx); err != nil {
			return errors.Wrap(err, "failed to start the ETH1 miner")
		}
		return nil
	})

	// ETH1 non-mining nodes.
	eth1Nodes := eth1.NewNodeSet()
	g.Go(func() error {
		if err := helpers.ComponentsStarted(ctx, []e2etypes.ComponentRunner{miner}); err != nil {
			return errors.Wrap(err, "execution nodes require miner to run")
		}
		eth1Nodes.SetMinerENR(miner.ENR())
		if err := eth1Nodes.Start(ctx); err != nil {
			return errors.Wrap(err, "failed to start ETH1 nodes")
		}
		return nil
	})
	c.eth1Nodes = eth1Nodes

	if config.TestCheckpointSync {
		appendDebugEndpoints(config)
	}

	var builders *components.BuilderSet
	var proxies *eth1.ProxySet
	if config.UseBuilder {
		// Builder
		builders = components.NewBuilderSet()
		g.Go(func() error {
			if err := helpers.ComponentsStarted(ctx, []e2etypes.ComponentRunner{eth1Nodes}); err != nil {
				return errors.Wrap(err, "builders require execution nodes to run")
			}
			if err := builders.Start(ctx); err != nil {
				return errors.Wrap(err, "failed to start builders")
			}
			return nil
		})
		c.builders = builders
	} else {
		// Proxies
		proxies = eth1.NewProxySet()
		g.Go(func() error {
			if err := helpers.ComponentsStarted(ctx, []e2etypes.ComponentRunner{eth1Nodes}); err != nil {
				return errors.Wrap(err, "proxies require execution nodes to run")
			}
			if err := proxies.Start(ctx); err != nil {
				return errors.Wrap(err, "failed to start proxies")
			}
			return nil
		})
		c.eth1Proxy = proxies
	}

	// Beacon nodes.
	beaconNodes := components.NewBeaconNodes(config)
	g.Go(func() error {
		wantedComponents := []e2etypes.ComponentRunner{eth1Nodes, bootNode}
		if config.UseBuilder {
			wantedComponents = append(wantedComponents, builders)
		} else {
			wantedComponents = append(wantedComponents, proxies)
		}
		if err := helpers.ComponentsStarted(ctx, wantedComponents); err != nil {
			return errors.Wrap(err, "beacon nodes require proxies, execution and boot node to run")
		}
		beaconNodes.SetENR(bootNode.ENR())
		if err := beaconNodes.Start(ctx); err != nil {
			return errors.Wrap(err, "failed to start beacon nodes")
		}
		return nil
	})
	c.beaconNodes = beaconNodes

	if multiClientActive {
		lighthouseNodes = components.NewLighthouseBeaconNodes(config)
		g.Go(func() error {
			wantedComponents := []e2etypes.ComponentRunner{eth1Nodes, bootNode, beaconNodes}
			if config.UseBuilder {
				wantedComponents = append(wantedComponents, builders)
			} else {
				wantedComponents = append(wantedComponents, proxies)
			}
			if err := helpers.ComponentsStarted(ctx, wantedComponents); err != nil {
				return errors.Wrap(err, "lighthouse beacon nodes require proxies, execution, beacon and boot node to run")
			}
			lighthouseNodes.SetENR(bootNode.ENR())
			if err := lighthouseNodes.Start(ctx); err != nil {
				return errors.Wrap(err, "failed to start lighthouse beacon nodes")
			}
			return nil
		})
		c.lighthouseBeaconNodes = lighthouseNodes
	}
	// Validator nodes.
	validatorNodes := components.NewValidatorNodeSet(config)
	g.Go(func() error {
		comps := []e2etypes.ComponentRunner{beaconNodes}
		if config.UseWeb3RemoteSigner {
			comps = append(comps, web3RemoteSigner)
		}
		if err := helpers.ComponentsStarted(ctx, comps); err != nil {
			return errors.Wrap(err, "validator nodes require components to run")
		}
		if err := validatorNodes.Start(ctx); err != nil {
			return errors.Wrap(err, "failed to start validator nodes")
		}
		return nil
	})
	c.validatorNodes = validatorNodes

	if multiClientActive {
		// Lighthouse Validator nodes.
		lighthouseValidatorNodes = components.NewLighthouseValidatorNodeSet(config)
		g.Go(func() error {
			if err := helpers.ComponentsStarted(ctx, []e2etypes.ComponentRunner{keyGen, lighthouseNodes}); err != nil {
				return errors.Wrap(err, "lighthouse validator nodes require lighthouse beacon nodes to run")
			}
			if err := lighthouseValidatorNodes.Start(ctx); err != nil {
				return errors.Wrap(err, "failed to start lighthouse validator nodes")
			}
			return nil
		})
		c.lighthouseValidatorNodes = lighthouseValidatorNodes
	}
	c.group = g
}

func (c *componentHandler) required() []e2etypes.ComponentRunner {
	multiClientActive := e2e.TestParams.LighthouseBeaconNodeCount > 0
	requiredComponents := []e2etypes.ComponentRunner{
		c.tracingSink, c.eth1Nodes, c.bootnode, c.beaconNodes, c.validatorNodes,
	}
	if c.cfg.UseBuilder {
		requiredComponents = append(requiredComponents, c.builders)
	} else {
		requiredComponents = append(requiredComponents, c.eth1Proxy)
	}
	if multiClientActive {
		requiredComponents = append(requiredComponents, []e2etypes.ComponentRunner{c.keygen, c.lighthouseBeaconNodes, c.lighthouseValidatorNodes}...)
	}
	return requiredComponents
}

func (c *componentHandler) printPIDs(logger func(string, ...interface{})) {
	msg := "\nPID of components. Attach a debugger... if you dare!\n\n"

	msg += "This test PID: " + strconv.Itoa(os.Getpid()) + " (parent=" + strconv.Itoa(os.Getppid()) + ")\n"

	// Beacon chain nodes
	msg += fmt.Sprintf("Beacon chain nodes: %v\n", PIDsFromMultiComponentRunner(c.beaconNodes))
	// Validator nodes
	msg += fmt.Sprintf("Validators: %v\n", PIDsFromMultiComponentRunner(c.validatorNodes))
	// ETH1 nodes
	msg += fmt.Sprintf("ETH1 nodes: %v\n", PIDsFromMultiComponentRunner(c.eth1Nodes))

	logger(msg)
}

func PIDsFromMultiComponentRunner(runner e2etypes.MultipleComponentRunners) []int {
	var pids []int

	for i := 0; true; i++ {
		c, err := runner.ComponentAtIndex(i)
		if c == nil || err != nil {
			break
		}
		p := c.UnderlyingProcess()
		if p != nil {
			pids = append(pids, p.Pid)
		}
	}
	return pids
}

func appendDebugEndpoints(cfg *e2etypes.E2EConfig) {
	debug := []string{
		"--enable-debug-rpc-endpoints",
	}
	cfg.BeaconFlags = append(cfg.BeaconFlags, debug...)
}
