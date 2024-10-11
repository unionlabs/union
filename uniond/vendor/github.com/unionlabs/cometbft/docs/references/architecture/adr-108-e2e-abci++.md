# ADR 108: E2E tests for CometBFT's behaviour in respect to ABCI 1.0.

## Changelog
- 2023-08-08: Initial version (@nenadmilosevic95)
- 2023-15-12: Updated to account for grammar changes (@nenadmilosevic95)


## Context

ABCI 1.0 defines the interface between the application and CometBFT. A part of the specification is the [ABCI 1.0 grammar](../../spec/abci/abci%2B%2B_comet_expected_behavior) that describes the sequences of calls that the application can expect from CometBFT.
In order to demonstrate that CometBFT behaves as expected from the viewpoint of the application, we need to test whether CometBFT respects this ABCI 1.0 grammar. To do this, we need to enhance the e2e tests infrastructure. Specifically, we plan to do three things:
- Log every CometBFT's ABCI 1.0 request during the execution.
- Parse the logs post-mortem and extract all ABCI 1.0 requests.
- Check if the set of observed requests respects the ABCI 1.0 grammar.


Issue: [353](https://github.com/cometbft/cometbft/issues/353).

Current version, ABCI 1.0, does not support vote extensions (ABCI 2.0). However, this is the next step.


## Decision

### 1) ABCI 1.0 requests logging
The idea was to do this at the Application side. Every time the Application
receives a request, it logs it.

**Implementation**

The rationale behind this part of the implementation was to log the request concisely and use the existing structures as much as possible.

Whenever an ABCI 1.0 request is made, the application will create `abci.Request` (`abci` stands for `"github.com/cometbft/cometbft/abci/types"`) and log it.  The example is below.

```go
func (app *Application) InitChain(_ context.Context, req *abci.RequestInitChain) (*abci.ResponseInitChain, error) {

	r := &abci.Request{Value: &abci.Request_InitChain{InitChain: &abci.RequestInitChain{}}}
	err := app.logAbciRequest(r)
	if err != nil {
		return nil, err
	}

	...
}
```
Notice here that we create an empty `abci.RequestInitChain` object while we can also use the one passed to the `InitChain` function. The reason behind this is that, at the moment, we do not need specific fields of the request; we just need to be able to extract the information about the request type. For this, an empty object of a particular type is enough.

The `app.logABCIRequest(r)` function is a new function implemented in the same file (`test/e2e/app/app.go`). If the `ABCIRequestsLoggingEnabled` flag is set to `true`, set automatically when ABCI 1.0 tests are enabled, it logs received requests. The full implementation is the following:

```go
func (app *Application) logABCIRequest(req *abci.Request) error {
	if !app.cfg.ABCIRequestsLoggingEnabled {
		return nil
	}
	s, err := GetABCIRequestString(req)
	if err != nil {
		return err
	}
	app.logger.Info(s)
	return nil
}
```

`GetABCIRequestString(req)` is a new method that receives a request and returns its string representation. The implementation and tests for this function and the opposite function `GetABCIRequestFromString(req)`
that returns `abci.Request` from the string are provided in files `test/e2e/app/log.go` and `test/e2e/app/log_test.go`, respectively. To create a string representation of a request, we first marshal the request via `proto.Marshal()` method and then convert received bytes in the string using `base64.StdEncoding.EncodeToString()` method. In addition, we surround the new string with `abci-req` constants so that we can find lines with ABCI 1.0 request more easily. The code of the method is below:

```go
func GetABCIRequestString(req *abci.Request) (string, error) {
	b, err := proto.Marshal(req)
	if err != nil {
		return "", err
	}
	reqStr := base64.StdEncoding.EncodeToString(b)
	s := ABCI_REQ + reqStr + ABCI_REQ
	return s, nil
}
```

*Note:* At the moment, we are not compressing the marshalled request before converting it to `base64` `string` because we are logging the empty requests that take at most 24 bytes. However, if we decide to log the actual requests in the future, we might want to compress them. Based on a few tests, we observed that the size of a request can go up to 7KB.

If in the future we want to log another ABCI 1.0 request type, we just need to do the same thing:
create a corresponding `abci.Request` and log it via
`app.logABCIRequest(r)`.

### 2) Parsing the logs
We need a code that will take the logs from all nodes and collect the ABCI 1.0 requests that were logged by the application.

**Implementation**

This logic is implemented inside the `fetchABCIRequests(t *testing.T, nodeName string)` function that resides in `test/e2e/tests/e2e_test.go` file. This function does three things:
- Takes the output of a specific node in the testnet from the moment we launched the testnet until the function is called. The node name is passed as a function parameter. It uses the `docker-compose logs` and `grep nodeName` commands.
- Parses the logs line by line and extracts the  `abci.Request`, if one exists. The request is received by forwarding each line to the `app.GetABCIRequestFromString(req)` method.
- Returns the array of slices where each slice contains the set of `abci.Request`s logged on that node. Every time a crash happens, a new array element (new slice `[]*abci.Request`) will be created. We know a crash has happened because we log "Application started" every time the application starts. Specifically, we added this log inside `NewApplication()` function in `test/e2e/app/app.go` file. In the end, `fetchABCIRequests()` will return just one slice if the node did not experience any crashes and $n+1$ slices if there were $n$ crashes. The benefit of logging the requests in the previously described way is that now we can use `[]*abci.Request` to store ABCI 1.0 requests of any type.



### 3) ABCI 1.0 grammar checker
The idea here was to find a library that automatically verifies whether a specific execution respects the prescribed grammar.

**Implementation**

We found the following library - https://github.com/goccmack/gogll. It generates a GLL or LR(1) parser and an FSA-based lexer for any context-free grammar. What we needed to do is to rewrite [ABCI 1.0 grammar](../../../spec/abci/abci++_comet_expected_behavior.md#valid-method-call-sequences)
using the syntax that the library understands.
The new grammar is below and can be found in `test/e2e/pkg/grammar/abci_grammar.md` file.

```abnf

Start : CleanStart | Recovery;

CleanStart : InitChain ConsensusExec | StateSync ConsensusExec ;
StateSync : StateSyncAttempts SuccessSync |  SuccessSync ;
StateSyncAttempts : StateSyncAttempt | StateSyncAttempt StateSyncAttempts ;
StateSyncAttempt : OfferSnapshot ApplyChunks | OfferSnapshot ;
SuccessSync : OfferSnapshot ApplyChunks ;
ApplyChunks : ApplyChunk | ApplyChunk ApplyChunks ;

Recovery :  InitChain ConsensusExec | ConsensusExec ;

ConsensusExec : ConsensusHeights ;
ConsensusHeights : ConsensusHeight | ConsensusHeight ConsensusHeights ;
ConsensusHeight : ConsensusRounds FinalizeBlock Commit | FinalizeBlock Commit ;
ConsensusRounds : ConsensusRound | ConsensusRound ConsensusRounds ;
ConsensusRound : Proposer | NonProposer ;

Proposer : PrepareProposal | PrepareProposal ProcessProposal ;
NonProposer: ProcessProposal ;

InitChain : "init_chain" ;
FinalizeBlock : "finalize_block" ;
Commit : "commit" ;
OfferSnapshot : "offer_snapshot" ;
ApplyChunk : "apply_snapshot_chunk" ;
PrepareProposal : "prepare_proposal" ;
ProcessProposal : "process_proposal" ;

 ```

If you compare this grammar with the original one, you will notice that, in addition to vote extensions,
`Info` is removed. The reason is that, as explained in the section [CometBFT's expected behaviour](../../../spec/abci/abci++_comet_expected_behavior.md#valid-method-call-sequences), one of the
purposes of the `Info` method is being part of the RPC handling from an external
client. Since this can happen at any time, it complicates the
grammar.
This is not true in other cases, but since the Application does
not know why the `Info` is called, we removed
it totally from the new grammar. The Application is still logging the `Info`
call, but a specific test would need to be written to check whether it happens
at the right moment.

Moreover, it is worth noticing that the `(inf)` part of the grammar is replaced with the `*`. This results in the new grammar being finite compared to the original, which represents an infinite (omega) grammar.

The `gogll` library receives the file with the grammar as input, and it generates the corresponding parser and lexer. The actual commands are integrated into `test/e2e/Makefile` and executed when `make grammar-gen` is invoked.
The resulting code is stored inside `test/e2e/pkg/grammar/grammar-auto` directory.

Apart from this auto-generated code, we implemented `GrammarChecker` abstraction
which knows how to use the generated parsers and lexers to verify whether a
specific execution (list of ABCI 1.0 calls logged by the Application while the
testnet was running) respects the ABCI 1.0 grammar. The implementation and tests
for it are inside `test/e2e/pkg/grammar/checker.go` and
`test/e2e/pkg/grammar/checker_test.go`, respectively.

How the `GrammarChecker` works is demonstrated with the test `TestCheckABCIGrammar`
implemented in `test/e2e/tests/abci_test.go` file.

```go
func TestCheckABCIGrammar(t *testing.T) {
	checker := grammar.NewGrammarChecker(grammar.DefaultConfig())
	testNode(t, func(t *testing.T, node e2e.Node) {
		if !node.Testnet.ABCITestsEnabled {
			return
		}
		executions, err := fetchABCIRequests(t, node.Name)
		require.NoError(t, err)
		for i, e := range executions {
			isCleanStart := i == 0
			_, err := checker.Verify(e, isCleanStart)
			require.NoError(t, err)
		}
	})
}
```

Specifically, the test first creates a `GrammarChecker` object. Then for each node in the testnet, it collects all requests
logged by this node. Remember here that `fetchABCIRequests()` returns an array of slices(`[]*abci.Request`) where the slice
with index 0 corresponds to the node's `CleanStart` execution, and each additional slice corresponds to the `Recovery`
execution after a specific crash. Each node must have one `CleanStart` execution and the same number of `Recovery` executions
as the number of crashes that happened on this node. If collecting was successful, the test checks whether each execution
respects the ABCI 1.0
grammar by calling `checker.Verify()` method. If `Verify` returns an error, the specific execution does not respect the
grammar, and the test will fail.

The tests are executed only if `ABCITestsEnabled` is set to `true`. This is done through the manifest file. Namely, if we
want to test whether CometBFT respects ABCI 1.0 grammar, we would need to enable these tests by adding `abci_tests_enabled =
true` in the manifest file of a particular testnet (e.g. `networks/ci.toml`). This will automatically activate logging on the
application side.

The `Verify()` method is shown below.
```go
func (g *Checker) Verify(reqs []*abci.Request, isCleanStart bool) (bool, error) {
	if len(reqs) == 0 {
		return false, fmt.Errorf("execution with no ABCI calls")
	}
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
	return false, fmt.Errorf("%v\nFull execution:\n%v", g.combineErrors(errors, g.cfg.NumberOfErrorsToShow), g.addHeightNumbersToTheExecution(execution))
}
```

It receives a set of ABCI 1.0 requests and a flag saying whether they represent a `CleanStart` execution or not and does the following things:
- Checks if the execution is an empty execution.
- Filter the requests by calling the method `filterRequests()`. This method will remove all the requests from the set that are not supported by the current version of the grammar. In addition, it will filter the last height by removing all ABCI 1.0 requests after the
last `Commit`. The function `fetchABCIRequests()` can be called in the middle of the height. As a result, the last height may be incomplete and
classified as invalid, even if that is not the reality. The simple example here is that the last
request fetched via `fetchABCIRequests()` is `FinalizeBlock`; however, `Commit` happens after
`fetchABCIRequests()` was invoked. Consequently, the execution
will be considered as faulty because `Commit` is missing, even though the `Commit`
will happen after. This is why if the execution consists of only one incomplete height and function `filterRequests()` returns an empty set of requests, the `Verify()` method considers this execution as valid and returns `true`.
- Generates an execution string by replacing `abci.Request` with the
corresponding terminal from the grammar. This logic is implemented in
`getExecutionString()` function. This function receives a list of `abci.Request` and generates a string where every request
will be replaced with a corresponding terminal. For example, request `r` of type `abci.Request_PrepareProposal` is replaced with the string `prepare_proposal`, the first part of `r`'s string representation.
- Checks if the resulting string with terminals respects the grammar by calling the
`verify()` function. The implementations of both functions are the same; they just use different parsers and lexers.
- Returns true if the execution is valid and an error if that's not the case. An example of an error is below.

```
FAIL: TestCheckABCIGrammar/full02 (8.76s)
        abci_test.go:24: ABCI grammar verification failed: The error: "Invalid execution: parser was expecting one of [init_chain], got [offer_snapshot] instead." has occurred at height 0.

            Full execution:
            0: offer_snapshot apply_snapshot_chunk finalize_block commit
            1: finalize_block commit
            2: finalize_block commit
            3: finalize_block commit
			...
```
The error shown above reports an invalid execution. Moreover, it says why it is considered invalid (`init_chain` was missing) and the height of the error. Notice here that the height in the case of `CleanStart` execution corresponds to the actual consensus height, while for the `Recovery` execution, height 0 represents the first height after the crash. Lastly, after the error, the full execution, one height per line, is printed. This part may be optional and handled with a config flag, but we left it like this for now.

*Note:* The `gogll` parser can return many errors because it returns an error at every point at which the parser fails to parse
a grammar production. Usually, the error of interest is the one that has
parsed the largest number of tokens. This is why, by default, we are printing only the last error; however, this can be configured with the `NumberOfErrorsToShow` field of `GrammarChecker`'s config.

Lastly, we present the `verify()` function since this function is the heart of this code.
```go
func (g *Checker) verify(execution string, isCleanStart bool) []*Error {
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
		description: fmt.Sprintf("The execution is not of valid type."),
		height:      0,
	}
	errors = append(errors, e)
	return errors

}

```

This function first checks if the specific execution represents a valid execution concerning the ABCI grammar. For this, it uses
the auto-generated parser and lexer. If the execution passes this initial test, it checks whether the execution is of a valid type (`CleanStart` or `Recovery`). Namely, it checks whether the execution is of the type specified with the function's second parameter (`isCleanStart`).

**Changing the grammar**

Any modification to the grammar (`test/e2e/pkg/grammar/abci_grammar.md`) requires generating a new parser and lexer. This is done by
going to the `test/e2e/` directory and running:

```bash
make grammar-gen
```

Make sure you commit any changes to the auto-generated code together with the changes to the grammar.

### Supporting additional ABCI requests

Here we present all the steps we need to do if we want to support other
ABCI requests in the future:

- The application needs to log the new request in the same way as we do now.
- We should include the new request to the grammar and generate a new parser and lexer.
- We should add new requests to the list of supported requests. Namely, we should modify the function `isSupportedByGrammar()` in `test/e2e/pkg/grammar/checker.go` to return `true` for the new type of requests.

## Status

Implemented.

To-do list:
- adding the CI workflow to check if make grammar is executed.
- extend this ADR (and implementation) to support ABCI 2.0 (i.e., ABCI calls related to vote extensions)
- in the future, we might consider whether the logging (actually, tracing) should be done on the e2e application side, or on CometBFT side, so this infra can be reused for MBT-like activities)
## Consequences

### Positive
- We should be able to check whether CommetBFT respects ABCI 1.0 grammar.
### Negative

### Neutral

