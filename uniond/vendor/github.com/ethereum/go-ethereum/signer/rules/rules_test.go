// Copyright 2018 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rules

import (
	"fmt"
	"math/big"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/internal/ethapi"
	"github.com/ethereum/go-ethereum/signer/core"
	"github.com/ethereum/go-ethereum/signer/core/apitypes"
	"github.com/ethereum/go-ethereum/signer/storage"
)

const JS = `
/**
This is an example implementation of a Javascript rule file.

When the signer receives a request over the external API, the corresponding method is evaluated.
Three things can happen:

1. The method returns "Approve". This means the operation is permitted.
2. The method returns "Reject". This means the operation is rejected.
3. Anything else; other return values [*], method not implemented or exception occurred during processing. This means
that the operation will continue to manual processing, via the regular UI method chosen by the user.

[*] Note: Future version of the ruleset may use more complex json-based return values, making it possible to not
only respond Approve/Reject/Manual, but also modify responses. For example, choose to list only one, but not all
accounts in a list-request. The points above will continue to hold for non-json based responses ("Approve"/"Reject").

**/

function ApproveListing(request){
	console.log("In js approve listing");
	console.log(request.accounts[3].Address)
	console.log(request.meta.Remote)
	return "Approve"
}

function ApproveTx(request){
	console.log("test");
	console.log("from");
	return "Reject";
}

function test(thing){
	console.log(thing.String())
}

`

func mixAddr(a string) (*common.MixedcaseAddress, error) {
	return common.NewMixedcaseAddressFromString(a)
}

type alwaysDenyUI struct{}

func (alwaysDenyUI) OnInputRequired(info core.UserInputRequest) (core.UserInputResponse, error) {
	return core.UserInputResponse{}, nil
}
func (alwaysDenyUI) RegisterUIServer(api *core.UIServerAPI) {
}

func (alwaysDenyUI) OnSignerStartup(info core.StartupInfo) {
}

func (alwaysDenyUI) ApproveTx(request *core.SignTxRequest) (core.SignTxResponse, error) {
	return core.SignTxResponse{Transaction: request.Transaction, Approved: false}, nil
}

func (alwaysDenyUI) ApproveSignData(request *core.SignDataRequest) (core.SignDataResponse, error) {
	return core.SignDataResponse{Approved: false}, nil
}

func (alwaysDenyUI) ApproveListing(request *core.ListRequest) (core.ListResponse, error) {
	return core.ListResponse{Accounts: nil}, nil
}

func (alwaysDenyUI) ApproveNewAccount(request *core.NewAccountRequest) (core.NewAccountResponse, error) {
	return core.NewAccountResponse{Approved: false}, nil
}

func (alwaysDenyUI) ShowError(message string) {
	panic("implement me")
}

func (alwaysDenyUI) ShowInfo(message string) {
	panic("implement me")
}

func (alwaysDenyUI) OnApprovedTx(tx ethapi.SignTransactionResult) {
	panic("implement me")
}

func initRuleEngine(js string) (*rulesetUI, error) {
	r, err := NewRuleEvaluator(&alwaysDenyUI{}, storage.NewEphemeralStorage())
	if err != nil {
		return nil, fmt.Errorf("failed to create js engine: %v", err)
	}
	if err = r.Init(js); err != nil {
		return nil, fmt.Errorf("failed to load bootstrap js: %v", err)
	}
	return r, nil
}

func TestListRequest(t *testing.T) {
	accs := make([]accounts.Account, 5)

	for i := range accs {
		addr := fmt.Sprintf("000000000000000000000000000000000000000%x", i)
		acc := accounts.Account{
			Address: common.BytesToAddress(common.Hex2Bytes(addr)),
			URL:     accounts.URL{Scheme: "test", Path: fmt.Sprintf("acc-%d", i)},
		}
		accs[i] = acc
	}

	js := `function ApproveListing(){ return "Approve" }`

	r, err := initRuleEngine(js)
	if err != nil {
		t.Errorf("Couldn't create evaluator %v", err)
		return
	}
	resp, _ := r.ApproveListing(&core.ListRequest{
		Accounts: accs,
		Meta:     core.Metadata{Remote: "remoteip", Local: "localip", Scheme: "inproc"},
	})
	if len(resp.Accounts) != len(accs) {
		t.Errorf("Expected check to resolve to 'Approve'")
	}
}

