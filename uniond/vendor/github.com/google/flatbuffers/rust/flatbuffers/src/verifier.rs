use crate::follow::Follow;
use crate::{ForwardsUOffset, SOffsetT, SkipSizePrefix, UOffsetT, VOffsetT, Vector, SIZE_UOFFSET};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::ops::Range;
use core::option::Option;

#[cfg(all(nightly, not(feature = "std")))]
use core::error::Error;
#[cfg(feature = "std")]
use std::error::Error;

/// Traces the location of data errors. Not populated for Dos detecting errors.
/// Useful for MissingRequiredField and Utf8Error in particular, though
/// the other errors should not be producible by correct flatbuffers implementations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorTraceDetail {
    VectorElement {
        index: usize,
        position: usize,
    },
    TableField {
        field_name: &'static str,
        position: usize,
    },
    UnionVariant {
        variant: &'static str,
        position: usize,
    },
}

#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct ErrorTrace(Vec<ErrorTraceDetail>);

impl core::convert::AsRef<[ErrorTraceDetail]> for ErrorTrace {
    #[inline]
    fn as_ref(&self) -> &[ErrorTraceDetail] {
        &self.0
    }
}

/// Describes how a flatuffer is invalid and, for data errors, roughly where. No extra tracing
/// information is given for DoS detecting errors since it will probably be a lot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidFlatbuffer {
    MissingRequiredField {
        required: &'static str,
        error_trace: ErrorTrace,
    },
    InconsistentUnion {
        field: &'static str,
        field_type: &'static str,
        error_trace: ErrorTrace,
    },
    Utf8Error {
        error: core::str::Utf8Error,
        range: Range<usize>,
        error_trace: ErrorTrace,
    },
    MissingNullTerminator {
        range: Range<usize>,
        error_trace: ErrorTrace,
    },
    Unaligned {
        position: usize,
        unaligned_type: &'static str,
        error_trace: ErrorTrace,
    },
    RangeOutOfBounds {
        range: Range<usize>,
        error_trace: ErrorTrace,
    },
    SignedOffsetOutOfBounds {
        soffset: SOffsetT,
        position: usize,
        error_trace: ErrorTrace,
    },
    // Dos detecting errors. These do not get error traces since it will probably be very large.
    TooManyTables,
    ApparentSizeTooLarge,
    DepthLimitReached,
}

#[cfg(any(nightly, feature = "std"))]
impl Error for InvalidFlatbuffer {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let InvalidFlatbuffer::Utf8Error { error: source, .. } = self {
            Some(source)
        } else {
            None
        }
    }
}

impl core::fmt::Display for InvalidFlatbuffer {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            InvalidFlatbuffer::MissingRequiredField {
                required,
                error_trace,
            } => {
                writeln!(f, "Missing required field `{}`.\n{}", required, error_trace)?;
            }
            InvalidFlatbuffer::InconsistentUnion {
                field,
                field_type,
                error_trace,
            } => {
                writeln!(
                    f,
                    "Exactly one of union discriminant (`{}`) and value (`{}`) are present.\n{}",
                    field_type, field, error_trace
                )?;
            }
            InvalidFlatbuffer::Utf8Error {
                error,
                range,
                error_trace,
            } => {
                writeln!(
                    f,
                    "Utf8 error for string in {:?}: {}\n{}",
                    range, error, error_trace
                )?;
            }
            InvalidFlatbuffer::MissingNullTerminator { range, error_trace } => {
                writeln!(
                    f,
                    "String in range [{}, {}) is missing its null terminator.\n{}",
                    range.start, range.end, error_trace
                )?;
            }
            InvalidFlatbuffer::Unaligned {
                position,
                unaligned_type,
                error_trace,
            } => {
                writeln!(
                    f,
                    "Type `{}` at position {} is unaligned.\n{}",
                    unaligned_type, position, error_trace
                )?;
            }
            InvalidFlatbuffer::RangeOutOfBounds { range, error_trace } => {
                writeln!(
                    f,
                    "Range [{}, {}) is out of bounds.\n{}",
                    range.start, range.end, error_trace
                )?;
            }
            InvalidFlatbuffer::SignedOffsetOutOfBounds {
                soffset,
                position,
                error_trace,
            } => {
                writeln!(
                    f,
                    "Signed offset at position {} has value {} which points out of bounds.\n{}",
                    position, soffset, error_trace
                )?;
            }
            InvalidFlatbuffer::TooManyTables {} => {
                writeln!(f, "Too many tables.")?;
            }
            InvalidFlatbuffer::ApparentSizeTooLarge {} => {
                writeln!(f, "Apparent size too large.")?;
            }
            InvalidFlatbuffer::DepthLimitReached {} => {
                writeln!(f, "Nested table depth limit reached.")?;
            }
        }
        Ok(())
    }
}

