package consensus

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"

	dbm "github.com/cometbft/cometbft-db"

	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/log"
	cmtos "github.com/cometbft/cometbft/libs/os"
	"github.com/cometbft/cometbft/proxy"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
)

const (
	// event bus subscriber
	subscriber = "replay-file"
)

//--------------------------------------------------------
// replay messages interactively or all at once

// replay the wal file
func RunReplayFile(config cfg.BaseConfig, csConfig *cfg.ConsensusConfig, console bool) {
	consensusState := newConsensusStateForReplay(config, csConfig)

	if err := consensusState.ReplayFile(csConfig.WalFile(), console); err != nil {
		cmtos.Exit(fmt.Sprintf("Error during consensus replay: %v", err))
	}
}

// Replay msgs in file or start the console
func (cs *State) ReplayFile(file string, console bool) error {
	if cs.IsRunning() {
		return errors.New("cs is already running, cannot replay")
	}
	if cs.wal != nil {
		return errors.New("cs wal is open, cannot replay")
	}

	cs.startForReplay()

	// ensure all new step events are regenerated as expected

	ctx := context.Background()
	newStepSub, err := cs.eventBus.Subscribe(ctx, subscriber, types.EventQueryNewRoundStep)
	if err != nil {
		return fmt.Errorf("failed to subscribe %s to %v", subscriber, types.EventQueryNewRoundStep)
	}
	defer func() {
		if err := cs.eventBus.Unsubscribe(ctx, subscriber, types.EventQueryNewRoundStep); err != nil {
			cs.Logger.Error("Error unsubscribing to event bus", "err", err)
		}
	}()

	// just open the file for reading, no need to use wal
	fp, err := os.OpenFile(file, os.O_RDONLY, 0o600)
	if err != nil {
		return err
	}

	pb := newPlayback(file, fp, cs, cs.state.Copy())
	defer pb.fp.Close()

	var nextN int // apply N msgs in a row
	var msg *TimedWALMessage
	for {
		if nextN == 0 && console {
			nextN = pb.replayConsoleLoop()
		}

		msg, err = pb.dec.Decode()
		if err == io.EOF {
			return nil
		} else if err != nil {
			return err
		}

		if err := pb.cs.readReplayMessage(msg, newStepSub); err != nil {
			return err
		}

		if nextN > 0 {
			nextN--
		}
		pb.count++
	}
}

//------------------------------------------------
// playback manager

type playback struct {
	cs *State

	fp    *os.File
	dec   *WALDecoder
	count int // how many lines/msgs into the file are we

	// replays can be reset to beginning
	fileName     string   // so we can close/reopen the file
	genesisState sm.State // so the replay session knows where to restart from
}

func newPlayback(fileName string, fp *os.File, cs *State, genState sm.State) *playback {
	return &playback{
		cs:           cs,
		fp:           fp,
		fileName:     fileName,
		genesisState: genState,
		dec:          NewWALDecoder(fp),
	}
}

// go back count steps by resetting the state and running (pb.count - count) steps
func (pb *playback) replayReset(count int, newStepSub types.Subscription) error {
	if err := pb.cs.Stop(); err != nil {
		return err
	}
	pb.cs.Wait()

	newCS := NewState(pb.cs.config, pb.genesisState.Copy(), pb.cs.blockExec,
		pb.cs.blockStore, pb.cs.txNotifier, pb.cs.evpool)
	newCS.SetEventBus(pb.cs.eventBus)
	newCS.startForReplay()

	if err := pb.fp.Close(); err != nil {
		return err
	}
	fp, err := os.OpenFile(pb.fileName, os.O_RDONLY, 0o600)
	if err != nil {
		return err
	}
	pb.fp = fp
	pb.dec = NewWALDecoder(fp)
	count = pb.count - count
	fmt.Printf("Reseting from %d to %d\n", pb.count, count)
	pb.count = 0
	pb.cs = newCS
	var msg *TimedWALMessage
	for i := 0; i < count; i++ {
		msg, err = pb.dec.Decode()
		if err == io.EOF {
			return nil
		} else if err != nil {
			return err
		}
		if err := pb.cs.readReplayMessage(msg, newStepSub); err != nil {
			return err
		}
		pb.count++
	}
	return nil
}

func (cs *State) startForReplay() {
	cs.Logger.Error("Replay commands are disabled until someone updates them and writes tests")
	/* TODO:!
	// since we replay tocks we just ignore ticks
		go func() {
			for {
				select {
				case <-cs.tickChan:
				case <-cs.Quit:
					return
				}
			}
		}()*/
}