func TestSignTxRequest(t *testing.T) {
	js := `
	function ApproveTx(r){
		console.log("transaction.from", r.transaction.from);
		console.log("transaction.to", r.transaction.to);
		console.log("transaction.value", r.transaction.value);
		console.log("transaction.nonce", r.transaction.nonce);
		if(r.transaction.from.toLowerCase()=="0x0000000000000000000000000000000000001337"){ return "Approve"}
		if(r.transaction.from.toLowerCase()=="0x000000000000000000000000000000000000dead"){ return "Reject"}
	}`

	r, err := initRuleEngine(js)
	if err != nil {
		t.Errorf("Couldn't create evaluator %v", err)
		return
	}
	to, err := mixAddr("000000000000000000000000000000000000dead")
	if err != nil {
		t.Error(err)
		return
	}
	from, err := mixAddr("0000000000000000000000000000000000001337")

	if err != nil {
		t.Error(err)
		return
	}
	t.Logf("to %v", to.Address().String())
	resp, err := r.ApproveTx(&core.SignTxRequest{
		Transaction: apitypes.SendTxArgs{
			From: *from,
			To:   to},
		Callinfo: nil,
		Meta:     core.Metadata{Remote: "remoteip", Local: "localip", Scheme: "inproc"},
	})
	if err != nil {
		t.Errorf("Unexpected error %v", err)
	}
	if !resp.Approved {
		t.Errorf("Expected check to resolve to 'Approve'")
	}
}

type dummyUI struct {
	calls []string
}

func (d *dummyUI) RegisterUIServer(api *core.UIServerAPI) {
	panic("implement me")
}

func (d *dummyUI) OnInputRequired(info core.UserInputRequest) (core.UserInputResponse, error) {
	d.calls = append(d.calls, "OnInputRequired")
	return core.UserInputResponse{}, nil
}

func (d *dummyUI) ApproveTx(request *core.SignTxRequest) (core.SignTxResponse, error) {
	d.calls = append(d.calls, "ApproveTx")
	return core.SignTxResponse{}, core.ErrRequestDenied
}

func (d *dummyUI) ApproveSignData(request *core.SignDataRequest) (core.SignDataResponse, error) {
	d.calls = append(d.calls, "ApproveSignData")
	return core.SignDataResponse{}, core.ErrRequestDenied
}

func (d *dummyUI) ApproveListing(request *core.ListRequest) (core.ListResponse, error) {
	d.calls = append(d.calls, "ApproveListing")
	return core.ListResponse{}, core.ErrRequestDenied
}

func (d *dummyUI) ApproveNewAccount(request *core.NewAccountRequest) (core.NewAccountResponse, error) {
	d.calls = append(d.calls, "ApproveNewAccount")
	return core.NewAccountResponse{}, core.ErrRequestDenied
}

func (d *dummyUI) ShowError(message string) {
	d.calls = append(d.calls, "ShowError")
}

func (d *dummyUI) ShowInfo(message string) {
	d.calls = append(d.calls, "ShowInfo")
}

func (d *dummyUI) OnApprovedTx(tx ethapi.SignTransactionResult) {
	d.calls = append(d.calls, "OnApprovedTx")
}

func (d *dummyUI) OnSignerStartup(info core.StartupInfo) {
}