impl core::fmt::Display for ErrorTrace {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use ErrorTraceDetail::*;
        for e in self.0.iter() {
            match e {
                VectorElement { index, position } => {
                    writeln!(
                        f,
                        "\twhile verifying vector element {:?} at position {:?}",
                        index, position
                    )?;
                }
                TableField {
                    field_name,
                    position,
                } => {
                    writeln!(
                        f,
                        "\twhile verifying table field `{}` at position {:?}",
                        field_name, position
                    )?;
                }
                UnionVariant { variant, position } => {
                    writeln!(
                        f,
                        "\t while verifying union variant `{}` at position {:?}",
                        variant, position
                    )?;
                }
            }
        }
        Ok(())
    }
}

pub type Result<T> = core::result::Result<T, InvalidFlatbuffer>;

impl InvalidFlatbuffer {
    fn new_range_oob<T>(start: usize, end: usize) -> Result<T> {
        Err(Self::RangeOutOfBounds {
            range: Range { start, end },
            error_trace: Default::default(),
        })
    }
    fn new_inconsistent_union<T>(field: &'static str, field_type: &'static str) -> Result<T> {
        Err(Self::InconsistentUnion {
            field,
            field_type,
            error_trace: Default::default(),
        })
    }
    fn new_missing_required<T>(required: &'static str) -> Result<T> {
        Err(Self::MissingRequiredField {
            required,
            error_trace: Default::default(),
        })
    }
}

/// Records the path to the verifier detail if the error is a data error and not a DoS error.
fn append_trace<T>(mut res: Result<T>, d: ErrorTraceDetail) -> Result<T> {
    if let Err(e) = res.as_mut() {
        use InvalidFlatbuffer::*;
        if let MissingRequiredField { error_trace, .. }
        | Unaligned { error_trace, .. }
        | RangeOutOfBounds { error_trace, .. }
        | InconsistentUnion { error_trace, .. }
        | Utf8Error { error_trace, .. }
        | MissingNullTerminator { error_trace, .. }
        | SignedOffsetOutOfBounds { error_trace, .. } = e
        {
            error_trace.0.push(d)
        }
    }
    res
}

/// Adds a TableField trace detail if `res` is a data error.
fn trace_field<T>(res: Result<T>, field_name: &'static str, position: usize) -> Result<T> {
    append_trace(
        res,
        ErrorTraceDetail::TableField {
            field_name,
            position,
        },
    )
}

