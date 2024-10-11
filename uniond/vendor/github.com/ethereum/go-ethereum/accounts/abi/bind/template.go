// Copyright 2016 The go-ethereum Authors
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

package bind

import "github.com/ethereum/go-ethereum/accounts/abi"

// tmplData is the data structure required to fill the binding template.
type tmplData struct {
	Package   string                   // Name of the package to place the generated file in
	Contracts map[string]*tmplContract // List of contracts to generate into this file
	Libraries map[string]string        // Map the bytecode's link pattern to the library name
	Structs   map[string]*tmplStruct   // Contract struct type definitions
}

// tmplContract contains the data needed to generate an individual contract binding.
type tmplContract struct {
	Type        string                 // Type name of the main contract binding
	InputABI    string                 // JSON ABI used as the input to generate the binding from
	InputBin    string                 // Optional EVM bytecode used to generate deploy code from
	FuncSigs    map[string]string      // Optional map: string signature -> 4-byte signature
	Constructor abi.Method             // Contract constructor for deploy parametrization
	Calls       map[string]*tmplMethod // Contract calls that only read state data
	Transacts   map[string]*tmplMethod // Contract calls that write state data
	Fallback    *tmplMethod            // Additional special fallback function
	Receive     *tmplMethod            // Additional special receive function
	Events      map[string]*tmplEvent  // Contract events accessors
	Libraries   map[string]string      // Same as tmplData, but filtered to only keep what the contract needs
	Library     bool                   // Indicator whether the contract is a library
}

// tmplMethod is a wrapper around an abi.Method that contains a few preprocessed
// and cached data fields.
type tmplMethod struct {
	Original   abi.Method // Original method as parsed by the abi package
	Normalized abi.Method // Normalized version of the parsed method (capitalized names, non-anonymous args/returns)
	Structured bool       // Whether the returns should be accumulated into a struct
}

// tmplEvent is a wrapper around an abi.Event that contains a few preprocessed
// and cached data fields.
type tmplEvent struct {
	Original   abi.Event // Original event as parsed by the abi package
	Normalized abi.Event // Normalized version of the parsed fields
}

// tmplField is a wrapper around a struct field with binding language
// struct type definition and relative filed name.
type tmplField struct {
	Type    string   // Field type representation depends on target binding language
	Name    string   // Field name converted from the raw user-defined field name
	SolKind abi.Type // Raw abi type information
}

// tmplStruct is a wrapper around an abi.tuple and contains an auto-generated
// struct name.
type tmplStruct struct {
	Name   string       // Auto-generated struct name(before solidity v0.5.11) or raw name.
	Fields []*tmplField // Struct fields definition depends on the binding language.
}

// tmplSource is language to template mapping containing all the supported
// programming languages the package can generate to.
var tmplSource = map[Lang]string{
	LangGo: tmplSourceGo,
}

