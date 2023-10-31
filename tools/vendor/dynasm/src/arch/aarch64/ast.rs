use syn;
use proc_macro2::Span;

use crate::common::{Size, Jump};


/// A complete abstraction of an aarch64 register access.
#[derive(Debug, Clone)]
pub enum Register {
    Scalar(RegScalar),
    Vector(RegVector)
}

/// A vcalar register. Can be either of the integer or simd families. 
#[derive(Debug, Clone)]
pub struct RegScalar {
    pub kind: RegKind,
    pub size: Size
}

/// A vector register. Can only be of the simd family
#[derive(Debug, Clone)]
pub struct RegVector {
    pub kind: RegKind,
    pub element_size: Size,
    pub lanes: Option<u8>,
    pub element: Option<syn::Expr>
}

// Register id without indication of its usage. Either a static Regid or a family identifier + expression to choose the register
#[derive(Debug, Clone)]
pub enum RegKind {
    Static(RegId),
    Dynamic(RegFamily, syn::Expr)
}

// a register identifier. This identifies an architecturally completely separate register.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RegId {
    // regular registers. Either 4 or 8 bytes
    X0 = 0x00, X1 = 0x01, X2 = 0x02, X3 = 0x03,
    X4 = 0x04, X5 = 0x05, X6 = 0x06, X7 = 0x07,
    X8 = 0x08, X9 = 0x09, X10= 0x0A, X11= 0x0B,
    X12= 0x0C, X13= 0x0D, X14= 0x0E, X15= 0x0F,
    X16= 0x10, X17= 0x11, X18= 0x12, X19= 0x13,
    X20= 0x14, X21= 0x15, X22= 0x16, X23= 0x17,
    X24= 0x18, X25= 0x19, X26= 0x1A, X27= 0x1B,
    X28= 0x1C, X29= 0x1D, X30= 0x1E,

    // zero register. Either 4 or 8 bytes
    XZR= 0x1F,

    // stack pointer. Either 4 or 8 bytes. the encoding overlaps XZR, and we only differentiate
    // the two of them to provide diagnostics. They count as the same family.
    SP = 0x3F,

    // scalar FP / vector SIMD registers. Can be used as 1, 2, 4, 8 or 16-byte size.
    V0 = 0x40, V1 = 0x41, V2 = 0x42, V3 = 0x43,
    V4 = 0x44, V5 = 0x45, V6 = 0x46, V7 = 0x47,
    V8 = 0x48, V9 = 0x49, V10= 0x4A, V11= 0x4B,
    V12= 0x4C, V13= 0x4D, V14= 0x4E, V15= 0x4F,
    V16= 0x50, V17= 0x51, V18= 0x52, V19= 0x53,
    V20= 0x54, V21= 0x55, V22= 0x56, V23= 0x57,
    V24= 0x58, V25= 0x59, V26= 0x5A, V27= 0x5B,
    V28= 0x5C, V29= 0x5D, V30= 0x5E, V31= 0x5F
}

// register family. INTEGER = Xn/Wn including XZR/WZR. INTEGERSP is just SP or XSP. SIMD = Bn/Hn/Sn/Dn/Qn
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RegFamily {
    INTEGER   = 0,
    INTEGERSP = 1,
    SIMD      = 2,
}

impl RegId {
    /// Encode this RegId in a 5-bit value
    pub fn code(self) -> u8 {
        self as u8 & 0x1F
    }

    /// Returns what family this Regid is from
    pub fn family(self) -> RegFamily {
        match self as u8 >> 5 {
            0 => RegFamily::INTEGER,
            1 => RegFamily::INTEGERSP,
            2 => RegFamily::SIMD,
            _ => unreachable!()
        }
    }
}

impl RegKind {
    /// Get the 5-bit code of this RegKind. Returns None if it was dynamic
    pub fn code(&self) -> Option<u8> {
        match self {
            RegKind::Static(code) => Some(code.code()),
            RegKind::Dynamic(_, _) => None
        }
    }

    /// Encode this RegKind into a 5-bit value, returning 0 if it was dynamic
    pub fn encode(&self) -> u8 {
        self.code().unwrap_or(0)
    }

    /// Returns the family that this regkind is of
    pub fn family(&self) -> RegFamily {
        match *self {
            RegKind::Static(code) => code.family(),
            RegKind::Dynamic(family, _) => family
        }
    }

    /// Returns true if this RegKind is dynamic
    pub fn is_dynamic(&self) -> bool {
        match self {
            RegKind::Static(_) => false,
            RegKind::Dynamic(_, _) => true
        }
    }

    /// Returns true if this RegKind is static and identifies the zero register
    pub fn is_zero_reg(&self) -> bool {
        match self {
            RegKind::Static(ref id) => *id == RegId::XZR,
            RegKind::Dynamic(_, _) => false,
        }
    }
}

impl PartialEq<RegKind> for RegKind {
    fn eq(&self, other: &RegKind) -> bool {
        match self {
            RegKind::Static(id) => match other {
                RegKind::Static(other_id) => other_id == id,
                RegKind::Dynamic(_, _) => false,
            },
            RegKind::Dynamic(_, _) => false,
        }
    }
}

impl RegScalar {
    pub fn size(&self) -> Size {
        self.size
    }
}

impl RegVector {
    /// Returns the size of individual elements in this vector register
    pub fn element_size(&self) -> Size {
        self.element_size
    }