// TestForwarding tests that the rule-engine correctly dispatches requests to the next caller
func TestForwarding(t *testing.T) {
	js := ""
	ui := &dummyUI{make([]string, 0)}
	jsBackend := storage.NewEphemeralStorage()
	r, err := NewRuleEvaluator(ui, jsBackend)
	if err != nil {
		t.Fatalf("Failed to create js engine: %v", err)
	}
	if err = r.Init(js); err != nil {
		t.Fatalf("Failed to load bootstrap js: %v", err)
	}
	r.ApproveSignData(nil)
	r.ApproveTx(nil)
	r.ApproveNewAccount(nil)
	r.ApproveListing(nil)
	r.ShowError("test")
	r.ShowInfo("test")

	//This one is not forwarded
	r.OnApprovedTx(ethapi.SignTransactionResult{})

	expCalls := 6
	if len(ui.calls) != expCalls {
		t.Errorf("Expected %d forwarded calls, got %d: %s", expCalls, len(ui.calls), strings.Join(ui.calls, ","))
	}
}

func TestMissingFunc(t *testing.T) {
	r, err := initRuleEngine(JS)
	if err != nil {
		t.Errorf("Couldn't create evaluator %v", err)
		return
	}

	_, err = r.execute("MissingMethod", "test")

	if err == nil {
		t.Error("Expected error")
	}

	approved, err := r.checkApproval("MissingMethod", nil, nil)
	if err == nil {
		t.Errorf("Expected missing method to yield error'")
	}
	if approved {
		t.Errorf("Expected missing method to cause non-approval")
	}
	t.Logf("Err %v", err)
}
func TestStorage(t *testing.T) {
	js := `
	function testStorage(){
		storage.put("mykey", "myvalue")
		a = storage.get("mykey")

		storage.put("mykey", ["a", "list"])  	// Should result in "a,list"
		a += storage.get("mykey")


		storage.put("mykey", {"an": "object"}) 	// Should result in "[object Object]"
		a += storage.get("mykey")


		storage.put("mykey", JSON.stringify({"an": "object"})) // Should result in '{"an":"object"}'
		a += storage.get("mykey")

		a += storage.get("missingkey")		//Missing keys should result in empty string
		storage.put("","missing key==noop") // Can't store with 0-length key
		a += storage.get("")				// Should result in ''

		var b = new BigNumber(2)
		var c = new BigNumber(16)//"0xf0",16)
		var d = b.plus(c)
		console.log(d)
		return a
	}
`
	r, err := initRuleEngine(js)
	if err != nil {
		t.Errorf("Couldn't create evaluator %v", err)
		return
	}

	v, err := r.execute("testStorage", nil)

	if err != nil {
		t.Errorf("Unexpected error %v", err)
	}
	retval := v.ToString().String()

	if err != nil {
		t.Errorf("Unexpected error %v", err)
	}
	exp := `myvaluea,list[object Object]{"an":"object"}`
	if retval != exp {
		t.Errorf("Unexpected data, expected '%v', got '%v'", exp, retval)
	}
	t.Logf("Err %v", err)
}

const ExampleTxWindow = `
	function big(str){
		if(str.slice(0,2) == "0x"){ return new BigNumber(str.slice(2),16)}
		return new BigNumber(str)
	}

	// Time window: 1 week
	var window = 1000* 3600*24*7;

	// Limit : 1 ether
	var limit = new BigNumber("1e18");

	function isLimitOk(transaction){
		var value = big(transaction.value)
		// Start of our window function
		var windowstart = new Date().getTime() - window;

		var txs = [];
		var stored = storage.get('txs');

		if(stored != ""){
			txs = JSON.parse(stored)
		}
		// First, remove all that have passed out of the time-window
		var newtxs = txs.filter(function(tx){return tx.tstamp > windowstart});
		console.log(txs, newtxs.length);

		// Secondly, aggregate the current sum
		sum = new BigNumber(0)

		sum = newtxs.reduce(function(agg, tx){ return big(tx.value).plus(agg)}, sum);
		console.log("ApproveTx > Sum so far", sum);
		console.log("ApproveTx > Requested", value.toNumber());

		// Would we exceed weekly limit ?
		return sum.plus(value).lt(limit)

	}
	function ApproveTx(r){
		console.log(r)
		console.log(typeof(r))
		if (isLimitOk(r.transaction)){
			return "Approve"
		}
		return "Nope"
	}

	/**
	* OnApprovedTx(str) is called when a transaction has been approved and signed. The parameter
 	* 'response_str' contains the return value that will be sent to the external caller.
	* The return value from this method is ignore - the reason for having this callback is to allow the
	* ruleset to keep track of approved transactions.
	*
	* When implementing rate-limited rules, this callback should be used.
	* If a rule responds with neither 'Approve' nor 'Reject' - the tx goes to manual processing. If the user
	* then accepts the transaction, this method will be called.
	*
	* TLDR; Use this method to keep track of signed transactions, instead of using the data in ApproveTx.
	*/
 	function OnApprovedTx(resp){
		var value = big(resp.tx.value)
		var txs = []
		// Load stored transactions
		var stored = storage.get('txs');
		if(stored != ""){
			txs = JSON.parse(stored)
		}
		// Add this to the storage
		txs.push({tstamp: new Date().getTime(), value: value});
		storage.put("txs", JSON.stringify(txs));
	}

`

