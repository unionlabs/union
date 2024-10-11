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
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/dop251/goja"
	"github.com/ethereum/go-ethereum/internal/ethapi"
	"github.com/ethereum/go-ethereum/internal/jsre/deps"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/signer/core"
	"github.com/ethereum/go-ethereum/signer/storage"
)

// consoleOutput is an override for the console.log and console.error methods to
// stream the output into the configured output stream instead of stdout.
func consoleOutput(call goja.FunctionCall) goja.Value {
	output := []string{"JS:> "}
	for _, argument := range call.Arguments {
		output = append(output, fmt.Sprintf("%v", argument))
	}
	fmt.Fprintln(os.Stderr, strings.Join(output, " "))
	return goja.Undefined()
}

// rulesetUI provides an implementation of UIClientAPI that evaluates a javascript
// file for each defined UI-method
type rulesetUI struct {
	next    core.UIClientAPI // The next handler, for manual processing
	storage storage.Storage
	jsRules string // The rules to use
}

func NewRuleEvaluator(next core.UIClientAPI, jsbackend storage.Storage) (*rulesetUI, error) {
	c := &rulesetUI{
		next:    next,
		storage: jsbackend,
		jsRules: "",
	}

	return c, nil
}
func (r *rulesetUI) RegisterUIServer(api *core.UIServerAPI) {
	r.next.RegisterUIServer(api)
	// TODO, make it possible to query from js
}

func (r *rulesetUI) Init(javascriptRules string) error {
	r.jsRules = javascriptRules
	return nil
}
func (r *rulesetUI) execute(jsfunc string, jsarg interface{}) (goja.Value, error) {
	// Instantiate a fresh vm engine every time
	vm := goja.New()

	// Set the native callbacks
	consoleObj := vm.NewObject()
	consoleObj.Set("log", consoleOutput)
	consoleObj.Set("error", consoleOutput)
	vm.Set("console", consoleObj)

	storageObj := vm.NewObject()
	storageObj.Set("put", func(call goja.FunctionCall) goja.Value {
		key, val := call.Argument(0).String(), call.Argument(1).String()
		if val == "" {
			r.storage.Del(key)
		} else {
			r.storage.Put(key, val)
		}
		return goja.Null()
	})
	storageObj.Set("get", func(call goja.FunctionCall) goja.Value {
		goval, _ := r.storage.Get(call.Argument(0).String())
		jsval := vm.ToValue(goval)
		return jsval
	})
	vm.Set("storage", storageObj)

	// Load bootstrap libraries
	script, err := goja.Compile("bignumber.js", deps.BigNumberJS, true)
	if err != nil {
		log.Warn("Failed loading libraries", "err", err)
		return goja.Undefined(), err
	}
	vm.RunProgram(script)

	// Run the actual rule implementation
	_, err = vm.RunString(r.jsRules)
	if err != nil {
		log.Warn("Execution failed", "err", err)
		return goja.Undefined(), err
	}

	// And the actual call
	// All calls are objects with the parameters being keys in that object.
	// To provide additional insulation between js and go, we serialize it into JSON on the Go-side,
	// and deserialize it on the JS side.

	jsonbytes, err := json.Marshal(jsarg)
	if err != nil {
		log.Warn("failed marshalling data", "data", jsarg)
		return goja.Undefined(), err
	}
	// Now, we call foobar(JSON.parse(<jsondata>)).
	var call string
	if len(jsonbytes) > 0 {
		call = fmt.Sprintf("%v(JSON.parse(%v))", jsfunc, string(jsonbytes))
	} else {
		call = fmt.Sprintf("%v()", jsfunc)
	}
	return vm.RunString(call)
}

func (r *rulesetUI) checkApproval(jsfunc string, jsarg []byte, err error) (bool, error) {
	if err != nil {
		return false, err
	}
	v, err := r.execute(jsfunc, string(jsarg))
	if err != nil {
		log.Info("error occurred during execution", "error", err)
		return false, err
	}
	result := v.ToString().String()
	if result == "Approve" {
		log.Info("Op approved")
		return true, nil
	} else if result == "Reject" {
		log.Info("Op rejected")
		return false, nil
	}
	return false, errors.New("unknown response")
}

func (r *rulesetUI) ApproveTx(request *core.SignTxRequest) (core.SignTxResponse, error) {
	jsonreq, err := json.Marshal(request)
	approved, err := r.checkApproval("ApproveTx", jsonreq, err)
	if err != nil {
		log.Info("Rule-based approval error, going to manual", "error", err)
		return r.next.ApproveTx(request)
	}

	if approved {
		return core.SignTxResponse{
				Transaction: request.Transaction,
				Approved:    true},
			nil
	}
	return core.SignTxResponse{Approved: false}, err
}

func (r *rulesetUI) ApproveSignData(request *core.SignDataRequest) (core.SignDataResponse, error) {
	jsonreq, err := json.Marshal(request)
	approved, err := r.checkApproval("ApproveSignData", jsonreq, err)
	if err != nil {
		log.Info("Rule-based approval error, going to manual", "error", err)
		return r.next.ApproveSignData(request)
	}
	if approved {
		return core.SignDataResponse{Approved: true}, nil
	}
	return core.SignDataResponse{Approved: false}, err
}

// OnInputRequired not handled by rules
func (r *rulesetUI) OnInputRequired(info core.UserInputRequest) (core.UserInputResponse, error) {
	return r.next.OnInputRequired(info)
}

func (r *rulesetUI) ApproveListing(request *core.ListRequest) (core.ListResponse, error) {
	jsonreq, err := json.Marshal(request)
	approved, err := r.checkApproval("ApproveListing", jsonreq, err)
	if err != nil {
		log.Info("Rule-based approval error, going to manual", "error", err)
		return r.next.ApproveListing(request)
	}
	if approved {
		return core.ListResponse{Accounts: request.Accounts}, nil
	}
	return core.ListResponse{}, err
}

func (r *rulesetUI) ApproveNewAccount(request *core.NewAccountRequest) (core.NewAccountResponse, error) {
	// This cannot be handled by rules, requires setting a password
	// dispatch to next
	return r.next.ApproveNewAccount(request)
}

func (r *rulesetUI) ShowError(message string) {
	log.Error(message)
	r.next.ShowError(message)
}

func (r *rulesetUI) ShowInfo(message string) {
	log.Info(message)
	r.next.ShowInfo(message)
}

func (r *rulesetUI) OnSignerStartup(info core.StartupInfo) {
	jsonInfo, err := json.Marshal(info)
	if err != nil {
		log.Warn("failed marshalling data", "data", info)
		return
	}
	r.next.OnSignerStartup(info)
	_, err = r.execute("OnSignerStartup", string(jsonInfo))
	if err != nil {
		log.Info("error occurred during execution", "error", err)
	}
}

func (r *rulesetUI) OnApprovedTx(tx ethapi.SignTransactionResult) {
	jsonTx, err := json.Marshal(tx)
	if err != nil {
		log.Warn("failed marshalling transaction", "tx", tx)
		return
	}
	_, err = r.execute("OnApprovedTx", string(jsonTx))
	if err != nil {
		log.Info("error occurred during execution", "error", err)
	}
}