// console function for parsing input and running commands
func (pb *playback) replayConsoleLoop() int {
	for {
		fmt.Printf("> ")
		bufReader := bufio.NewReader(os.Stdin)
		line, more, err := bufReader.ReadLine()
		if more {
			cmtos.Exit("input is too long")
		} else if err != nil {
			cmtos.Exit(err.Error())
		}

		tokens := strings.Split(string(line), " ")
		if len(tokens) == 0 {
			continue
		}

		switch tokens[0] {
		case "next":
			// "next" -> replay next message
			// "next N" -> replay next N messages

			if len(tokens) == 1 {
				return 0
			}
			i, err := strconv.Atoi(tokens[1])
			if err != nil {
				fmt.Println("next takes an integer argument")
			} else {
				return i
			}

		case "back":
			// "back" -> go back one message
			// "back N" -> go back N messages

			// NOTE: "back" is not supported in the state machine design,
			// so we restart and replay up to

			ctx := context.Background()
			// ensure all new step events are regenerated as expected

			newStepSub, err := pb.cs.eventBus.Subscribe(ctx, subscriber, types.EventQueryNewRoundStep)
			if err != nil {
				cmtos.Exit(fmt.Sprintf("failed to subscribe %s to %v", subscriber, types.EventQueryNewRoundStep))
			}
			defer func() {
				if err := pb.cs.eventBus.Unsubscribe(ctx, subscriber, types.EventQueryNewRoundStep); err != nil {
					pb.cs.Logger.Error("Error unsubscribing from eventBus", "err", err)
				}
			}()

			if len(tokens) == 1 {
				if err := pb.replayReset(1, newStepSub); err != nil {
					pb.cs.Logger.Error("Replay reset error", "err", err)
				}
			} else {
				i, err := strconv.Atoi(tokens[1])
				if err != nil {
					fmt.Println("back takes an integer argument")
				} else if i > pb.count {
					fmt.Printf("argument to back must not be larger than the current count (%d)\n", pb.count)
				} else if err := pb.replayReset(i, newStepSub); err != nil {
					pb.cs.Logger.Error("Replay reset error", "err", err)
				}
			}

		case "rs":
			// "rs" -> print entire round state
			// "rs short" -> print height/round/step
			// "rs <field>" -> print another field of the round state

			rs := pb.cs.RoundState
			if len(tokens) == 1 {
				fmt.Println(rs)
			} else {
				switch tokens[1] {
				case "short":
					fmt.Printf("%v/%v/%v\n", rs.Height, rs.Round, rs.Step)
				case "validators":
					fmt.Println(rs.Validators)
				case "proposal":
					fmt.Println(rs.Proposal)
				case "proposal_block":
					fmt.Printf("%v %v\n", rs.ProposalBlockParts.StringShort(), rs.ProposalBlock.StringShort())
				case "locked_round":
					fmt.Println(rs.LockedRound)
				case "locked_block":
					fmt.Printf("%v %v\n", rs.LockedBlockParts.StringShort(), rs.LockedBlock.StringShort())
				case "votes":
					fmt.Println(rs.Votes.StringIndented("  "))

				default:
					fmt.Println("Unknown option", tokens[1])
				}
			}
		case "n":
			fmt.Println(pb.count)
		}
	}
}

//--------------------------------------------------------------------------------

// convenience for replay mode
func newConsensusStateForReplay(config cfg.BaseConfig, csConfig *cfg.ConsensusConfig) *State {
	dbType := dbm.BackendType(config.DBBackend)
	// Get BlockStore
	blockStoreDB, err := dbm.NewDB("blockstore", dbType, config.DBDir())
	if err != nil {
		cmtos.Exit(err.Error())
	}
	blockStore := store.NewBlockStore(blockStoreDB)

	// Get State
	stateDB, err := dbm.NewDB("state", dbType, config.DBDir())
	if err != nil {
		cmtos.Exit(err.Error())
	}
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	gdoc, err := sm.MakeGenesisDocFromFile(config.GenesisFile())
	if err != nil {
		cmtos.Exit(err.Error())
	}
	state, err := sm.MakeGenesisState(gdoc)
	if err != nil {
		cmtos.Exit(err.Error())
	}

	// Create proxyAppConn connection (consensus, mempool, query)
	clientCreator := proxy.DefaultClientCreator(config.ProxyApp, config.ABCI, config.DBDir())
	proxyApp := proxy.NewAppConns(clientCreator, proxy.NopMetrics())
	err = proxyApp.Start()
	if err != nil {
		cmtos.Exit(fmt.Sprintf("Error starting proxy app conns: %v", err))
	}

	eventBus := types.NewEventBus()
	if err := eventBus.Start(); err != nil {
		cmtos.Exit(fmt.Sprintf("Failed to start event bus: %v", err))
	}

	handshaker := NewHandshaker(stateStore, state, blockStore, gdoc)
	handshaker.SetEventBus(eventBus)
	err = handshaker.Handshake(proxyApp)
	if err != nil {
		cmtos.Exit(fmt.Sprintf("Error on handshake: %v", err))
	}

	mempool, evpool := emptyMempool{}, sm.EmptyEvidencePool{}
	blockExec := sm.NewBlockExecutor(stateStore, log.TestingLogger(), proxyApp.Consensus(), mempool, evpool, blockStore)

	consensusState := NewState(csConfig, state.Copy(), blockExec,
		blockStore, mempool, evpool)

	consensusState.SetEventBus(eventBus)
	return consensusState
}
