use crate::SourceLoc;

/// Description of a frame in a backtrace.
///
/// Each runtime error includes a backtrace of the WebAssembly frames that led
/// to the trap, and each frame is described by this structure.
#[derive(Debug, Clone)]
pub struct FrameInfo {
    /// The name of the module
    module_name: String,
    /// The index of the function in the module
    func_index: u32,
    /// The function name, if one is available.
    function_name: Option<String>,
    /// The source location of the function
    func_start: SourceLoc,
    /// The source location of the instruction
    instr: SourceLoc,
}

impl FrameInfo {
    /// Creates a new [FrameInfo], useful for testing.
    pub fn new(
        module_name: String,
        func_index: u32,
        function_name: Option<String>,
        func_start: SourceLoc,
        instr: SourceLoc,
    ) -> Self {
        Self {
            module_name,
            func_index,
            function_name,
            func_start,
            instr,
        }
    }

    /// Returns the WebAssembly function index for this frame.
    ///
    /// This function index is the index in the function index space of the
    /// WebAssembly module that this frame comes from.
    pub fn func_index(&self) -> u32 {
        self.func_index
    }

    /// Returns the identifer of the module that this frame is for.
    ///
    /// ModuleInfo identifiers are present in the `name` section of a WebAssembly
    /// binary, but this may not return the exact item in the `name` section.
    /// ModuleInfo names can be overwritten at construction time or perhaps inferred
    /// from file names. The primary purpose of this function is to assist in
    /// debugging and therefore may be tweaked over time.
    ///
    /// This function returns `None` when no name can be found or inferred.
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    /// Returns a descriptive name of the function for this frame, if one is
    /// available.
    ///
    /// The name of this function may come from the `name` section of the
    /// WebAssembly binary, or wasmer may try to infer a better name for it if
    /// not available, for example the name of the export if it's exported.
    ///
    /// This return value is primarily used for debugging and human-readable
    /// purposes for things like traps. Note that the exact return value may be
    /// tweaked over time here and isn't guaranteed to be something in
    /// particular about a wasm module due to its primary purpose of assisting
    /// in debugging.
    ///
    /// This function returns `None` when no name could be inferred.
    pub fn function_name(&self) -> Option<&str> {
        self.function_name.as_deref()
    }

    /// Returns the offset within the original wasm module this frame's program
    /// counter was at.
    ///
    /// The offset here is the offset from the beginning of the original wasm
    /// module to the instruction that this frame points to.
    pub fn module_offset(&self) -> usize {
        self.instr.bits() as usize
    }

    /// Returns the offset from the original wasm module's function to this
    /// frame's program counter.
    ///
    /// The offset here is the offset from the beginning of the defining
    /// function of this frame (within the wasm module) to the instruction this
    /// frame points to.
    pub fn func_offset(&self) -> usize {
        (self.instr.bits() - self.func_start.bits()) as usize
    }
}