    /// Returns the full size of this vector register (element size * lanecount).
    /// Returns None if lanes was not set
    pub fn full_size(&self) -> Option<u16> {
        if let Some(lanes) = self.lanes {
            Some(u16::from(lanes) * u16::from(self.element_size.in_bytes()))
        } else { 
            None
        }
    }
}

impl Register {
    pub fn size(&self) -> Size {
        match self {
            Register::Scalar(s) => s.size(),
            Register::Vector(v) => v.element_size()
        }
    }

    pub fn kind(&self) -> &RegKind {
        match self {
            Register::Scalar(s) => &s.kind,
            Register::Vector(v) => &v.kind
        }
    }

    pub fn kind_owned(self) -> RegKind {
        match self {
            Register::Scalar(s) => s.kind,
            Register::Vector(v) => v.kind
        }
    }

    pub fn family(&self) -> RegFamily {
        match self {
            Register::Scalar(s) => s.kind.family(),
            Register::Vector(_) => RegFamily::SIMD,
        }
    }

    pub fn assume_vector(&self) -> &RegVector {
        match self {
            Register::Scalar(_) => panic!("That wasn't a vector register"),
            Register::Vector(v) => v
        }
    }
}

/**
 * Modifier types
 */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    LSL,
    LSR,
    ASR,
    ROR,
    SXTX,
    SXTW,
    SXTH,
    SXTB,
    UXTX,
    UXTW,
    UXTH,
    UXTB,
    MSL,
}

impl Modifier {
    pub fn as_str(self) -> &'static str {
        match self {
            Modifier::LSL => "LSL",
            Modifier::LSR => "LSR",
            Modifier::ASR => "ASR",
            Modifier::ROR => "ROR",
            Modifier::SXTX => "SXTX",
            Modifier::SXTW => "SXTW",
            Modifier::SXTH => "SXTH",
            Modifier::SXTB => "SXTB",
            Modifier::UXTX => "UXTX",
            Modifier::UXTW => "UXTW",
            Modifier::UXTH => "UXTH",
            Modifier::UXTB => "UXTB",
            Modifier::MSL => "MSL",
        }
    }

    pub fn expr_required(self) -> bool {
        match self {
            Modifier::LSL
            | Modifier::LSR
            | Modifier::ASR
            | Modifier::ROR
            | Modifier::MSL => true,
            Modifier::SXTX
            | Modifier::SXTW
            | Modifier::SXTH
            | Modifier::SXTB
            | Modifier::UXTX
            | Modifier::UXTW
            | Modifier::UXTH
            | Modifier::UXTB => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModifyExpr {
    pub op: Modifier,
    pub expr: Option<syn::Expr>
}

impl ModifyExpr {
    pub fn new(op: Modifier, expr: Option<syn::Expr>) -> ModifyExpr {
        ModifyExpr {
            op,
            expr
        }
    }
}

/**
 * Memory ref item types
 */

#[derive(Debug)]
pub enum RefItem {
    Direct {
        span: Span,
        reg: Register
    },
    Immediate {
        value: syn::Expr
    },
    Modifier {
        span: Span,
        modifier: ModifyExpr
    }
}

// basic parse results, before we start doing any kind of checking
#[derive(Debug)]
pub enum RawArg {
    // A memory reference
    Reference {
        span: Span,
        items: Vec<RefItem>,
        bang: bool
    },
    // A register list, defined as first - last
    DashList {
        span: Span,
        first: Register,
        last: Register,
        element: Option<syn::Expr>
    },
    // A register list, defined as item, item, item, item
    CommaList{
        span: Span,
        items: Vec<Register>,
        element: Option<syn::Expr>
    },
    AmountList {
        span: Span,
        first: Register,
        amount: syn::Expr,
        element: Option<syn::Expr>
    },
    // direct register reference
    Direct {
        span: Span,
        reg: Register
    },
    // jump target. Also used by PC-rel loads etc
    JumpTarget {
        jump: Jump
    },
    // just an arbitrary expression
    Immediate {
        prefixed: bool,
        value: syn::Expr
    },
    // a modifier
    Modifier {
        span: Span,
        modifier: ModifyExpr
    },
    // a dot
    Dot {
        span: Span
    },
    // an ident, not intended to be parsed as an expression
    Lit {
        ident: syn::Ident
    }
}

// Contains the actual instruction mnemnonic.
#[derive(Debug)]
pub struct Instruction {
    pub span: Span,
    pub ident: syn::Ident
}

#[derive(Debug)]
pub enum RefKind {
    Base,
    Offset(syn::Expr),
    Indexed(Register, Option<ModifyExpr>),
    PreIndexed(syn::Expr),
}

// sanitized parse results
#[derive(Debug)]
pub enum CleanArg {
    Reference {
        span: Span,
        base: Register,
        kind: RefKind
    },
    RegList {
        span: Span,
        first: Register,
        amount: u8,
        element: Option<syn::Expr>
    },
    Direct {
        span: Span,
        reg: Register
    },
    JumpTarget {
        jump: Jump
    },
    Immediate {
        prefixed: bool,
        value: syn::Expr,
    },
    Modifier {
        span: Span,
        modifier: ModifyExpr
    },
    Dot {
        span: Span
    },
    Lit {
        ident: syn::Ident
    }
}

// flat arg list after matching, for encoding
#[derive(Debug)]
pub enum FlatArg {
    Direct {
        span: Span,
        reg: RegKind
    },
    Immediate {
        value: syn::Expr,
    },
    Modifier {
        span: Span,
        modifier: Modifier,
    },
    JumpTarget {
        jump: Jump
    },
    Lit {
        ident: syn::Ident
    },
    Default
}
