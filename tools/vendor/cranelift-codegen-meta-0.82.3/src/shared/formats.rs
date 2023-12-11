use crate::cdsl::formats::{InstructionFormat, InstructionFormatBuilder as Builder};
use crate::shared::{entities::EntityRefs, immediates::Immediates};
use std::rc::Rc;

pub(crate) struct Formats {
    pub(crate) atomic_cas: Rc<InstructionFormat>,
    pub(crate) atomic_rmw: Rc<InstructionFormat>,
    pub(crate) binary: Rc<InstructionFormat>,
    pub(crate) binary_imm8: Rc<InstructionFormat>,
    pub(crate) binary_imm64: Rc<InstructionFormat>,
    pub(crate) branch: Rc<InstructionFormat>,
    pub(crate) branch_float: Rc<InstructionFormat>,
    pub(crate) branch_icmp: Rc<InstructionFormat>,
    pub(crate) branch_int: Rc<InstructionFormat>,
    pub(crate) branch_table: Rc<InstructionFormat>,
    pub(crate) call: Rc<InstructionFormat>,
    pub(crate) call_indirect: Rc<InstructionFormat>,
    pub(crate) cond_trap: Rc<InstructionFormat>,
    pub(crate) float_compare: Rc<InstructionFormat>,
    pub(crate) float_cond: Rc<InstructionFormat>,
    pub(crate) float_cond_trap: Rc<InstructionFormat>,
    pub(crate) func_addr: Rc<InstructionFormat>,
    pub(crate) heap_addr: Rc<InstructionFormat>,
    pub(crate) int_compare: Rc<InstructionFormat>,
    pub(crate) int_compare_imm: Rc<InstructionFormat>,
    pub(crate) int_cond: Rc<InstructionFormat>,
    pub(crate) int_cond_trap: Rc<InstructionFormat>,
    pub(crate) int_select: Rc<InstructionFormat>,
    pub(crate) jump: Rc<InstructionFormat>,
    pub(crate) load: Rc<InstructionFormat>,
    pub(crate) load_complex: Rc<InstructionFormat>,
    pub(crate) load_no_offset: Rc<InstructionFormat>,
    pub(crate) multiary: Rc<InstructionFormat>,
    pub(crate) nullary: Rc<InstructionFormat>,
    pub(crate) shuffle: Rc<InstructionFormat>,
    pub(crate) stack_load: Rc<InstructionFormat>,
    pub(crate) stack_store: Rc<InstructionFormat>,
    pub(crate) store: Rc<InstructionFormat>,
    pub(crate) store_complex: Rc<InstructionFormat>,
    pub(crate) store_no_offset: Rc<InstructionFormat>,
    pub(crate) table_addr: Rc<InstructionFormat>,
    pub(crate) ternary: Rc<InstructionFormat>,
    pub(crate) ternary_imm8: Rc<InstructionFormat>,
    pub(crate) trap: Rc<InstructionFormat>,
    pub(crate) unary: Rc<InstructionFormat>,
    pub(crate) unary_bool: Rc<InstructionFormat>,
    pub(crate) unary_const: Rc<InstructionFormat>,
    pub(crate) unary_global_value: Rc<InstructionFormat>,
    pub(crate) unary_ieee32: Rc<InstructionFormat>,
    pub(crate) unary_ieee64: Rc<InstructionFormat>,
    pub(crate) unary_imm: Rc<InstructionFormat>,
}

