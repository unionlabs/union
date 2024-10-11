# ADR 030: Consensus Refactor

## Context

One of the biggest challenges this project faces is to proof that the
implementations of the specifications are correct, much like we strive to
formally verify our algorithms and protocols we should work towards high
confidence about the correctness of our program code. One of those is the core
of Tendermint - Consensus - which currently resides in the `consensus` package.
Over time there has been high friction making changes to the package due to the
algorithm being scattered in a side-effectful container (the current
`ConsensusState`). In order to test the algorithm a large object-graph needs to
be set up and even than the non-deterministic parts of the container makes will
prevent high certainty. Where ideally we have a 1-to-1 representation of the
[spec](https://github.com/tendermint/spec), ready and easy to test for domain
experts.

Addresses:

- [#1495](https://github.com/tendermint/tendermint/issues/1495)
- [#1692](https://github.com/tendermint/tendermint/issues/1692)

## Decision

To remedy these issues we plan a gradual, non-invasive refactoring of the
`consensus` package. Starting of by isolating the consensus algorithm into
a pure function and a finite state machine to address the most pressuring issue
of lack of confidence. Doing so while leaving the rest of the package in tact
and have follow-up optional changes to improve the separation of concerns.

### Implementation changes

The core of Consensus can be modelled as a function with clear defined inputs:

* `State` - data container for current round, height, etc.
* `Event`- significant events in the network

producing clear outputs;

* `State` - updated input
* `Message` - signal what actions to perform

```go
type Event int

const (
	EventUnknown Event = iota
	EventProposal
	Majority23PrevotesBlock
	Majority23PrecommitBlock
	Majority23PrevotesAny
	Majority23PrecommitAny
	TimeoutNewRound
	TimeoutPropose
	TimeoutPrevotes
	TimeoutPrecommit
)

type Message int

const (
	MeesageUnknown Message = iota
	MessageProposal
	MessageVotes
	MessageDecision
)

type State struct {
	height      uint64
	round       uint64
	step        uint64
	lockedValue interface{} // TODO: Define proper type.
	lockedRound interface{} // TODO: Define proper type.
	validValue  interface{} // TODO: Define proper type.
	validRound  interface{} // TODO: Define proper type.
	// From the original notes: valid(v)
	valid       interface{} // TODO: Define proper type.
	// From the original notes: proposer(h, r)
	proposer    interface{} // TODO: Define proper type.
}

func Consensus(Event, State) (State, Message) {
	// Consolidate implementation.
}
```

Tracking of relevant information to feed `Event` into the function and act on
the output is left to the `ConsensusExecutor` (formerly `ConsensusState`). 

Benefits for testing surfacing nicely as testing for a sequence of events
against algorithm could be as simple as the following example:

``` go
func TestConsensusXXX(t *testing.T) {
	type expected struct {
		message Message
		state   State
	}

	// Setup order of events, initial state and expectation.
	var (
		events = []struct {
			event Event
			want  expected
		}{
		// ...
		}
		state = State{
		// ...
		}
	)

	for _, e := range events {
		state, msg = Consensus(e.event, state)

		// Test message expectation.
		if msg != e.want.message {
			t.Fatalf("have %v, want %v", msg, e.want.message)
		}

		// Test state expectation.
		if !reflect.DeepEqual(state, e.want.state) {
			t.Fatalf("have %v, want %v", state, e.want.state)
		}
	}
}
```


## Consensus Executor

## Consensus Core

```go
type Event interface{}

type EventNewHeight struct {
    Height           int64
    ValidatorId      int
}

type EventNewRound HeightAndRound

type EventProposal struct {
    Height           int64
    Round            int
    Timestamp        Time
    BlockID          BlockID
    POLRound         int
    Sender           int   
}

type Majority23PrevotesBlock struct {
    Height           int64
    Round            int
    BlockID          BlockID
}

type Majority23PrecommitBlock struct {
    Height           int64
    Round            int
    BlockID          BlockID
}

type HeightAndRound struct {
    Height           int64
    Round            int
}

type Majority23PrevotesAny HeightAndRound
type Majority23PrecommitAny HeightAndRound
type TimeoutPropose HeightAndRound
type TimeoutPrevotes HeightAndRound
type TimeoutPrecommit HeightAndRound


type Message interface{}

type MessageProposal struct {
    Height           int64
    Round            int
    BlockID          BlockID
    POLRound         int
}

type VoteType int

const (
	VoteTypeUnknown VoteType = iota
	Prevote
	Precommit
)


type MessageVote struct {
    Height           int64
    Round            int
    BlockID          BlockID
    Type             VoteType
}


type MessageDecision struct {
    Height           int64
    Round            int
    BlockID          BlockID
}

type TriggerTimeout struct {
    Height           int64
    Round            int
    Duration         Duration
}


type RoundStep int

const (
	RoundStepUnknown RoundStep = iota
	RoundStepPropose       
	RoundStepPrevote
	RoundStepPrecommit
	RoundStepCommit
)

type State struct {
	Height           int64
	Round            int
	Step             RoundStep
	LockedValue      BlockID
	LockedRound      int
	ValidValue       BlockID
	ValidRound       int
	ValidatorId      int
	ValidatorSetSize int
}

func proposer(height int64, round int) int {}
func getValue() BlockID {}

func Consensus(event Event, state State) (State, Message, TriggerTimeout) {
    msg = nil
    timeout = nil
	switch event := event.(type) {
    	case EventNewHeight:
    		if event.Height > state.Height {
    		    state.Height = event.Height
    		    state.Round = -1
    		    state.Step = RoundStepPropose
    		    state.LockedValue = nil
    		    state.LockedRound = -1
    		    state.ValidValue = nil
    		    state.ValidRound = -1
    		    state.ValidatorId = event.ValidatorId
    		} 
    	    return state, msg, timeout
    	
    	case EventNewRound:
    		if event.Height == state.Height and event.Round > state.Round {
               state.Round = eventRound
               state.Step = RoundStepPropose
               if proposer(state.Height, state.Round) == state.ValidatorId {
                   proposal = state.ValidValue
                   if proposal == nil {
                   	    proposal = getValue()
                   }
                   msg =  MessageProposal { state.Height, state.Round, proposal, state.ValidRound }
               }
               timeout = TriggerTimeout { state.Height, state.Round, timeoutPropose(state.Round) }
            }
    	    return state, msg, timeout
    	
    	case EventProposal:
    		if event.Height == state.Height and event.Round == state.Round and 
    	       event.Sender == proposal(state.Height, state.Round) and state.Step == RoundStepPropose { 
    	       	if event.POLRound >= state.LockedRound or event.BlockID == state.BlockID or state.LockedRound == -1 {
    	       		msg = MessageVote { state.Height, state.Round, event.BlockID, Prevote }
    	       	}
    	       	state.Step = RoundStepPrevote
            }
    	    return state, msg, timeout
    	
    	case TimeoutPropose:
    		if event.Height == state.Height and event.Round == state.Round and state.Step == RoundStepPropose {
    		    msg = MessageVote { state.Height, state.Round, nil, Prevote }
    			state.Step = RoundStepPrevote
            }
    	    return state, msg, timeout
    	
    	case Majority23PrevotesBlock:
    		if event.Height == state.Height and event.Round == state.Round and state.Step >= RoundStepPrevote and event.Round > state.ValidRound {
    		    state.ValidRound = event.Round
    		    state.ValidValue = event.BlockID
    		    if state.Step == RoundStepPrevote {
    		    	state.LockedRound = event.Round
    		    	state.LockedValue = event.BlockID
    		    	msg = MessageVote { state.Height, state.Round, event.BlockID, Precommit }
    		    	state.Step = RoundStepPrecommit
    		    }
            }
    	    return state, msg, timeout
    	
    	case Majority23PrevotesAny:
    		if event.Height == state.Height and event.Round == state.Round and state.Step == RoundStepPrevote {
    			timeout = TriggerTimeout { state.Height, state.Round, timeoutPrevote(state.Round) }
    		}
    	    return state, msg, timeout
    	
    	case TimeoutPrevote:
    		if event.Height == state.Height and event.Round == state.Round and state.Step == RoundStepPrevote {
    			msg = MessageVote { state.Height, state.Round, nil, Precommit }
    			state.Step = RoundStepPrecommit
    		}
    	    return state, msg, timeout
    	
    	case Majority23PrecommitBlock:
    		if event.Height == state.Height {
    		    state.Step = RoundStepCommit
    		    state.LockedValue = event.BlockID
    		}
    	    return state, msg, timeout
    		
    	case Majority23PrecommitAny:
    		if event.Height == state.Height and event.Round == state.Round {
    			timeout = TriggerTimeout { state.Height, state.Round, timeoutPrecommit(state.Round) }
    		}
    	    return state, msg, timeout
    	
    	case TimeoutPrecommit:
            if event.Height == state.Height and event.Round == state.Round {
            	state.Round = state.Round + 1
            }
    	    return state, msg, timeout
	}
}	

func ConsensusExecutor() {
	proposal = nil
	votes = HeightVoteSet { Height: 1 }
	state = State {
		Height:       1
		Round:        0          
		Step:         RoundStepPropose
		LockedValue:  nil
		LockedRound:  -1
		ValidValue:   nil
		ValidRound:   -1
	}
	
	event = EventNewHeight {1, id}
	state, msg, timeout = Consensus(event, state)
	
	event = EventNewRound {state.Height, 0}
	state, msg, timeout = Consensus(event, state)
	
	if msg != nil {
		send msg
	}
	
	if timeout != nil {
		trigger timeout
	}
	
	for {
		select {
		    case message := <- msgCh:
		    	switch msg := message.(type) {
		    	    case MessageProposal:
		    	        
		    	    case MessageVote:	
		    	    	if msg.Height == state.Height {
		    	    		newVote = votes.AddVote(msg)
		    	    		if newVote {
		    	    			switch msg.Type {
                                	case Prevote:
                                		prevotes = votes.Prevotes(msg.Round)
                                		if prevotes.WeakCertificate() and msg.Round > state.Round {
                                			event = EventNewRound { msg.Height, msg.Round }
                                			state, msg, timeout = Consensus(event, state)
                                			state = handleStateChange(state, msg, timeout)
                                		}	
                                		
                                		if blockID, ok = prevotes.TwoThirdsMajority(); ok and blockID != nil {
                                		    if msg.Round == state.Round and hasBlock(blockID) {
                                		    	event = Majority23PrevotesBlock { msg.Height, msg.Round, blockID }
                                		    	state, msg, timeout = Consensus(event, state)
                                		    	state = handleStateChange(state, msg, timeout)
                                		    } 
                                		    if proposal != nil and proposal.POLRound == msg.Round and hasBlock(blockID) {
                                		        event = EventProposal {
                                                        Height: state.Height
                                                        Round:  state.Round
                                                        BlockID: blockID
                                                        POLRound: proposal.POLRound
                                                        Sender: message.Sender
                                		        }
                                		        state, msg, timeout = Consensus(event, state)
                                		        state = handleStateChange(state, msg, timeout)
                                		    }
                                		}
                                		
                                		if prevotes.HasTwoThirdsAny() and msg.Round == state.Round {
                                			event = Majority23PrevotesAny { msg.Height, msg.Round, blockID }
                                			state, msg, timeout = Consensus(event, state)
                                            state = handleStateChange(state, msg, timeout)
                                		}
                                		
                                	case Precommit:	
                                		
		    	    		    }
		    	    	    }
		    	        }
		    case timeout := <- timeoutCh:
		    
		    case block := <- blockCh:	
		    	
		}
	}
}
	
func handleStateChange(state, msg, timeout) State {
	if state.Step == Commit {
		state = ExecuteBlock(state.LockedValue)
	}	
	if msg != nil {
		send msg
	}	
	if timeout != nil {
		trigger timeout
	}	
}

```

### Implementation roadmap

* implement proposed implementation
* replace currently scattered calls in `ConsensusState` with calls to the new
  `Consensus` function
* rename `ConsensusState` to `ConsensusExecutor` to avoid confusion
* propose design for improved separation and clear information flow between
  `ConsensusExecutor` and `ConsensusReactor`

## Status

Draft.

## Consequences

### Positive

- isolated implementation of the algorithm
- improved testability - simpler to proof correctness
- clearer separation of concerns - easier to reason

### Negative

### Neutral