func dummyTx(value hexutil.Big) *core.SignTxRequest {
	to, _ := mixAddr("000000000000000000000000000000000000dead")
	from, _ := mixAddr("000000000000000000000000000000000000dead")
	n := hexutil.Uint64(3)
	gas := hexutil.Uint64(21000)
	gasPrice := hexutil.Big(*big.NewInt(2000000))

	return &core.SignTxRequest{
		Transaction: apitypes.SendTxArgs{
			From:     *from,
			To:       to,
			Value:    value,
			Nonce:    n,
			GasPrice: &gasPrice,
			Gas:      gas,
		},
		Callinfo: []apitypes.ValidationInfo{
			{Typ: "Warning", Message: "All your base are belong to us"},
		},
		Meta: core.Metadata{Remote: "remoteip", Local: "localip", Scheme: "inproc"},
	}
}

func dummyTxWithV(value uint64) *core.SignTxRequest {
	v := new(big.Int).SetUint64(value)
	h := hexutil.Big(*v)
	return dummyTx(h)
}

func dummySigned(value *big.Int) *types.Transaction {
	to := common.HexToAddress("000000000000000000000000000000000000dead")
	gas := uint64(21000)
	gasPrice := big.NewInt(2000000)
	data := make([]byte, 0)
	return types.NewTransaction(3, to, value, gas, gasPrice, data)
}

func TestLimitWindow(t *testing.T) {
	r, err := initRuleEngine(ExampleTxWindow)
	if err != nil {
		t.Errorf("Couldn't create evaluator %v", err)
		return
	}
	// 0.3 ether: 429D069189E0000 wei
	v := new(big.Int).SetBytes(common.Hex2Bytes("0429D069189E0000"))
	h := hexutil.Big(*v)
	// The first three should succeed
	for i := 0; i < 3; i++ {
		unsigned := dummyTx(h)
		resp, err := r.ApproveTx(unsigned)
		if err != nil {
			t.Errorf("Unexpected error %v", err)
		}
		if !resp.Approved {
			t.Errorf("Expected check to resolve to 'Approve'")
		}
		// Create a dummy signed transaction

		response := ethapi.SignTransactionResult{
			Tx:  dummySigned(v),
			Raw: common.Hex2Bytes("deadbeef"),
		}
		r.OnApprovedTx(response)
	}
	// Fourth should fail
	resp, _ := r.ApproveTx(dummyTx(h))
	if resp.Approved {
		t.Errorf("Expected check to resolve to 'Reject'")
	}
}

// dontCallMe is used as a next-handler that does not want to be called - it invokes test failure
type dontCallMe struct {
	t *testing.T
}

func (d *dontCallMe) OnInputRequired(info core.UserInputRequest) (core.UserInputResponse, error) {
	d.t.Fatalf("Did not expect next-handler to be called")
	return core.UserInputResponse{}, nil
}

func (d *dontCallMe) RegisterUIServer(api *core.UIServerAPI) {
}

