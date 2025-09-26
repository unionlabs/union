use alloy::primitives::Bytes;
use alloy_sol_types::SolValue as _;
use ucs03_zkgm::com::{Call, Instruction, INSTR_VERSION_0, OP_BATCH, OP_CALL};

pub fn make_batch(instructions: Vec<Instruction>) -> Instruction {
    Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_BATCH,
        operand: instructions.abi_encode_params().into(),
    }
}

pub fn make_call(sender: Bytes, contract_address: Bytes, contract_calldata: Bytes) -> Instruction {
    Instruction {
        version: INSTR_VERSION_0,
        opcode: OP_CALL,
        operand: Call {
            sender,
            eureka: false,
            contract_address,
            contract_calldata,
        }
        .abi_encode_params()
        .into(),
    }
}
