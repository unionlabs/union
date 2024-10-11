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

// prestateTracer outputs sufficient information to create a local execution of
// the transaction from a custom assembled genesis block.
{
	// prestate is the genesis that we're building.
	prestate: null,

	// lookupAccount injects the specified account into the prestate object.
	lookupAccount: function(addr, db){
		var acc = toHex(addr);
		if (this.prestate[acc] === undefined) {
			this.prestate[acc] = {
				balance: '0x' + db.getBalance(addr).toString(16),
				nonce:   db.getNonce(addr),
				code:    toHex(db.getCode(addr)),
				storage: {}
			};
		}
	},

	// lookupStorage injects the specified storage entry of the given account into
	// the prestate object.
	lookupStorage: function(addr, key, db){
		var acc = toHex(addr);
		var idx = toHex(key);

		if (this.prestate[acc].storage[idx] === undefined) {
			this.prestate[acc].storage[idx] = toHex(db.getState(addr, key));
		}
	},

	// result is invoked when all the opcodes have been iterated over and returns
	// the final result of the tracing.
	result: function(ctx, db) {
		if (this.prestate === null) {
			this.prestate = {};
			// If tx is transfer-only, the recipient account
			// hasn't been populated.
			this.lookupAccount(ctx.to, db);
		}

		// At this point, we need to deduct the 'value' from the
		// outer transaction, and move it back to the origin
		this.lookupAccount(ctx.from, db);

		var fromBal = bigInt(this.prestate[toHex(ctx.from)].balance.slice(2), 16);
		var toBal   = bigInt(this.prestate[toHex(ctx.to)].balance.slice(2), 16);

		this.prestate[toHex(ctx.to)].balance   = '0x'+toBal.subtract(ctx.value).toString(16);
		this.prestate[toHex(ctx.from)].balance = '0x'+fromBal.add(ctx.value).add(ctx.gasUsed * ctx.gasPrice).toString(16);

		// Decrement the caller's nonce, and remove empty create targets
		this.prestate[toHex(ctx.from)].nonce--;
		if (ctx.type == 'CREATE') {
			// We can blibdly delete the contract prestate, as any existing state would
			// have caused the transaction to be rejected as invalid in the first place.
			delete this.prestate[toHex(ctx.to)];
		}
		// Return the assembled allocations (prestate)
		return this.prestate;
	},

	// step is invoked for every opcode that the VM executes.
	step: function(log, db) {
		// Add the current account if we just started tracing
		if (this.prestate === null){
			this.prestate = {};
			// Balance will potentially be wrong here, since this will include the value
			// sent along with the message. We fix that in 'result()'.
			this.lookupAccount(log.contract.getAddress(), db);
		}
		// Whenever new state is accessed, add it to the prestate
		switch (log.op.toString()) {
			case "EXTCODECOPY": case "EXTCODESIZE": case "EXTCODEHASH": case "BALANCE":
				this.lookupAccount(toAddress(log.stack.peek(0).toString(16)), db);
				break;
			case "CREATE":
				var from = log.contract.getAddress();
				this.lookupAccount(toContract(from, db.getNonce(from)), db);
				break;
			case "CREATE2":
				var from = log.contract.getAddress();
				// stack: salt, size, offset, endowment
				var offset = log.stack.peek(1).valueOf()
				var size = log.stack.peek(2).valueOf()
				var end = offset + size
				this.lookupAccount(toContract2(from, log.stack.peek(3).toString(16), log.memory.slice(offset, end)), db);
				break;
			case "CALL": case "CALLCODE": case "DELEGATECALL": case "STATICCALL":
				this.lookupAccount(toAddress(log.stack.peek(1).toString(16)), db);
				break;
			case 'SSTORE':case 'SLOAD':
				this.lookupStorage(log.contract.getAddress(), toWord(log.stack.peek(0).toString(16)), db);
				break;
		}
	},

	// fault is invoked when the actual execution of an opcode fails.
	fault: function(log, db) {}
}