/// Adds a TableField trace detail if `res` is a data error.
fn trace_elem<T>(res: Result<T>, index: usize, position: usize) -> Result<T> {
    append_trace(res, ErrorTraceDetail::VectorElement { index, position })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifierOptions {
    /// Maximum depth of nested tables allowed in a valid flatbuffer.
    pub max_depth: usize,
    /// Maximum number of tables allowed in a valid flatbuffer.
    pub max_tables: usize,
    /// Maximum "apparent" size of the message if the Flatbuffer object DAG is expanded into a
    /// tree.
    pub max_apparent_size: usize,
    /// Ignore errors where a string is missing its null terminator.
    /// This is mostly a problem if the message will be sent to a client using old c-strings.
    pub ignore_missing_null_terminator: bool,
    // probably want an option to ignore utf8 errors since strings come from c++
    // options to error un-recognized enums and unions? possible footgun.
    // Ignore nested flatbuffers, etc?
}

impl Default for VerifierOptions {
    fn default() -> Self {
        Self {
            max_depth: 64,
            max_tables: 1_000_000,
            // size_ might do something different.
            max_apparent_size: 1 << 31,
            ignore_missing_null_terminator: false,
        }
    }
}

/// Carries the verification state. Should not be reused between tables.
#[derive(Debug)]
pub struct Verifier<'opts, 'buf> {
    buffer: &'buf [u8],
    opts: &'opts VerifierOptions,
    depth: usize,
    num_tables: usize,
    apparent_size: usize,
}

impl<'opts, 'buf> Verifier<'opts, 'buf> {
    pub fn new(opts: &'opts VerifierOptions, buffer: &'buf [u8]) -> Self {
        Self {
            opts,
            buffer,
            depth: 0,
            num_tables: 0,
            apparent_size: 0,
        }
    }
    /// Resets verifier internal state.
    #[inline]
    pub fn reset(&mut self) {
        self.depth = 0;
        self.num_tables = 0;
        self.num_tables = 0;
    }
    /// Checks `pos` is aligned to T's alignment. This does not mean `buffer[pos]` is aligned w.r.t
    /// memory since `buffer: &[u8]` has alignment 1.
    ///
    /// ### WARNING
    ///
    /// This does not work for flatbuffers-structs as they have alignment 1 according to
    /// `core::mem::align_of` but are meant to have higher alignment within a Flatbuffer w.r.t.
    /// `buffer[0]`. TODO(caspern).
    ///
    /// Note this does not impact soundness as this crate does not assume alignment of structs
    #[inline]
    fn is_aligned<T>(&self, pos: usize) -> Result<()> {
        if pos % core::mem::align_of::<T>() == 0 {
            Ok(())
        } else {
            Err(InvalidFlatbuffer::Unaligned {
                unaligned_type: core::any::type_name::<T>(),
                position: pos,
                error_trace: Default::default(),
            })
        }
    }
    #[inline]
    fn range_in_buffer(&mut self, pos: usize, size: usize) -> Result<()> {
        let end = pos.saturating_add(size);
        if end > self.buffer.len() {
            return InvalidFlatbuffer::new_range_oob(pos, end);
        }
        self.apparent_size += size;
        if self.apparent_size > self.opts.max_apparent_size {
            return Err(InvalidFlatbuffer::ApparentSizeTooLarge);
        }
        Ok(())
    }
    /// Check that there really is a T in there.
    #[inline]
    pub fn in_buffer<T>(&mut self, pos: usize) -> Result<()> {
        self.is_aligned::<T>(pos)?;
        self.range_in_buffer(pos, core::mem::size_of::<T>())
    }
    #[inline]
    fn get_u16(&mut self, pos: usize) -> Result<u16> {
        self.in_buffer::<u16>(pos)?;
        Ok(u16::from_le_bytes([self.buffer[pos], self.buffer[pos + 1]]))
    }
    #[inline]
    fn get_uoffset(&mut self, pos: usize) -> Result<UOffsetT> {
        self.in_buffer::<u32>(pos)?;
        Ok(u32::from_le_bytes([
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
        ]))
    }
    #[inline]
    fn deref_soffset(&mut self, pos: usize) -> Result<usize> {
        self.in_buffer::<SOffsetT>(pos)?;
        let offset = SOffsetT::from_le_bytes([
            self.buffer[pos],
            self.buffer[pos + 1],
            self.buffer[pos + 2],
            self.buffer[pos + 3],
        ]);

        // signed offsets are subtracted.
        let derefed = if offset > 0 {
            pos.checked_sub(offset.unsigned_abs() as usize)
        } else {
            pos.checked_add(offset.unsigned_abs() as usize)
        };
        if let Some(x) = derefed {
            if x < self.buffer.len() {
                return Ok(x);
            }
        }
        Err(InvalidFlatbuffer::SignedOffsetOutOfBounds {
            soffset: offset,
            position: pos,
            error_trace: Default::default(),
        })
    }
    #[inline]
    pub fn visit_table<'ver>(
        &'ver mut self,
        table_pos: usize,
    ) -> Result<TableVerifier<'ver, 'opts, 'buf>> {
        let vtable_pos = self.deref_soffset(table_pos)?;
        let vtable_len = self.get_u16(vtable_pos)? as usize;
        self.is_aligned::<VOffsetT>(vtable_pos.saturating_add(vtable_len))?; // i.e. vtable_len is even.
        self.range_in_buffer(vtable_pos, vtable_len)?;
        // Check bounds.
        self.num_tables += 1;
        if self.num_tables > self.opts.max_tables {
            return Err(InvalidFlatbuffer::TooManyTables);
        }
        self.depth += 1;
        if self.depth > self.opts.max_depth {
            return Err(InvalidFlatbuffer::DepthLimitReached);
        }
        Ok(TableVerifier {
            pos: table_pos,
            vtable: vtable_pos,
            vtable_len,
            verifier: self,
        })
    }

    /// Runs the union variant's type's verifier assuming the variant is at the given position,
    /// tracing the error.
    pub fn verify_union_variant<T: Verifiable>(
        &mut self,
        variant: &'static str,
        position: usize,
    ) -> Result<()> {
        let res = T::run_verifier(self, position);
        append_trace(res, ErrorTraceDetail::UnionVariant { variant, position })
    }
}

