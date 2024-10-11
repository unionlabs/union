// Copyright 2021 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"reflect"
	"strings"
	"testing"

	"github.com/docker/docker/pkg/reexec"
	"github.com/ethereum/go-ethereum/cmd/evm/internal/t8ntool"
	"github.com/ethereum/go-ethereum/internal/cmdtest"
)

func TestMain(m *testing.M) {
	// Run the app if we've been exec'd as "ethkey-test" in runEthkey.
	reexec.Register("evm-test", func() {
		if err := app.Run(os.Args); err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		os.Exit(0)
	})
	// check if we have been reexec'd
	if reexec.Init() {
		return
	}
	os.Exit(m.Run())
}

type testT8n struct {
	*cmdtest.TestCmd
}

type t8nInput struct {
	inAlloc  string
	inTxs    string
	inEnv    string
	stFork   string
	stReward string
}

func (args *t8nInput) get(base string) []string {
	var out []string
	if opt := args.inAlloc; opt != "" {
		out = append(out, "--input.alloc")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.inTxs; opt != "" {
		out = append(out, "--input.txs")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.inEnv; opt != "" {
		out = append(out, "--input.env")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.stFork; opt != "" {
		out = append(out, "--state.fork", opt)
	}
	if opt := args.stReward; opt != "" {
		out = append(out, "--state.reward", opt)
	}
	return out
}

type t8nOutput struct {
	alloc  bool
	result bool
	body   bool
}

func (args *t8nOutput) get() (out []string) {
	if args.body {
		out = append(out, "--output.body", "stdout")
	} else {
		out = append(out, "--output.body", "") // empty means ignore
	}
	if args.result {
		out = append(out, "--output.result", "stdout")
	} else {
		out = append(out, "--output.result", "")
	}
	if args.alloc {
		out = append(out, "--output.alloc", "stdout")
	} else {
		out = append(out, "--output.alloc", "")
	}
	return out
}

func TestT8n(t *testing.T) {
	tt := new(testT8n)
	tt.TestCmd = cmdtest.NewTestCmd(t, tt)
	for i, tc := range []struct {
		base        string
		input       t8nInput
		output      t8nOutput
		expExitCode int
		expOut      string
	}{
		{ // Test exit (3) on bad config
			base: "./testdata/1",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Frontier+1346", "",
			},
			output:      t8nOutput{alloc: true, result: true},
			expExitCode: 3,
		},
		{
			base: "./testdata/1",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Byzantium", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // blockhash test
			base: "./testdata/3",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Berlin", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // missing blockhash test
			base: "./testdata/4",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Berlin", "",
			},
			output:      t8nOutput{alloc: true, result: true},
			expExitCode: 4,
		},
		{ // Uncle test
			base: "./testdata/5",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Byzantium", "0x80",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // Sign json transactions
			base: "./testdata/13",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "London", "",
			},
			output: t8nOutput{body: true},
			expOut: "exp.json",
		},
		{ // Already signed transactions
			base: "./testdata/13",
			input: t8nInput{
				"alloc.json", "signed_txs.rlp", "env.json", "London", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp2.json",
		},
		{ // Difficulty calculation - no uncles
			base: "./testdata/14",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "London", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp.json",
		},
		{ // Difficulty calculation - with uncles
			base: "./testdata/14",
			input: t8nInput{
				"alloc.json", "txs.json", "env.uncles.json", "London", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp2.json",
		},
		{ // Difficulty calculation - with ommers + Berlin
			base: "./testdata/14",
			input: t8nInput{
				"alloc.json", "txs.json", "env.uncles.json", "Berlin", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp_berlin.json",
		},
		{ // Difficulty calculation on arrow glacier
			base: "./testdata/19",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "London", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp_london.json",
		},
		{ // Difficulty calculation on arrow glacier
			base: "./testdata/19",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "ArrowGlacier", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp_arrowglacier.json",
		},
		{ // Difficulty calculation on gray glacier
			base: "./testdata/19",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "GrayGlacier", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp_grayglacier.json",
		},
		{ // Sign unprotected (pre-EIP155) transaction
			base: "./testdata/23",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Berlin", "",
			},
			output: t8nOutput{result: true},
			expOut: "exp.json",
		},
		{ // Test post-merge transition
			base: "./testdata/24",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Merge", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // Test post-merge transition where input is missing random
			base: "./testdata/24",
			input: t8nInput{
				"alloc.json", "txs.json", "env-missingrandom.json", "Merge", "",
			},
			output:      t8nOutput{alloc: false, result: false},
			expExitCode: 3,
		},
		{ // Test base fee calculation
			base: "./testdata/25",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Merge", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // Test withdrawals transition
			base: "./testdata/26",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Shanghai", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // Cancun tests
			base: "./testdata/28",
			input: t8nInput{
				"alloc.json", "txs.rlp", "env.json", "Cancun", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // More cancun tests
			base: "./testdata/29",
			input: t8nInput{
				"alloc.json", "txs.json", "env.json", "Cancun", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
		{ // More cancun test, plus example of rlp-transaction that cannot be decoded properly
			base: "./testdata/30",
			input: t8nInput{
				"alloc.json", "txs_more.rlp", "env.json", "Cancun", "",
			},
			output: t8nOutput{alloc: true, result: true},
			expOut: "exp.json",
		},
	} {
		args := []string{"t8n"}
		args = append(args, tc.output.get()...)
		args = append(args, tc.input.get(tc.base)...)
		var qArgs []string // quoted args for debugging purposes
		for _, arg := range args {
			if len(arg) == 0 {
				qArgs = append(qArgs, `""`)
			} else {
				qArgs = append(qArgs, arg)
			}
		}
		tt.Logf("args: %v\n", strings.Join(qArgs, " "))
		tt.Run("evm-test", args...)
		// Compare the expected output, if provided
		if tc.expOut != "" {
			file := fmt.Sprintf("%v/%v", tc.base, tc.expOut)
			want, err := os.ReadFile(file)
			if err != nil {
				t.Fatalf("test %d: could not read expected output: %v", i, err)
			}
			have := tt.Output()
			ok, err := cmpJson(have, want)
			switch {
			case err != nil:
				t.Fatalf("test %d, file %v: json parsing failed: %v", i, file, err)
			case !ok:
				t.Fatalf("test %d, file %v: output wrong, have \n%v\nwant\n%v\n", i, file, string(have), string(want))
			}
		}
		tt.WaitExit()
		if have, want := tt.ExitStatus(), tc.expExitCode; have != want {
			t.Fatalf("test %d: wrong exit code, have %d, want %d", i, have, want)
		}
	}
}

type t9nInput struct {
	inTxs  string
	stFork string
}

func (args *t9nInput) get(base string) []string {
	var out []string
	if opt := args.inTxs; opt != "" {
		out = append(out, "--input.txs")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.stFork; opt != "" {
		out = append(out, "--state.fork", opt)
	}
	return out
}

func TestT9n(t *testing.T) {
	tt := new(testT8n)
	tt.TestCmd = cmdtest.NewTestCmd(t, tt)
	for i, tc := range []struct {
		base        string
		input       t9nInput
		expExitCode int
		expOut      string
	}{
		{ // London txs on homestead
			base: "./testdata/15",
			input: t9nInput{
				inTxs:  "signed_txs.rlp",
				stFork: "Homestead",
			},
			expOut: "exp.json",
		},
		{ // London txs on London
			base: "./testdata/15",
			input: t9nInput{
				inTxs:  "signed_txs.rlp",
				stFork: "London",
			},
			expOut: "exp2.json",
		},
		{ // An RLP list (a blockheader really)
			base: "./testdata/15",
			input: t9nInput{
				inTxs:  "blockheader.rlp",
				stFork: "London",
			},
			expOut: "exp3.json",
		},
		{ // Transactions with too low gas
			base: "./testdata/16",
			input: t9nInput{
				inTxs:  "signed_txs.rlp",
				stFork: "London",
			},
			expOut: "exp.json",
		},
		{ // Transactions with value exceeding 256 bits
			base: "./testdata/17",
			input: t9nInput{
				inTxs:  "signed_txs.rlp",
				stFork: "London",
			},
			expOut: "exp.json",
		},
		{ // Invalid RLP
			base: "./testdata/18",
			input: t9nInput{
				inTxs:  "invalid.rlp",
				stFork: "London",
			},
			expExitCode: t8ntool.ErrorIO,
		},
	} {
		args := []string{"t9n"}
		args = append(args, tc.input.get(tc.base)...)

		tt.Run("evm-test", args...)
		tt.Logf("args:\n go run . %v\n", strings.Join(args, " "))
		// Compare the expected output, if provided
		if tc.expOut != "" {
			want, err := os.ReadFile(fmt.Sprintf("%v/%v", tc.base, tc.expOut))
			if err != nil {
				t.Fatalf("test %d: could not read expected output: %v", i, err)
			}
			have := tt.Output()
			ok, err := cmpJson(have, want)
			switch {
			case err != nil:
				t.Logf(string(have))
				t.Fatalf("test %d, json parsing failed: %v", i, err)
			case !ok:
				t.Fatalf("test %d: output wrong, have \n%v\nwant\n%v\n", i, string(have), string(want))
			}
		}
		tt.WaitExit()
		if have, want := tt.ExitStatus(), tc.expExitCode; have != want {
			t.Fatalf("test %d: wrong exit code, have %d, want %d", i, have, want)
		}
	}
}

type b11rInput struct {
	inEnv         string
	inOmmersRlp   string
	inWithdrawals string
	inTxsRlp      string
	inClique      string
	ethash        bool
	ethashMode    string
	ethashDir     string
}

func (args *b11rInput) get(base string) []string {
	var out []string
	if opt := args.inEnv; opt != "" {
		out = append(out, "--input.header")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.inOmmersRlp; opt != "" {
		out = append(out, "--input.ommers")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.inWithdrawals; opt != "" {
		out = append(out, "--input.withdrawals")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.inTxsRlp; opt != "" {
		out = append(out, "--input.txs")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.inClique; opt != "" {
		out = append(out, "--seal.clique")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if args.ethash {
		out = append(out, "--seal.ethash")
	}
	if opt := args.ethashMode; opt != "" {
		out = append(out, "--seal.ethash.mode")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	if opt := args.ethashDir; opt != "" {
		out = append(out, "--seal.ethash.dir")
		out = append(out, fmt.Sprintf("%v/%v", base, opt))
	}
	out = append(out, "--output.block")
	out = append(out, "stdout")
	return out
}

func TestB11r(t *testing.T) {
	tt := new(testT8n)
	tt.TestCmd = cmdtest.NewTestCmd(t, tt)
	for i, tc := range []struct {
		base        string
		input       b11rInput
		expExitCode int
		expOut      string
	}{
		{ // unsealed block
			base: "./testdata/20",
			input: b11rInput{
				inEnv:       "header.json",
				inOmmersRlp: "ommers.json",
				inTxsRlp:    "txs.rlp",
			},
			expOut: "exp.json",
		},
		{ // ethash test seal
			base: "./testdata/21",
			input: b11rInput{
				inEnv:       "header.json",
				inOmmersRlp: "ommers.json",
				inTxsRlp:    "txs.rlp",
			},
			expOut: "exp.json",
		},
		{ // clique test seal
			base: "./testdata/21",
			input: b11rInput{
				inEnv:       "header.json",
				inOmmersRlp: "ommers.json",
				inTxsRlp:    "txs.rlp",
				inClique:    "clique.json",
			},
			expOut: "exp-clique.json",
		},
		{ // block with ommers
			base: "./testdata/22",
			input: b11rInput{
				inEnv:       "header.json",
				inOmmersRlp: "ommers.json",
				inTxsRlp:    "txs.rlp",
			},
			expOut: "exp.json",
		},
		{ // block with withdrawals
			base: "./testdata/27",
			input: b11rInput{
				inEnv:         "header.json",
				inOmmersRlp:   "ommers.json",
				inWithdrawals: "withdrawals.json",
				inTxsRlp:      "txs.rlp",
			},
			expOut: "exp.json",
		},
	} {
		args := []string{"b11r"}
		args = append(args, tc.input.get(tc.base)...)

		tt.Run("evm-test", args...)
		tt.Logf("args:\n go run . %v\n", strings.Join(args, " "))
		// Compare the expected output, if provided
		if tc.expOut != "" {
			want, err := os.ReadFile(fmt.Sprintf("%v/%v", tc.base, tc.expOut))
			if err != nil {
				t.Fatalf("test %d: could not read expected output: %v", i, err)
			}
			have := tt.Output()
			ok, err := cmpJson(have, want)
			switch {
			case err != nil:
				t.Logf(string(have))
				t.Fatalf("test %d, json parsing failed: %v", i, err)
			case !ok:
				t.Fatalf("test %d: output wrong, have \n%v\nwant\n%v\n", i, string(have), string(want))
			}
		}
		tt.WaitExit()
		if have, want := tt.ExitStatus(), tc.expExitCode; have != want {
			t.Fatalf("test %d: wrong exit code, have %d, want %d", i, have, want)
		}
	}
}

// cmpJson compares the JSON in two byte slices.
func cmpJson(a, b []byte) (bool, error) {
	var j, j2 interface{}
	if err := json.Unmarshal(a, &j); err != nil {
		return false, err
	}
	if err := json.Unmarshal(b, &j2); err != nil {
		return false, err
	}
	return reflect.DeepEqual(j2, j), nil
}
