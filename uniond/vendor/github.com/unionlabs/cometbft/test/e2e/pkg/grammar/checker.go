package grammar

import (
	"errors"
	"fmt"
	"os"
	"strings"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/test/e2e/pkg/grammar/grammar-auto/lexer"
	"github.com/cometbft/cometbft/test/e2e/pkg/grammar/grammar-auto/parser"
	"github.com/cometbft/cometbft/test/e2e/pkg/grammar/grammar-auto/parser/symbols"
)

const Commit = "commit"

// Checker is a checker that can verify whether a specific set of ABCI calls
// respects the ABCI grammar.
type Checker struct {
	logger log.Logger
	cfg    *Config
}

// Config allows for setting some parameters, mostly about error logging.
type Config struct {
	// The number of errors checker outputs.
	NumberOfErrorsToShow int
}

// DefaultConfig returns a default config for GrammarChecker.
func DefaultConfig() *Config {
	return &Config{
		NumberOfErrorsToShow: 1,
	}
}

// Error represents the error type checker returns.
type Error struct {
	description string
	// height maps 1-to-1 to the consensus height only in case of clean-start. If it is recovery, it corresponds to the height number after recovery: height 0 is
	// first consensus height after recovery.
	height int
}

// String returns string representation of an error.
func (e *Error) String() string {
	s := fmt.Sprintf("The error: %q has occurred at height %v.", e.description, e.height)
	return s
}

// NewGrammarChecker returns a grammar checker object.
func NewGrammarChecker(cfg *Config) *Checker {
	return &Checker{
		cfg:    cfg,
		logger: log.NewTMLogger(log.NewSyncWriter(os.Stdout)),
	}
}

// isSupportedByGrammar returns true for all requests supported by the current grammar ("/pkg/grammar/clean-start/abci_grammar_clean_start.md" and "/pkg/grammar/recovery/abci_grammar_recovery.md").
// This method needs to be modified if we add another ABCI call.
func (*Checker) isSupportedByGrammar(req *abci.Request) bool {
	switch req.Value.(type) {
	case *abci.Request_InitChain, *abci.Request_FinalizeBlock, *abci.Request_Commit,
		*abci.Request_OfferSnapshot, *abci.Request_ApplySnapshotChunk, *abci.Request_PrepareProposal,
		*abci.Request_ProcessProposal, *abci.Request_ExtendVote, *abci.Request_VerifyVoteExtension:
		return true
	default:
		return false
	}
}

// filterRequests returns requests supported by grammar and remove the last height.
func (g *Checker) filterRequests(reqs []*abci.Request) []*abci.Request {
	var r []*abci.Request
	for _, req := range reqs {
		if g.isSupportedByGrammar(req) {
			r = append(r, req)
		}
	}
	r, _ = g.filterLastHeight(r)
	return r
}

// filterLastHeight removes ABCI requests from the last height if "commit" has not been called
// and returns the tuple (remaining(non-filtered) requests, # of filtered requests).
func (g *Checker) filterLastHeight(reqs []*abci.Request) ([]*abci.Request, int) {
	if len(reqs) == 0 {
		return nil, 0
	}
	pos := len(reqs) - 1
	cnt := 0
	// Find the last commit.
	for pos >= 0 && g.getRequestTerminal(reqs[pos]) != Commit {
		pos--
		cnt++
	}
	return reqs[:pos+1], cnt
}

// getRequestTerminal returns a value of a corresponding terminal in the ABCI grammar for a specific request.
func (*Checker) getRequestTerminal(req *abci.Request) string {
	// req.String() produces an output like this "init_chain:<time:<seconds:-62135596800 > >"
	// we take just the part before the ":" (init_chain, in previous example) for each request
	parts := strings.Split(req.String(), ":")
	if len(parts) < 2 || len(parts[0]) == 0 {
		panic(fmt.Errorf("abci.Request doesn't have the expected string format: %v", req.String()))
	}
	return parts[0]
}

// getExecutionString returns a string of terminal symbols in parser readable format.
func (g *Checker) getExecutionString(reqs []*abci.Request) string {
	s := ""
	for _, r := range reqs {
		t := g.getRequestTerminal(r)
		// We ensure to have one height per line for readability.
		if t == Commit {
			s += t + "\n"
		} else {
			s += t + " "
		}
	}
	return s
}

// Verify verifies whether a list of request satisfy ABCI grammar.
func (g *Checker) Verify(reqs []*abci.Request, isCleanStart bool) (bool, error) {
	if len(reqs) == 0 {
		return false, errors.New("execution with no ABCI calls")
	}
	fullExecution := g.getExecutionString(reqs)
	r := g.filterRequests(reqs)
	// Check if the execution is incomplete.
	if len(r) == 0 {
		return true, nil
	}
	execution := g.getExecutionString(r)
	errors := g.verify(execution, isCleanStart)
	if errors == nil {
		return true, nil
	}
	return false, fmt.Errorf("%v\nFull execution:\n%v", g.combineErrors(errors, g.cfg.NumberOfErrorsToShow), g.addHeightNumbersToTheExecution(fullExecution))
}

// verifyCleanStart verifies if a specific execution is a valid execution.
func (*Checker) verify(execution string, isCleanStart bool) []*Error {
	errors := make([]*Error, 0)
	lexer := lexer.New([]rune(execution))
	bsrForest, errs := parser.Parse(lexer)
	for _, err := range errs {
		exp := []string{}
		for _, ex := range err.Expected {
			exp = append(exp, ex)
		}
		expectedTokens := strings.Join(exp, ",")
		unexpectedToken := err.Token.TypeID()
		e := &Error{
			description: fmt.Sprintf("Invalid execution: parser was expecting one of [%v], got [%v] instead.", expectedTokens, unexpectedToken),
			height:      err.Line - 1,
		}
		errors = append(errors, e)
	}
	if len(errors) != 0 {
		return errors
	}
	eType := symbols.NT_Recovery
	if isCleanStart {
		eType = symbols.NT_CleanStart
	}
	roots := bsrForest.GetRoots()
	for _, r := range roots {
		for _, s := range r.Label.Slot().Symbols {
			if s == eType {
				return nil
			}
		}
	}
	e := &Error{
		description: "The execution is not of valid type.",
		height:      0,
	}
	errors = append(errors, e)
	return errors
}

// addHeightNumbersToTheExecution adds height numbers to the execution. This is used just when printing the execution so we can find the height with error more easily.
func (*Checker) addHeightNumbersToTheExecution(execution string) string {
	heights := strings.Split(execution, "\n")
	s := ""
	for i, l := range heights {
		if i == len(heights)-1 && l == "" {
			break
		}
		s = fmt.Sprintf("%v%v: %v\n", s, i, l)
	}
	return s
}

// combineErrors combines at most n errors in one.
func (*Checker) combineErrors(errors []*Error, n int) error {
	s := ""
	for i, e := range errors {
		if i == n {
			break
		}
		s += e.String() + "\n"
	}
	return fmt.Errorf("%v", s)
}