// tmplSourceGo is the Go source template that the generated Go contract binding
// is based on.
const tmplSourceGo = `
// Code generated - DO NOT EDIT.
// This file is a generated binding and any manual changes will be lost.

package {{.Package}}

import (
	"math/big"
	"strings"
	"errors"

	ethereum "github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/event"
)

// Reference imports to suppress errors if they are not otherwise used.
var (
	_ = errors.New
	_ = big.NewInt
	_ = strings.NewReader
	_ = ethereum.NotFound
	_ = bind.Bind
	_ = common.Big1
	_ = types.BloomLookup
	_ = event.NewSubscription
	_ = abi.ConvertType
)

{{$structs := .Structs}}
{{range $structs}}
	// {{.Name}} is an auto generated low-level Go binding around an user-defined struct.
	type {{.Name}} struct {
	{{range $field := .Fields}}
	{{$field.Name}} {{$field.Type}}{{end}}
	}
{{end}}

{{range $contract := .Contracts}}
	// {{.Type}}MetaData contains all meta data concerning the {{.Type}} contract.
	var {{.Type}}MetaData = &bind.MetaData{
		ABI: "{{.InputABI}}",
		{{if $contract.FuncSigs -}}
		Sigs: map[string]string{
			{{range $strsig, $binsig := .FuncSigs}}"{{$binsig}}": "{{$strsig}}",
			{{end}}
		},
		{{end -}}
		{{if .InputBin -}}
		Bin: "0x{{.InputBin}}",
		{{end}}
	}
	// {{.Type}}ABI is the input ABI used to generate the binding from.
	// Deprecated: Use {{.Type}}MetaData.ABI instead.
	var {{.Type}}ABI = {{.Type}}MetaData.ABI

	{{if $contract.FuncSigs}}
		// Deprecated: Use {{.Type}}MetaData.Sigs instead.
		// {{.Type}}FuncSigs maps the 4-byte function signature to its string representation.
		var {{.Type}}FuncSigs = {{.Type}}MetaData.Sigs
	{{end}}

	{{if .InputBin}}
		// {{.Type}}Bin is the compiled bytecode used for deploying new contracts.
		// Deprecated: Use {{.Type}}MetaData.Bin instead.
		var {{.Type}}Bin = {{.Type}}MetaData.Bin

		// Deploy{{.Type}} deploys a new Ethereum contract, binding an instance of {{.Type}} to it.
		func Deploy{{.Type}}(auth *bind.TransactOpts, backend bind.ContractBackend {{range .Constructor.Inputs}}, {{.Name}} {{bindtype .Type $structs}}{{end}}) (common.Address, *types.Transaction, *{{.Type}}, error) {
		  parsed, err := {{.Type}}MetaData.GetAbi()
		  if err != nil {
		    return common.Address{}, nil, nil, err
		  }
		  if parsed == nil {
			return common.Address{}, nil, nil, errors.New("GetABI returned nil")
		  }
		  {{range $pattern, $name := .Libraries}}
			{{decapitalise $name}}Addr, _, _, _ := Deploy{{capitalise $name}}(auth, backend)
			{{$contract.Type}}Bin = strings.ReplaceAll({{$contract.Type}}Bin, "__${{$pattern}}$__", {{decapitalise $name}}Addr.String()[2:])
		  {{end}}
		  address, tx, contract, err := bind.DeployContract(auth, *parsed, common.FromHex({{.Type}}Bin), backend {{range .Constructor.Inputs}}, {{.Name}}{{end}})
		  if err != nil {
		    return common.Address{}, nil, nil, err
		  }
		  return address, tx, &{{.Type}}{ {{.Type}}Caller: {{.Type}}Caller{contract: contract}, {{.Type}}Transactor: {{.Type}}Transactor{contract: contract}, {{.Type}}Filterer: {{.Type}}Filterer{contract: contract} }, nil
		}
	{{end}}

	// {{.Type}} is an auto generated Go binding around an Ethereum contract.
	type {{.Type}} struct {
	  {{.Type}}Caller     // Read-only binding to the contract
	  {{.Type}}Transactor // Write-only binding to the contract
	  {{.Type}}Filterer   // Log filterer for contract events
	}

	// {{.Type}}Caller is an auto generated read-only Go binding around an Ethereum contract.
	type {{.Type}}Caller struct {
	  contract *bind.BoundContract // Generic contract wrapper for the low level calls
	}

	// {{.Type}}Transactor is an auto generated write-only Go binding around an Ethereum contract.
	type {{.Type}}Transactor struct {
	  contract *bind.BoundContract // Generic contract wrapper for the low level calls
	}

	// {{.Type}}Filterer is an auto generated log filtering Go binding around an Ethereum contract events.
	type {{.Type}}Filterer struct {
	  contract *bind.BoundContract // Generic contract wrapper for the low level calls
	}

	// {{.Type}}Session is an auto generated Go binding around an Ethereum contract,
	// with pre-set call and transact options.
	type {{.Type}}Session struct {
	  Contract     *{{.Type}}        // Generic contract binding to set the session for
	  CallOpts     bind.CallOpts     // Call options to use throughout this session
	  TransactOpts bind.TransactOpts // Transaction auth options to use throughout this session
	}

	// {{.Type}}CallerSession is an auto generated read-only Go binding around an Ethereum contract,
	// with pre-set call options.
	type {{.Type}}CallerSession struct {
	  Contract *{{.Type}}Caller // Generic contract caller binding to set the session for
	  CallOpts bind.CallOpts    // Call options to use throughout this session
	}

	// {{.Type}}TransactorSession is an auto generated write-only Go binding around an Ethereum contract,
	// with pre-set transact options.
	type {{.Type}}TransactorSession struct {
	  Contract     *{{.Type}}Transactor // Generic contract transactor binding to set the session for
	  TransactOpts bind.TransactOpts    // Transaction auth options to use throughout this session
	}

	// {{.Type}}Raw is an auto generated low-level Go binding around an Ethereum contract.
	type {{.Type}}Raw struct {
	  Contract *{{.Type}} // Generic contract binding to access the raw methods on
	}

	// {{.Type}}CallerRaw is an auto generated low-level read-only Go binding around an Ethereum contract.
	type {{.Type}}CallerRaw struct {
		Contract *{{.Type}}Caller // Generic read-only contract binding to access the raw methods on
	}

	// {{.Type}}TransactorRaw is an auto generated low-level write-only Go binding around an Ethereum contract.
	type {{.Type}}TransactorRaw struct {
		Contract *{{.Type}}Transactor // Generic write-only contract binding to access the raw methods on
	}

	// New{{.Type}} creates a new instance of {{.Type}}, bound to a specific deployed contract.
	func New{{.Type}}(address common.Address, backend bind.ContractBackend) (*{{.Type}}, error) {
	  contract, err := bind{{.Type}}(address, backend, backend, backend)
	  if err != nil {
	    return nil, err
	  }
	  return &{{.Type}}{ {{.Type}}Caller: {{.Type}}Caller{contract: contract}, {{.Type}}Transactor: {{.Type}}Transactor{contract: contract}, {{.Type}}Filterer: {{.Type}}Filterer{contract: contract} }, nil
	}

	// New{{.Type}}Caller creates a new read-only instance of {{.Type}}, bound to a specific deployed contract.
	func New{{.Type}}Caller(address common.Address, caller bind.ContractCaller) (*{{.Type}}Caller, error) {
	  contract, err := bind{{.Type}}(address, caller, nil, nil)
	  if err != nil {
	    return nil, err
	  }
	  return &{{.Type}}Caller{contract: contract}, nil
	}

	// New{{.Type}}Transactor creates a new write-only instance of {{.Type}}, bound to a specific deployed contract.
	func New{{.Type}}Transactor(address common.Address, transactor bind.ContractTransactor) (*{{.Type}}Transactor, error) {
	  contract, err := bind{{.Type}}(address, nil, transactor, nil)
	  if err != nil {
	    return nil, err
	  }
	  return &{{.Type}}Transactor{contract: contract}, nil
	}

	// New{{.Type}}Filterer creates a new log filterer instance of {{.Type}}, bound to a specific deployed contract.
 	func New{{.Type}}Filterer(address common.Address, filterer bind.ContractFilterer) (*{{.Type}}Filterer, error) {
 	  contract, err := bind{{.Type}}(address, nil, nil, filterer)
 	  if err != nil {
 	    return nil, err
 	  }
 	  return &{{.Type}}Filterer{contract: contract}, nil
 	}

	// bind{{.Type}} binds a generic wrapper to an already deployed contract.
	func bind{{.Type}}(address common.Address, caller bind.ContractCaller, transactor bind.ContractTransactor, filterer bind.ContractFilterer) (*bind.BoundContract, error) {
	  parsed, err := {{.Type}}MetaData.GetAbi()
	  if err != nil {
	    return nil, err
	  }
	  return bind.NewBoundContract(address, *parsed, caller, transactor, filterer), nil
	}

	// Call invokes the (constant) contract method with params as input values and
	// sets the output to result. The result type might be a single field for simple
	// returns, a slice of interfaces for anonymous returns and a struct for named
	// returns.
	func (_{{$contract.Type}} *{{$contract.Type}}Raw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
		return _{{$contract.Type}}.Contract.{{$contract.Type}}Caller.contract.Call(opts, result, method, params...)
	}

	// Transfer initiates a plain transaction to move funds to the contract, calling
	// its default method if one is available.
	func (_{{$contract.Type}} *{{$contract.Type}}Raw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
		return _{{$contract.Type}}.Contract.{{$contract.Type}}Transactor.contract.Transfer(opts)
	}

	// Transact invokes the (paid) contract method with params as input values.
	func (_{{$contract.Type}} *{{$contract.Type}}Raw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
		return _{{$contract.Type}}.Contract.{{$contract.Type}}Transactor.contract.Transact(opts, method, params...)
	}

	// Call invokes the (constant) contract method with params as input values and
	// sets the output to result. The result type might be a single field for simple
	// returns, a slice of interfaces for anonymous returns and a struct for named
	// returns.
	func (_{{$contract.Type}} *{{$contract.Type}}CallerRaw) Call(opts *bind.CallOpts, result *[]interface{}, method string, params ...interface{}) error {
		return _{{$contract.Type}}.Contract.contract.Call(opts, result, method, params...)
	}

	// Transfer initiates a plain transaction to move funds to the contract, calling
	// its default method if one is available.
	func (_{{$contract.Type}} *{{$contract.Type}}TransactorRaw) Transfer(opts *bind.TransactOpts) (*types.Transaction, error) {
		return _{{$contract.Type}}.Contract.contract.Transfer(opts)
	}

	// Transact invokes the (paid) contract method with params as input values.
	func (_{{$contract.Type}} *{{$contract.Type}}TransactorRaw) Transact(opts *bind.TransactOpts, method string, params ...interface{}) (*types.Transaction, error) {
		return _{{$contract.Type}}.Contract.contract.Transact(opts, method, params...)
	}

	{{range .Calls}}
		// {{.Normalized.Name}} is a free data retrieval call binding the contract method 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Caller) {{.Normalized.Name}}(opts *bind.CallOpts {{range .Normalized.Inputs}}, {{.Name}} {{bindtype .Type $structs}} {{end}}) ({{if .Structured}}struct{ {{range .Normalized.Outputs}}{{.Name}} {{bindtype .Type $structs}};{{end}} },{{else}}{{range .Normalized.Outputs}}{{bindtype .Type $structs}},{{end}}{{end}} error) {
			var out []interface{}
			err := _{{$contract.Type}}.contract.Call(opts, &out, "{{.Original.Name}}" {{range .Normalized.Inputs}}, {{.Name}}{{end}})
			{{if .Structured}}
			outstruct := new(struct{ {{range .Normalized.Outputs}} {{.Name}} {{bindtype .Type $structs}}; {{end}} })
			if err != nil {
				return *outstruct, err
			}
			{{range $i, $t := .Normalized.Outputs}}
			outstruct.{{.Name}} = *abi.ConvertType(out[{{$i}}], new({{bindtype .Type $structs}})).(*{{bindtype .Type $structs}}){{end}}

			return *outstruct, err
			{{else}}
			if err != nil {
				return {{range $i, $_ := .Normalized.Outputs}}*new({{bindtype .Type $structs}}), {{end}} err
			}
			{{range $i, $t := .Normalized.Outputs}}
			out{{$i}} := *abi.ConvertType(out[{{$i}}], new({{bindtype .Type $structs}})).(*{{bindtype .Type $structs}}){{end}}

			return {{range $i, $t := .Normalized.Outputs}}out{{$i}}, {{end}} err
			{{end}}
		}

		// {{.Normalized.Name}} is a free data retrieval call binding the contract method 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Session) {{.Normalized.Name}}({{range $i, $_ := .Normalized.Inputs}}{{if ne $i 0}},{{end}} {{.Name}} {{bindtype .Type $structs}} {{end}}) ({{if .Structured}}struct{ {{range .Normalized.Outputs}}{{.Name}} {{bindtype .Type $structs}};{{end}} }, {{else}} {{range .Normalized.Outputs}}{{bindtype .Type $structs}},{{end}} {{end}} error) {
		  return _{{$contract.Type}}.Contract.{{.Normalized.Name}}(&_{{$contract.Type}}.CallOpts {{range .Normalized.Inputs}}, {{.Name}}{{end}})
		}

		// {{.Normalized.Name}} is a free data retrieval call binding the contract method 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}CallerSession) {{.Normalized.Name}}({{range $i, $_ := .Normalized.Inputs}}{{if ne $i 0}},{{end}} {{.Name}} {{bindtype .Type $structs}} {{end}}) ({{if .Structured}}struct{ {{range .Normalized.Outputs}}{{.Name}} {{bindtype .Type $structs}};{{end}} }, {{else}} {{range .Normalized.Outputs}}{{bindtype .Type $structs}},{{end}} {{end}} error) {
		  return _{{$contract.Type}}.Contract.{{.Normalized.Name}}(&_{{$contract.Type}}.CallOpts {{range .Normalized.Inputs}}, {{.Name}}{{end}})
		}
	{{end}}

	{{range .Transacts}}
		// {{.Normalized.Name}} is a paid mutator transaction binding the contract method 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Transactor) {{.Normalized.Name}}(opts *bind.TransactOpts {{range .Normalized.Inputs}}, {{.Name}} {{bindtype .Type $structs}} {{end}}) (*types.Transaction, error) {
			return _{{$contract.Type}}.contract.Transact(opts, "{{.Original.Name}}" {{range .Normalized.Inputs}}, {{.Name}}{{end}})
		}

		// {{.Normalized.Name}} is a paid mutator transaction binding the contract method 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Session) {{.Normalized.Name}}({{range $i, $_ := .Normalized.Inputs}}{{if ne $i 0}},{{end}} {{.Name}} {{bindtype .Type $structs}} {{end}}) (*types.Transaction, error) {
		  return _{{$contract.Type}}.Contract.{{.Normalized.Name}}(&_{{$contract.Type}}.TransactOpts {{range $i, $_ := .Normalized.Inputs}}, {{.Name}}{{end}})
		}

		// {{.Normalized.Name}} is a paid mutator transaction binding the contract method 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}TransactorSession) {{.Normalized.Name}}({{range $i, $_ := .Normalized.Inputs}}{{if ne $i 0}},{{end}} {{.Name}} {{bindtype .Type $structs}} {{end}}) (*types.Transaction, error) {
		  return _{{$contract.Type}}.Contract.{{.Normalized.Name}}(&_{{$contract.Type}}.TransactOpts {{range $i, $_ := .Normalized.Inputs}}, {{.Name}}{{end}})
		}
	{{end}}

	{{if .Fallback}}
		// Fallback is a paid mutator transaction binding the contract fallback function.
		//
		// Solidity: {{.Fallback.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Transactor) Fallback(opts *bind.TransactOpts, calldata []byte) (*types.Transaction, error) {
			return _{{$contract.Type}}.contract.RawTransact(opts, calldata)
		}

		// Fallback is a paid mutator transaction binding the contract fallback function.
		//
		// Solidity: {{.Fallback.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Session) Fallback(calldata []byte) (*types.Transaction, error) {
		  return _{{$contract.Type}}.Contract.Fallback(&_{{$contract.Type}}.TransactOpts, calldata)
		}

		// Fallback is a paid mutator transaction binding the contract fallback function.
		//
		// Solidity: {{.Fallback.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}TransactorSession) Fallback(calldata []byte) (*types.Transaction, error) {
		  return _{{$contract.Type}}.Contract.Fallback(&_{{$contract.Type}}.TransactOpts, calldata)
		}
	{{end}}

	{{if .Receive}}
		// Receive is a paid mutator transaction binding the contract receive function.
		//
		// Solidity: {{.Receive.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Transactor) Receive(opts *bind.TransactOpts) (*types.Transaction, error) {
			return _{{$contract.Type}}.contract.RawTransact(opts, nil) // calldata is disallowed for receive function
		}

		// Receive is a paid mutator transaction binding the contract receive function.
		//
		// Solidity: {{.Receive.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Session) Receive() (*types.Transaction, error) {
		  return _{{$contract.Type}}.Contract.Receive(&_{{$contract.Type}}.TransactOpts)
		}

		// Receive is a paid mutator transaction binding the contract receive function.
		//
		// Solidity: {{.Receive.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}TransactorSession) Receive() (*types.Transaction, error) {
		  return _{{$contract.Type}}.Contract.Receive(&_{{$contract.Type}}.TransactOpts)
		}
	{{end}}

	{{range .Events}}
		// {{$contract.Type}}{{.Normalized.Name}}Iterator is returned from Filter{{.Normalized.Name}} and is used to iterate over the raw logs and unpacked data for {{.Normalized.Name}} events raised by the {{$contract.Type}} contract.
		type {{$contract.Type}}{{.Normalized.Name}}Iterator struct {
			Event *{{$contract.Type}}{{.Normalized.Name}} // Event containing the contract specifics and raw log

			contract *bind.BoundContract // Generic contract to use for unpacking event data
			event    string              // Event name to use for unpacking event data

			logs chan types.Log        // Log channel receiving the found contract events
			sub  ethereum.Subscription // Subscription for errors, completion and termination
			done bool                  // Whether the subscription completed delivering logs
			fail error                 // Occurred error to stop iteration
		}
		// Next advances the iterator to the subsequent event, returning whether there
		// are any more events found. In case of a retrieval or parsing error, false is
		// returned and Error() can be queried for the exact failure.
		func (it *{{$contract.Type}}{{.Normalized.Name}}Iterator) Next() bool {
			// If the iterator failed, stop iterating
			if (it.fail != nil) {
				return false
			}
			// If the iterator completed, deliver directly whatever's available
			if (it.done) {
				select {
				case log := <-it.logs:
					it.Event = new({{$contract.Type}}{{.Normalized.Name}})
					if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
						it.fail = err
						return false
					}
					it.Event.Raw = log
					return true

				default:
					return false
				}
			}
			// Iterator still in progress, wait for either a data or an error event
			select {
			case log := <-it.logs:
				it.Event = new({{$contract.Type}}{{.Normalized.Name}})
				if err := it.contract.UnpackLog(it.Event, it.event, log); err != nil {
					it.fail = err
					return false
				}
				it.Event.Raw = log
				return true

			case err := <-it.sub.Err():
				it.done = true
				it.fail = err
				return it.Next()
			}
		}
		// Error returns any retrieval or parsing error occurred during filtering.
		func (it *{{$contract.Type}}{{.Normalized.Name}}Iterator) Error() error {
			return it.fail
		}
		// Close terminates the iteration process, releasing any pending underlying
		// resources.
		func (it *{{$contract.Type}}{{.Normalized.Name}}Iterator) Close() error {
			it.sub.Unsubscribe()
			return nil
		}

		// {{$contract.Type}}{{.Normalized.Name}} represents a {{.Normalized.Name}} event raised by the {{$contract.Type}} contract.
		type {{$contract.Type}}{{.Normalized.Name}} struct { {{range .Normalized.Inputs}}
			{{capitalise .Name}} {{if .Indexed}}{{bindtopictype .Type $structs}}{{else}}{{bindtype .Type $structs}}{{end}}; {{end}}
			Raw types.Log // Blockchain specific contextual infos
		}

		// Filter{{.Normalized.Name}} is a free log retrieval operation binding the contract event 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
 		func (_{{$contract.Type}} *{{$contract.Type}}Filterer) Filter{{.Normalized.Name}}(opts *bind.FilterOpts{{range .Normalized.Inputs}}{{if .Indexed}}, {{.Name}} []{{bindtype .Type $structs}}{{end}}{{end}}) (*{{$contract.Type}}{{.Normalized.Name}}Iterator, error) {
			{{range .Normalized.Inputs}}
			{{if .Indexed}}var {{.Name}}Rule []interface{}
			for _, {{.Name}}Item := range {{.Name}} {
				{{.Name}}Rule = append({{.Name}}Rule, {{.Name}}Item)
			}{{end}}{{end}}

			logs, sub, err := _{{$contract.Type}}.contract.FilterLogs(opts, "{{.Original.Name}}"{{range .Normalized.Inputs}}{{if .Indexed}}, {{.Name}}Rule{{end}}{{end}})
			if err != nil {
				return nil, err
			}
			return &{{$contract.Type}}{{.Normalized.Name}}Iterator{contract: _{{$contract.Type}}.contract, event: "{{.Original.Name}}", logs: logs, sub: sub}, nil
 		}

		// Watch{{.Normalized.Name}} is a free log subscription operation binding the contract event 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Filterer) Watch{{.Normalized.Name}}(opts *bind.WatchOpts, sink chan<- *{{$contract.Type}}{{.Normalized.Name}}{{range .Normalized.Inputs}}{{if .Indexed}}, {{.Name}} []{{bindtype .Type $structs}}{{end}}{{end}}) (event.Subscription, error) {
			{{range .Normalized.Inputs}}
			{{if .Indexed}}var {{.Name}}Rule []interface{}
			for _, {{.Name}}Item := range {{.Name}} {
				{{.Name}}Rule = append({{.Name}}Rule, {{.Name}}Item)
			}{{end}}{{end}}

			logs, sub, err := _{{$contract.Type}}.contract.WatchLogs(opts, "{{.Original.Name}}"{{range .Normalized.Inputs}}{{if .Indexed}}, {{.Name}}Rule{{end}}{{end}})
			if err != nil {
				return nil, err
			}
			return event.NewSubscription(func(quit <-chan struct{}) error {
				defer sub.Unsubscribe()
				for {
					select {
					case log := <-logs:
						// New log arrived, parse the event and forward to the user
						event := new({{$contract.Type}}{{.Normalized.Name}})
						if err := _{{$contract.Type}}.contract.UnpackLog(event, "{{.Original.Name}}", log); err != nil {
							return err
						}
						event.Raw = log

						select {
						case sink <- event:
						case err := <-sub.Err():
							return err
						case <-quit:
							return nil
						}
					case err := <-sub.Err():
						return err
					case <-quit:
						return nil
					}
				}
			}), nil
		}

		// Parse{{.Normalized.Name}} is a log parse operation binding the contract event 0x{{printf "%x" .Original.ID}}.
		//
		// Solidity: {{.Original.String}}
		func (_{{$contract.Type}} *{{$contract.Type}}Filterer) Parse{{.Normalized.Name}}(log types.Log) (*{{$contract.Type}}{{.Normalized.Name}}, error) {
			event := new({{$contract.Type}}{{.Normalized.Name}})
			if err := _{{$contract.Type}}.contract.UnpackLog(event, "{{.Original.Name}}", log); err != nil {
				return nil, err
			}
			event.Raw = log
			return event, nil
		}

 	{{end}}
{{end}}
`
