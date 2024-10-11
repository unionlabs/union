//go:build integrationtests

// Copyright 2023 The go-ethereum Authors
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
	"fmt"
	"math"
	"math/big"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/log"
	"github.com/holiman/uint256"
	"github.com/urfave/cli/v2"
)

var logTestCommand = &cli.Command{
	Action:    logTest,
	Name:      "logtest",
	Usage:     "Print some log messages",
	ArgsUsage: " ",
	Description: `
This command is only meant for testing.
`}

// logTest is an entry point which spits out some logs. This is used by testing
// to verify expected outputs
func logTest(ctx *cli.Context) error {
	log.ResetGlobalState()
	{ // big.Int
		ba, _ := new(big.Int).SetString("111222333444555678999", 10)    // "111,222,333,444,555,678,999"
		bb, _ := new(big.Int).SetString("-111222333444555678999", 10)   // "-111,222,333,444,555,678,999"
		bc, _ := new(big.Int).SetString("11122233344455567899900", 10)  // "11,122,233,344,455,567,899,900"
		bd, _ := new(big.Int).SetString("-11122233344455567899900", 10) // "-11,122,233,344,455,567,899,900"
		log.Info("big.Int", "111,222,333,444,555,678,999", ba)
		log.Info("-big.Int", "-111,222,333,444,555,678,999", bb)
		log.Info("big.Int", "11,122,233,344,455,567,899,900", bc)
		log.Info("-big.Int", "-11,122,233,344,455,567,899,900", bd)
	}
	{ //uint256
		ua, _ := uint256.FromDecimal("111222333444555678999")
		ub, _ := uint256.FromDecimal("11122233344455567899900")
		log.Info("uint256", "111,222,333,444,555,678,999", ua)
		log.Info("uint256", "11,122,233,344,455,567,899,900", ub)
	}
	{ // int64
		log.Info("int64", "1,000,000", int64(1000000))
		log.Info("int64", "-1,000,000", int64(-1000000))
		log.Info("int64", "9,223,372,036,854,775,807", int64(math.MaxInt64))
		log.Info("int64", "-9,223,372,036,854,775,808", int64(math.MinInt64))
	}
	{ // uint64
		log.Info("uint64", "1,000,000", uint64(1000000))
		log.Info("uint64", "18,446,744,073,709,551,615", uint64(math.MaxUint64))
	}
	{ // Special characters
		log.Info("Special chars in value", "key", "special \r\n\t chars")
		log.Info("Special chars in key", "special \n\t chars", "value")

		log.Info("nospace", "nospace", "nospace")
		log.Info("with space", "with nospace", "with nospace")

		log.Info("Bash escapes in value", "key", "\u001b[1G\u001b[K\u001b[1A")
		log.Info("Bash escapes in key", "\u001b[1G\u001b[K\u001b[1A", "value")

		log.Info("Bash escapes in message  \u001b[1G\u001b[K\u001b[1A end", "key", "value")

		colored := fmt.Sprintf("\u001B[%dmColored\u001B[0m[", 35)
		log.Info(colored, colored, colored)
	}
	{ // Custom Stringer() - type
		log.Info("Custom Stringer value", "2562047h47m16.854s", common.PrettyDuration(time.Duration(9223372036854775807)))
	}
	{ // Lazy eval
		log.Info("Lazy evaluation of value", "key", log.Lazy{Fn: func() interface{} { return "lazy value" }})
	}
	{ // Multi-line message
		log.Info("A message with wonky \U0001F4A9 characters")
		log.Info("A multiline message \nINFO [10-18|14:11:31.106] with wonky characters \U0001F4A9")
		log.Info("A multiline message \nLALA [ZZZZZZZZZZZZZZZZZZ] Actually part of message above")
	}
	{ // Miscellaneous json-quirks
		// This will check if the json output uses strings or json-booleans to represent bool values
		log.Info("boolean", "true", true, "false", false)
		// Handling of duplicate keys.
		// This is actually ill-handled by the current handler: the format.go
		// uses a global 'fieldPadding' map and mixes up the two keys. If 'alpha'
		// is shorter than beta, it sometimes causes erroneous padding -- and what's more
		// it causes _different_ padding in multi-handler context, e.g. both file-
		// and console output, making the two mismatch.
		log.Info("repeated-key 1", "foo", "alpha", "foo", "beta")
		log.Info("repeated-key 2", "xx", "short", "xx", "longer")
	}
	{ // loglevels
		log.Debug("log at level debug")
		log.Trace("log at level trace")
		log.Info("log at level info")
		log.Warn("log at level warn")
		log.Error("log at level error")
	}
	{
		// The current log formatter has a global map of paddings, storing the
		// longest seen padding per key in a map. This results in a statefulness
		// which has some odd side-effects. Demonstrated here:
		log.Info("test", "bar", "short", "a", "aligned left")
		log.Info("test", "bar", "a long message", "a", 1)
		log.Info("test", "bar", "short", "a", "aligned right")
	}
	{
		// This sequence of logs should be output with alignment, so each field becoems a column.
		log.Info("The following logs should align so that the key-fields make 5 columns")
		log.Info("Inserted known block", "number", 1_012, "hash", common.HexToHash("0x1234"), "txs", 200, "gas", 1_123_123, "other", "first")
		log.Info("Inserted new block", "number", 1, "hash", common.HexToHash("0x1235"), "txs", 2, "gas", 1_123, "other", "second")
		log.Info("Inserted known block", "number", 99, "hash", common.HexToHash("0x12322"), "txs", 10, "gas", 1, "other", "third")
		log.Warn("Inserted known block", "number", 1_012, "hash", common.HexToHash("0x1234"), "txs", 200, "gas", 99, "other", "fourth")
	}
	return nil
}
