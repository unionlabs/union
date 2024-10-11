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

// evmdisTracer returns sufficient information from a trace to perform evmdis-style
// disassembly.
{
	stack: [{ops: []}],

	npushes: {0: 0, 1: 1, 2: 1, 3: 1, 4: 1, 5: 1, 6: 1, 7: 1, 8: 1, 9: 1, 10: 1, 11: 1, 16: 1, 17: 1, 18: 1, 19: 1, 20: 1, 21: 1, 22: 1, 23: 1, 24: 1, 25: 1, 26: 1, 32: 1, 48: 1, 49: 1, 50: 1, 51: 1, 52: 1, 53: 1, 54: 1, 55: 0, 56: 1, 57: 0, 58: 1, 59: 1, 60: 0, 64: 1, 65: 1, 66: 1, 67: 1, 68: 1, 69: 1, 80: 0, 81: 1, 82: 0, 83: 0, 84: 1, 85: 0, 86: 0, 87: 0, 88: 1, 89: 1, 90: 1, 91: 0, 96: 1, 97: 1, 98: 1, 99: 1, 100: 1, 101: 1, 102: 1, 103: 1, 104: 1, 105: 1, 106: 1, 107: 1, 108: 1, 109: 1, 110: 1, 111: 1, 112: 1, 113: 1, 114: 1, 115: 1, 116: 1, 117: 1, 118: 1, 119: 1, 120: 1, 121: 1, 122: 1, 123: 1, 124: 1, 125: 1, 126: 1, 127: 1, 128: 2, 129: 3, 130: 4, 131: 5, 132: 6, 133: 7, 134: 8, 135: 9, 136: 10, 137: 11, 138: 12, 139: 13, 140: 14, 141: 15, 142: 16, 143: 17, 144: 2, 145: 3, 146: 4, 147: 5, 148: 6, 149: 7, 150: 8, 151: 9, 152: 10, 153: 11, 154: 12, 155: 13, 156: 14, 157: 15, 158: 16, 159: 17, 160: 0, 161: 0, 162: 0, 163: 0, 164: 0, 240: 1, 241: 1, 242: 1, 243: 0, 244: 0, 255: 0},

	// result is invoked when all the opcodes have been iterated over and returns
	// the final result of the tracing.
	result: function() { return this.stack[0].ops; },

	// fault is invoked when the actual execution of an opcode fails.
	fault: function(log, db) { },

	// step is invoked for every opcode that the VM executes.
	step: function(log, db) {
		var frame = this.stack[this.stack.length - 1];

		var error = log.getError();
		if (error) {
			frame["error"] = error;
		} else if (log.getDepth() == this.stack.length) {
			opinfo = {
				op:     log.op.toNumber(),
				depth : log.getDepth(),
				result: [],
			};
			if (frame.ops.length > 0) {
				var prevop = frame.ops[frame.ops.length - 1];
				for(var i = 0; i < this.npushes[prevop.op]; i++)
					prevop.result.push(log.stack.peek(i).toString(16));
			}
			switch(log.op.toString()) {
			case "CALL": case "CALLCODE":
				var instart = log.stack.peek(3).valueOf();
				var insize = log.stack.peek(4).valueOf();
				opinfo["gas"] = log.stack.peek(0).valueOf();
				opinfo["to"] = log.stack.peek(1).toString(16);
				opinfo["value"] = log.stack.peek(2).toString();
				opinfo["input"] = log.memory.slice(instart, instart + insize);
				opinfo["error"] = null;
				opinfo["return"] = null;
				opinfo["ops"] = [];
				this.stack.push(opinfo);
				break;
			case "DELEGATECALL": case "STATICCALL":
				var instart = log.stack.peek(2).valueOf();
				var insize = log.stack.peek(3).valueOf();
				opinfo["op"] =  log.op.toString();
				opinfo["gas"] =  log.stack.peek(0).valueOf();
				opinfo["to"] =  log.stack.peek(1).toString(16);
				opinfo["input"] =  log.memory.slice(instart, instart + insize);
				opinfo["error"] =  null;
				opinfo["return"] =  null;
				opinfo["ops"] = [];
				this.stack.push(opinfo);
				break;
			case "RETURN": case "REVERT":
				var out = log.stack.peek(0).valueOf();
				var outsize = log.stack.peek(1).valueOf();
				frame.return = log.memory.slice(out, out + outsize);
				break;
			case "STOP": case "SELFDESTRUCT":
				frame.return = log.memory.slice(0, 0);
				break;
			case "JUMPDEST":
				opinfo["pc"] = log.getPC();
			}
			if(log.op.isPush()) {
				opinfo["len"] = log.op.toNumber() - 0x5e;
			}
			frame.ops.push(opinfo);
		} else {
			this.stack = this.stack.slice(0, log.getDepth());
		}
	}
}