impl Formats {
    pub fn new(imm: &Immediates, entities: &EntityRefs) -> Self {
        Self {
            unary: Builder::new("Unary").value().build(),

            unary_imm: Builder::new("UnaryImm").imm(&imm.imm64).build(),

            unary_ieee32: Builder::new("UnaryIeee32").imm(&imm.ieee32).build(),

            unary_ieee64: Builder::new("UnaryIeee64").imm(&imm.ieee64).build(),

            unary_bool: Builder::new("UnaryBool").imm(&imm.boolean).build(),

            unary_const: Builder::new("UnaryConst").imm(&imm.pool_constant).build(),

            unary_global_value: Builder::new("UnaryGlobalValue")
                .imm(&entities.global_value)
                .build(),

            binary: Builder::new("Binary").value().value().build(),

            binary_imm8: Builder::new("BinaryImm8").value().imm(&imm.uimm8).build(),

            binary_imm64: Builder::new("BinaryImm64").value().imm(&imm.imm64).build(),

            // The select instructions are controlled by the second VALUE operand.
            // The first VALUE operand is the controlling flag which has a derived type.
            // The fma instruction has the same constraint on all inputs.
            ternary: Builder::new("Ternary")
                .value()
                .value()
                .value()
                .typevar_operand(1)
                .build(),

            ternary_imm8: Builder::new("TernaryImm8")
                .value()
                .imm(&imm.uimm8)
                .value()
                .build(),

            // Catch-all for instructions with many outputs and inputs and no immediate
            // operands.
            multiary: Builder::new("MultiAry").varargs().build(),

            nullary: Builder::new("NullAry").build(),

            shuffle: Builder::new("Shuffle")
                .value()
                .value()
                .imm(&imm.uimm128)
                .build(),

            int_compare: Builder::new("IntCompare")
                .imm(&imm.intcc)
                .value()
                .value()
                .build(),

            int_compare_imm: Builder::new("IntCompareImm")
                .imm(&imm.intcc)
                .value()
                .imm(&imm.imm64)
                .build(),

            int_cond: Builder::new("IntCond").imm(&imm.intcc).value().build(),

            float_compare: Builder::new("FloatCompare")
                .imm(&imm.floatcc)
                .value()
                .value()
                .build(),

            float_cond: Builder::new("FloatCond").imm(&imm.floatcc).value().build(),

            int_select: Builder::new("IntSelect")
                .imm(&imm.intcc)
                .value()
                .value()
                .value()
                .build(),

            jump: Builder::new("Jump").imm(&entities.block).varargs().build(),

            branch: Builder::new("Branch")
                .value()
                .imm(&entities.block)
                .varargs()
                .build(),

            branch_int: Builder::new("BranchInt")
                .imm(&imm.intcc)
                .value()
                .imm(&entities.block)
                .varargs()
                .build(),

            branch_float: Builder::new("BranchFloat")
                .imm(&imm.floatcc)
                .value()
                .imm(&entities.block)
                .varargs()
                .build(),

            branch_icmp: Builder::new("BranchIcmp")
                .imm(&imm.intcc)
                .value()
                .value()
                .imm(&entities.block)
                .varargs()
                .build(),

            branch_table: Builder::new("BranchTable")
                .value()
                .imm(&entities.block)
                .imm(&entities.jump_table)
                .build(),

            call: Builder::new("Call")
                .imm(&entities.func_ref)
                .varargs()
                .build(),

            call_indirect: Builder::new("CallIndirect")
                .imm(&entities.sig_ref)
                .value()
                .varargs()
                .build(),

            func_addr: Builder::new("FuncAddr").imm(&entities.func_ref).build(),

            atomic_rmw: Builder::new("AtomicRmw")
                .imm(&imm.memflags)
                .imm(&imm.atomic_rmw_op)
                .value()
                .value()
                .build(),

            atomic_cas: Builder::new("AtomicCas")
                .imm(&imm.memflags)
                .value()
                .value()
                .value()
                .typevar_operand(2)
                .build(),

            load: Builder::new("Load")
                .imm(&imm.memflags)
                .value()
                .imm(&imm.offset32)
                .build(),

            load_complex: Builder::new("LoadComplex")
                .imm(&imm.memflags)
                .varargs()
                .imm(&imm.offset32)
                .build(),

            load_no_offset: Builder::new("LoadNoOffset")
                .imm(&imm.memflags)
                .value()
                .build(),

            store: Builder::new("Store")
                .imm(&imm.memflags)
                .value()
                .value()
                .imm(&imm.offset32)
                .build(),

            store_complex: Builder::new("StoreComplex")
                .imm(&imm.memflags)
                .value()
                .varargs()
                .imm(&imm.offset32)
                .build(),

            store_no_offset: Builder::new("StoreNoOffset")
                .imm(&imm.memflags)
                .value()
                .value()
                .build(),

            stack_load: Builder::new("StackLoad")
                .imm(&entities.stack_slot)
                .imm(&imm.offset32)
                .build(),

            stack_store: Builder::new("StackStore")
                .value()
                .imm(&entities.stack_slot)
                .imm(&imm.offset32)
                .build(),

            // Accessing a WebAssembly heap.
            heap_addr: Builder::new("HeapAddr")
                .imm(&entities.heap)
                .value()
                .imm(&imm.uimm32)
                .build(),

            // Accessing a WebAssembly table.
            table_addr: Builder::new("TableAddr")
                .imm(&entities.table)
                .value()
                .imm(&imm.offset32)
                .build(),

            trap: Builder::new("Trap").imm(&imm.trapcode).build(),

            cond_trap: Builder::new("CondTrap").value().imm(&imm.trapcode).build(),

            int_cond_trap: Builder::new("IntCondTrap")
                .imm(&imm.intcc)
                .value()
                .imm(&imm.trapcode)
                .build(),

            float_cond_trap: Builder::new("FloatCondTrap")
                .imm(&imm.floatcc)
                .value()
                .imm(&imm.trapcode)
                .build(),
        }
    }
}