// Cache table metadata in usize so we don't have to cast types or jump around so much.
// We will visit every field anyway.
pub struct TableVerifier<'ver, 'opts, 'buf> {
    // Absolute position of table in buffer
    pos: usize,
    // Absolute position of vtable in buffer.
    vtable: usize,
    // Length of vtable.
    vtable_len: usize,
    // Verifier struct which holds the surrounding state and options.
    verifier: &'ver mut Verifier<'opts, 'buf>,
}

impl<'ver, 'opts, 'buf> TableVerifier<'ver, 'opts, 'buf> {
    fn deref(&mut self, field: VOffsetT) -> Result<Option<usize>> {
        let field = field as usize;
        if field < self.vtable_len {
            let field_offset = self.verifier.get_u16(self.vtable.saturating_add(field))?;
            if field_offset > 0 {
                // Field is present.
                let field_pos = self.pos.saturating_add(field_offset as usize);
                return Ok(Some(field_pos));
            }
        }
        Ok(None)
    }

    #[inline]
    pub fn visit_field<T: Verifiable>(
        mut self,
        field_name: &'static str,
        field: VOffsetT,
        required: bool,
    ) -> Result<Self> {
        if let Some(field_pos) = self.deref(field)? {
            trace_field(
                T::run_verifier(self.verifier, field_pos),
                field_name,
                field_pos,
            )?;
            return Ok(self);
        }
        if required {
            InvalidFlatbuffer::new_missing_required(field_name)
        } else {
            Ok(self)
        }
    }
    #[inline]
    /// Union verification is complicated. The schemas passes this function the metadata of the
    /// union's key (discriminant) and value fields, and a callback. The function verifies and
    /// reads the key, then invokes the callback to perform data-dependent verification.
    pub fn visit_union<Key, UnionVerifier>(
        mut self,
        key_field_name: &'static str,
        key_field_voff: VOffsetT,
        val_field_name: &'static str,
        val_field_voff: VOffsetT,
        required: bool,
        verify_union: UnionVerifier,
    ) -> Result<Self>
    where
        Key: Follow<'buf> + Verifiable,
        UnionVerifier:
            (core::ops::FnOnce(<Key as Follow<'buf>>::Inner, &mut Verifier, usize) -> Result<()>),
        // NOTE: <Key as Follow<'buf>>::Inner == Key
    {
        // TODO(caspern): how to trace vtable errors?
        let val_pos = self.deref(val_field_voff)?;
        let key_pos = self.deref(key_field_voff)?;
        match (key_pos, val_pos) {
            (None, None) => {
                if required {
                    InvalidFlatbuffer::new_missing_required(val_field_name)
                } else {
                    Ok(self)
                }
            }
            (Some(k), Some(v)) => {
                trace_field(Key::run_verifier(self.verifier, k), key_field_name, k)?;
                // Safety:
                // Run verifier on `k` above
                let discriminant = unsafe { Key::follow(self.verifier.buffer, k) };
                trace_field(
                    verify_union(discriminant, self.verifier, v),
                    val_field_name,
                    v,
                )?;
                Ok(self)
            }
            _ => InvalidFlatbuffer::new_inconsistent_union(key_field_name, val_field_name),
        }
    }
    pub fn finish(self) -> &'ver mut Verifier<'opts, 'buf> {
        self.verifier.depth -= 1;
        self.verifier
    }
}

// Needs to be implemented for Tables and maybe structs.
// Unions need some special treatment.
pub trait Verifiable {
    /// Runs the verifier for this type, assuming its at position `pos` in the verifier's buffer.
    /// Should not need to be called directly.
    fn run_verifier(v: &mut Verifier, pos: usize) -> Result<()>;
}

// Verify the uoffset and then pass verifier to the type being pointed to.
impl<T: Verifiable> Verifiable for ForwardsUOffset<T> {
    #[inline]
    fn run_verifier(v: &mut Verifier, pos: usize) -> Result<()> {
        let offset = v.get_uoffset(pos)? as usize;
        let next_pos = offset.saturating_add(pos);
        T::run_verifier(v, next_pos)
    }
}

/// Checks and returns the range containing the flatbuffers vector.
fn verify_vector_range<T>(v: &mut Verifier, pos: usize) -> Result<core::ops::Range<usize>> {
    let len = v.get_uoffset(pos)? as usize;
    let start = pos.saturating_add(SIZE_UOFFSET);
    v.is_aligned::<T>(start)?;
    let size = len.saturating_mul(core::mem::size_of::<T>());
    let end = start.saturating_add(size);
    v.range_in_buffer(start, size)?;
    Ok(core::ops::Range { start, end })
}

pub trait SimpleToVerifyInSlice {}

impl SimpleToVerifyInSlice for bool {}

impl SimpleToVerifyInSlice for i8 {}

impl SimpleToVerifyInSlice for u8 {}

impl SimpleToVerifyInSlice for i16 {}

impl SimpleToVerifyInSlice for u16 {}

impl SimpleToVerifyInSlice for i32 {}

impl SimpleToVerifyInSlice for u32 {}

impl SimpleToVerifyInSlice for f32 {}

impl SimpleToVerifyInSlice for i64 {}

impl SimpleToVerifyInSlice for u64 {}

impl SimpleToVerifyInSlice for f64 {}

impl<T: SimpleToVerifyInSlice> Verifiable for Vector<'_, T> {
    fn run_verifier(v: &mut Verifier, pos: usize) -> Result<()> {
        verify_vector_range::<T>(v, pos)?;
        Ok(())
    }
}

