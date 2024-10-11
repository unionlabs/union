// Copyright 2017 The go-ethereum Authors
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

// 4byteTracer searches for 4byte-identifiers, and collects them for post-processing.
// It collects the methods identifiers along with the size of the supplied data, so
// a reversed signature can be matched against the size of the data.
//
// Example:
//   > debug.traceTransaction( "0x214e597e35da083692f5386141e69f47e973b2c56e7a8073b1ea08fd7571e9de", {tracer: "4byteTracer"})
//   {
//     0x27dc297e-128: 1,
//     0x38cc4831-0: 2,
//     0x524f3889-96: 1,
//     0xadf59f99-288: 1,
//     0xc281d19e-0: 1
//   }
{
	// ids aggregates the 4byte ids found.
	ids : {},

	// callType returns 'false' for non-calls, or the peek-index for the first param
	// after 'value', i.e. meminstart.
	callType: function(opstr){
		switch(opstr){
		case "CALL": case "CALLCODE":
			// gas, addr, val, memin, meminsz, memout, memoutsz
			return 3; // stack ptr to memin

		case "DELEGATECALL": case "STATICCALL":
			// gas, addr, memin, meminsz, memout, memoutsz
			return 2; // stack ptr to memin
		}
		return false;
	},

	// store save the given identifier and datasize.
	store: function(id, size){
		var key = "" + toHex(id) + "-" + size;
		this.ids[key] = this.ids[key] + 1 || 1;
	},

	// step is invoked for every opcode that the VM executes.
	step: function(log, db) {
		// Skip any opcodes that are not internal calls
		var ct = this.callType(log.op.toString());
		if (!ct) {
			return;
		}
		// Skip any pre-compile invocations, those are just fancy opcodes
		if (isPrecompiled(toAddress(log.stack.peek(1).toString(16)))) {
			return;
		}
		// Gather internal call details
		var inSz = log.stack.peek(ct + 1).valueOf();
		if (inSz >= 4) {
			var inOff = log.stack.peek(ct).valueOf();
			this.store(log.memory.slice(inOff, inOff + 4), inSz-4);
		}
	},

	// fault is invoked when the actual execution of an opcode fails.
	fault: function(log, db) { },

	// result is invoked when all the opcodes have been iterated over and returns
	// the final result of the tracing.
	result: function(ctx) {
		// Save the outer calldata also
		if (ctx.input.length >= 4) {
			this.store(slice(ctx.input, 0, 4), ctx.input.length-4)
		}
		return this.ids;
	},
}