func (d *dontCallMe) OnSignerStartup(info core.StartupInfo) {
}

func (d *dontCallMe) ApproveTx(request *core.SignTxRequest) (core.SignTxResponse, error) {
	d.t.Fatalf("Did not expect next-handler to be called")
	return core.SignTxResponse{}, core.ErrRequestDenied
}

func (d *dontCallMe) ApproveSignData(request *core.SignDataRequest) (core.SignDataResponse, error) {
	d.t.Fatalf("Did not expect next-handler to be called")
	return core.SignDataResponse{}, core.ErrRequestDenied
}

func (d *dontCallMe) ApproveListing(request *core.ListRequest) (core.ListResponse, error) {
	d.t.Fatalf("Did not expect next-handler to be called")
	return core.ListResponse{}, core.ErrRequestDenied
}

func (d *dontCallMe) ApproveNewAccount(request *core.NewAccountRequest) (core.NewAccountResponse, error) {
	d.t.Fatalf("Did not expect next-handler to be called")
	return core.NewAccountResponse{}, core.ErrRequestDenied
}

func (d *dontCallMe) ShowError(message string) {
	d.t.Fatalf("Did not expect next-handler to be called")
}

func (d *dontCallMe) ShowInfo(message string) {
	d.t.Fatalf("Did not expect next-handler to be called")
}

func (d *dontCallMe) OnApprovedTx(tx ethapi.SignTransactionResult) {
	d.t.Fatalf("Did not expect next-handler to be called")
}

// TestContextIsCleared tests that the rule-engine does not retain variables over several requests.
// if it does, that would be bad since developers may rely on that to store data,
// instead of using the disk-based data storage
func TestContextIsCleared(t *testing.T) {
	js := `
	function ApproveTx(){
		if (typeof foobar == 'undefined') {
			foobar = "Approve"
 		}
		console.log(foobar)
		if (foobar == "Approve"){
			foobar = "Reject"
		}else{
			foobar = "Approve"
		}
		return foobar
	}
	`
	ui := &dontCallMe{t}
	r, err := NewRuleEvaluator(ui, storage.NewEphemeralStorage())
	if err != nil {
		t.Fatalf("Failed to create js engine: %v", err)
	}
	if err = r.Init(js); err != nil {
		t.Fatalf("Failed to load bootstrap js: %v", err)
	}
	tx := dummyTxWithV(0)
	r1, _ := r.ApproveTx(tx)
	r2, _ := r.ApproveTx(tx)
	if r1.Approved != r2.Approved {
		t.Errorf("Expected execution context to be cleared between executions")
	}
}

func TestSignData(t *testing.T) {
	js := `function ApproveListing(){
    return "Approve"
}
function ApproveSignData(r){
    if( r.address.toLowerCase() == "0x694267f14675d7e1b9494fd8d72fefe1755710fa")
    {
        if(r.messages[0].value.indexOf("bazonk") >= 0){
            return "Approve"
        }
        return "Reject"
    }
    // Otherwise goes to manual processing
}`
	r, err := initRuleEngine(js)
	if err != nil {
		t.Errorf("Couldn't create evaluator %v", err)
		return
	}
	message := "baz bazonk foo"
	hash, rawdata := accounts.TextAndHash([]byte(message))
	addr, _ := mixAddr("0x694267f14675d7e1b9494fd8d72fefe1755710fa")

	t.Logf("address %v %v\n", addr.String(), addr.Original())

	nvt := []*apitypes.NameValueType{
		{
			Name:  "message",
			Typ:   "text/plain",
			Value: message,
		},
	}
	resp, err := r.ApproveSignData(&core.SignDataRequest{
		Address:  *addr,
		Messages: nvt,
		Hash:     hash,
		Meta:     core.Metadata{Remote: "remoteip", Local: "localip", Scheme: "inproc"},
		Rawdata:  []byte(rawdata),
	})
	if err != nil {
		t.Fatalf("Unexpected error %v", err)
	}
	if !resp.Approved {
		t.Fatalf("Expected approved")
	}
}
