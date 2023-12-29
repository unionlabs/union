use crate::cdsl::operands::{OperandKind, OperandKindFields};

/// Small helper to initialize an OperandBuilder with the right kind, for a given name and doc.
fn new(format_field_name: &'static str, rust_type: &'static str, doc: &'static str) -> OperandKind {
    OperandKind::new(
        format_field_name,
        rust_type,
        OperandKindFields::EntityRef,
        doc,
    )
}

pub(crate) struct EntityRefs {
    /// A reference to a basic block in the same function.
    /// This is primarliy used in control flow instructions.
    pub(crate) block: OperandKind,

    /// A reference to a stack slot declared in the function preamble.
    pub(crate) stack_slot: OperandKind,

    /// A reference to a dynamic_stack slot declared in the function preamble.
    pub(crate) dynamic_stack_slot: OperandKind,

    /// A reference to a global value.
    pub(crate) global_value: OperandKind,

    /// A reference to a function signature declared in the function preamble.
    /// This is used to provide the call signature in a call_indirect instruction.
    pub(crate) sig_ref: OperandKind,

    /// A reference to an external function declared in the function preamble.
    /// This is used to provide the callee and signature in a call instruction.
    pub(crate) func_ref: OperandKind,

    /// A reference to a jump table declared in the function preamble.
    pub(crate) jump_table: OperandKind,

    /// A reference to a heap declared in the function preamble.
    pub(crate) heap: OperandKind,

    /// A reference to a table declared in the function preamble.
    pub(crate) table: OperandKind,

    /// A variable-sized list of value operands. Use for Block and function call arguments.
    pub(crate) varargs: OperandKind,
}

impl EntityRefs {
    pub fn new() -> Self {
        Self {
            block: new(
                "destination",
                "ir::Block",
                "a basic block in the same function.",
            ),
            stack_slot: new("stack_slot", "ir::StackSlot", "A stack slot"),

            dynamic_stack_slot: new(
                "dynamic_stack_slot",
                "ir::DynamicStackSlot",
                "A dynamic stack slot",
            ),

            global_value: new("global_value", "ir::GlobalValue", "A global value."),

            sig_ref: new("sig_ref", "ir::SigRef", "A function signature."),

            func_ref: new("func_ref", "ir::FuncRef", "An external function."),

            jump_table: new("table", "ir::JumpTable", "A jump table."),

            heap: new("heap", "ir::Heap", "A heap."),

            table: new("table", "ir::Table", "A table."),

            varargs: OperandKind::new(
                "",
                "&[Value]",
                OperandKindFields::VariableArgs,
                r#"
                        A variable size list of `value` operands.

                        Use this to represent arguments passed to a function call, arguments
                        passed to a basic block, or a variable number of results
                        returned from an instruction.
                    "#,
            ),
        }
    }
}