impl<T: Verifiable> Verifiable for SkipSizePrefix<T> {
    #[inline]
    fn run_verifier(v: &mut Verifier, pos: usize) -> Result<()> {
        T::run_verifier(v, pos.saturating_add(crate::SIZE_SIZEPREFIX))
    }
}

impl<T: Verifiable> Verifiable for Vector<'_, ForwardsUOffset<T>> {
    #[inline]
    fn run_verifier(v: &mut Verifier, pos: usize) -> Result<()> {
        let range = verify_vector_range::<ForwardsUOffset<T>>(v, pos)?;
        let size = core::mem::size_of::<ForwardsUOffset<T>>();
        for (i, element_pos) in range.step_by(size).enumerate() {
            trace_elem(
                <ForwardsUOffset<T>>::run_verifier(v, element_pos),
                i,
                element_pos,
            )?;
        }
        Ok(())
    }
}

impl<'a> Verifiable for &'a str {
    #[inline]
    fn run_verifier(v: &mut Verifier, pos: usize) -> Result<()> {
        let range = verify_vector_range::<u8>(v, pos)?;
        let has_null_terminator = v.buffer.get(range.end).map(|&b| b == 0).unwrap_or(false);
        let s = core::str::from_utf8(&v.buffer[range.clone()]);
        if let Err(error) = s {
            return Err(InvalidFlatbuffer::Utf8Error {
                error,
                range,
                error_trace: Default::default(),
            });
        }
        if !v.opts.ignore_missing_null_terminator && !has_null_terminator {
            return Err(InvalidFlatbuffer::MissingNullTerminator {
                range,
                error_trace: Default::default(),
            });
        }
        Ok(())
    }
}

// Verify VectorOfTables, Unions, Arrays, Structs...
macro_rules! impl_verifiable_for {
    ($T: ty) => {
        impl Verifiable for $T {
            #[inline]
            fn run_verifier<'opts, 'buf>(v: &mut Verifier<'opts, 'buf>, pos: usize) -> Result<()> {
                v.in_buffer::<$T>(pos)
            }
        }
    };
}
impl_verifiable_for!(bool);
impl_verifiable_for!(u8);
impl_verifiable_for!(i8);
impl_verifiable_for!(u16);
impl_verifiable_for!(i16);
impl_verifiable_for!(u32);
impl_verifiable_for!(i32);
impl_verifiable_for!(f32);
impl_verifiable_for!(u64);
impl_verifiable_for!(i64);
impl_verifiable_for!(f64);
