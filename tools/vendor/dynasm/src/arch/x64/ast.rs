use syn;
use proc_macro2::Span;

use crate::common::{Size, Jump};

use std::cmp::PartialEq;


/**
 * Reused AST parts
 */

/**
 * Registers
 */

#[derive(Debug, Clone)]
pub struct Register {
    pub size: Size,
    pub kind: RegKind
}

#[derive(Debug, Clone)]
pub enum RegKind {
    Static(RegId),
    Dynamic(RegFamily, syn::Expr)
}

// this map identifies the different registers that exist. some of these can be referred to as different sizes
// but they share the same ID here (think AL/AX/EAX/RAX, XMM/YMM)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RegId {
    // size: 1, 2, 4 or 8 bytes
    RAX = 0x00, RCX = 0x01, RDX = 0x02, RBX = 0x03,
    RSP = 0x04, RBP = 0x05, RSI = 0x06, RDI = 0x07,
    R8  = 0x08, R9  = 0x09, R10 = 0x0A, R11 = 0x0B,
    R12 = 0x0C, R13 = 0x0D, R14 = 0x0E, R15 = 0x0F,

    // size: 4 or 8 bytes
    RIP = 0x15,

    // size: 1 byte
    AH = 0x24, CH = 0x25, DH = 0x26, BH = 0x27,

    // size: 10 bytes
    ST0 = 0x30, ST1 = 0x31, ST2 = 0x32, ST3 = 0x33,
    ST4 = 0x34, ST5 = 0x35, ST6 = 0x36, ST7 = 0x37,

    // size: 8 bytes. alternative encoding exists
    MMX0 = 0x40, MMX1 = 0x41, MMX2 = 0x42, MMX3 = 0x43,
    MMX4 = 0x44, MMX5 = 0x45, MMX6 = 0x46, MMX7 = 0x47,

    // size: 16 bytes or 32 bytes
    XMM0  = 0x50, XMM1  = 0x51, XMM2  = 0x52, XMM3  = 0x53,
    XMM4  = 0x54, XMM5  = 0x55, XMM6  = 0x56, XMM7  = 0x57,
    XMM8  = 0x58, XMM9  = 0x59, XMM10 = 0x5A, XMM11 = 0x5B,
    XMM12 = 0x5C, XMM13 = 0x5D, XMM14 = 0x5E, XMM15 = 0x5F,

    // size: 2 bytes. alternative encoding exists
    ES = 0x60, CS = 0x61, SS = 0x62, DS = 0x63,
    FS = 0x64, GS = 0x65,

    // size: 4 bytes
    CR0  = 0x70, CR1  = 0x71, CR2  = 0x72, CR3  = 0x73,
    CR4  = 0x74, CR5  = 0x75, CR6  = 0x76, CR7  = 0x77,
    CR8  = 0x78, CR9  = 0x79, CR10 = 0x7A, CR11 = 0x7B,
    CR12 = 0x7C, CR13 = 0x7D, CR14 = 0x7E, CR15 = 0x7F,

    // size: 4 bytes
    DR0  = 0x80, DR1  = 0x81, DR2  = 0x82, DR3  = 0x83,
    DR4  = 0x84, DR5  = 0x85, DR6  = 0x86, DR7  = 0x87,
    DR8  = 0x88, DR9  = 0x89, DR10 = 0x8A, DR11 = 0x8B,
    DR12 = 0x8C, DR13 = 0x8D, DR14 = 0x8E, DR15 = 0x8F,

    // size: 16 bytes
    BND0 = 0x90, BND1 = 0x91, BND2 = 0x92, BND3 = 0x93
}

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy)]
pub enum RegFamily {
    LEGACY = 0,
    RIP = 1,
    HIGHBYTE = 2,
    FP = 3,
    MMX = 4,
    XMM = 5,
    SEGMENT = 6,
    CONTROL = 7,
    DEBUG = 8,
    BOUND = 9
}

impl Register {
    pub fn new_static(size: Size, id: RegId) -> Register {
        Register {size, kind: RegKind::Static(id) }
    }

    pub fn new_dynamic(size: Size, family: RegFamily, id: syn::Expr) -> Register {
        Register {size, kind: RegKind::Dynamic(family, id) }
    }

    pub fn size(&self) -> Size {
        self.size
    }
}

impl RegKind {
    pub fn code(&self) -> Option<u8> {
        match *self {
            RegKind::Static(code) => Some(code.code()),
            RegKind::Dynamic(_, _) => None
        }
    }

    pub fn family(&self) -> RegFamily {
        match *self {
            RegKind::Static(code) => code.family(),
            RegKind::Dynamic(family, _) => family
        }
    }

    pub fn is_dynamic(&self) -> bool {
        match *self {
            RegKind::Static(_) => false,
            RegKind::Dynamic(_, _) => true
        }
    }

    pub fn is_extended(&self) -> bool {
        match self.family() {
            RegFamily::LEGACY  |
            RegFamily::XMM     |
            RegFamily::CONTROL |
            RegFamily::DEBUG   => self.code().unwrap_or(8) > 7,
            _ => false
        }
    }

    pub fn encode(&self) -> u8 {
        self.code().unwrap_or(0)
    }

    pub fn from_number(id: u8) -> RegKind {
        RegKind::Static(RegId::from_number(id))
    }
}

impl PartialEq<Register> for Register {
    fn eq(&self, other: &Register) -> bool {
        if self.size == other.size {
            if let RegKind::Static(code) = self.kind {
                if let RegKind::Static(other_code) = other.kind {
                    return code == other_code
                }
            }
        }
        false
    }
}

impl PartialEq<RegId> for Register {
    fn eq(&self, other: &RegId) -> bool {
        self.kind == *other
    }
}

impl PartialEq<RegId> for RegKind {
    fn eq(&self, other: &RegId) -> bool {
        match *self {
            RegKind::Static(id) => id == *other,
            RegKind::Dynamic(_, _) => false
        }
    }
}

// workarounds to mask an impl<A, B> PartialEq<B> for Option<A: PartialEq<B>>
impl PartialEq<RegId> for Option<Register> {
    fn eq(&self, other: &RegId) -> bool {
        match *self {
            Some(ref a) => a == other,
            None => false
        }
    }
}

impl PartialEq<RegId> for Option<RegKind> {
    fn eq(&self, other: &RegId) -> bool {
        match *self {
            Some(ref a) => a == other,
            None => false
        }
    }
}

impl RegId {
    pub fn code(self) -> u8 {
        self as u8 & 0xF
    }

    pub fn family(self) -> RegFamily {
        match self as u8 >> 4 {
            0 => RegFamily::LEGACY,
            1 => RegFamily::RIP,
            2 => RegFamily::HIGHBYTE,
            3 => RegFamily::FP,
            4 => RegFamily::MMX,
            5 => RegFamily::XMM,
            6 => RegFamily::SEGMENT,
            7 => RegFamily::CONTROL,
            8 => RegFamily::DEBUG,
            9 => RegFamily::BOUND,
            _ => unreachable!()
        }
    }

    pub fn from_number(id: u8) -> RegId {
        match id {
            0  => RegId::RAX,
            1  => RegId::RCX,
            2  => RegId::RDX,
            3  => RegId::RBX,
            4  => RegId::RSP,
            5  => RegId::RBP,
            6  => RegId::RSI,
            7  => RegId::RDI,
            8  => RegId::R8,
            9  => RegId::R9,
            10 => RegId::R10,
            11 => RegId::R11,
            12 => RegId::R12,
            13 => RegId::R13,
            14 => RegId::R14,
            15 => RegId::R15,
            _ => panic!("invalid register code {:?}", id)
        }
    }
}

/**
 * Memory ref items
 */

#[derive(Debug)]
pub enum MemoryRefItem {
    ScaledRegister(Register, isize),
    Register(Register),
    Displacement(syn::Expr)
}

/**
 * Parsed ast
 */

#[derive(Debug)]
pub enum RawArg {
    // unprocessed typemapped argument
    TypeMappedRaw {
        span: Span,
        nosplit: bool,
        value_size: Option<Size>,
        disp_size: Option<Size>,
        base_reg: Register,
        scale: syn::Path,
        scaled_items: Vec<MemoryRefItem>,
        attribute: Option<syn::Ident>,
    },
    // unprocessed memory reference argument
    IndirectRaw {
        span: Span,
        nosplit: bool,
        value_size: Option<Size>,
        disp_size: Option<Size>,
        items: Vec<MemoryRefItem>,
    },
    // direct register reference, 
    Direct {
        span: Span,
        reg: Register
    },
    // a jump offset, i.e. ->foo
    JumpTarget {
        jump: Jump,
        size: Option<Size>
    },
    // a memory reference to a label, i.e. [->foo]
    IndirectJumpTarget {
        jump: Jump,
        size: Option<Size>
    },
    // just an arbitrary expression
    Immediate {
        value: syn::Expr,
        size: Option<Size>
    },
    // used to not block the parser on a parsing error in a single arg
    Invalid
}

#[derive(Debug)]
pub enum CleanArg {
    // memory reference
    Indirect {
        span: Span,
        nosplit: bool,
        size: Option<Size>,
        disp_size: Option<Size>,
        base: Option<Register>,
        index: Option<(Register, isize, Option<syn::Expr>)>,
        disp: Option<syn::Expr>
    },
    // direct register reference, 
    Direct {
        span: Span,
        reg: Register
    },
    // a jump offset, i.e. ->foo
    JumpTarget {
        jump: Jump,
        size: Option<Size>
    },
    // a memory reference to a label, i.e. [->foo]
    IndirectJumpTarget {
        jump: Jump,
        size: Option<Size>
    },
    // just an arbitrary expression
    Immediate {
        value: syn::Expr,
        size: Option<Size>
    }
}

#[derive(Debug)]
pub enum SizedArg {
    // memory reference. size info is lost here as
    // it is never actually encoded
    Indirect {
        span: Span,
        disp_size: Option<Size>,
        base: Option<Register>,
        index: Option<(Register, isize, Option<syn::Expr>)>,
        disp: Option<syn::Expr>
    },
    // direct register reference, 
    Direct {
        span: Span,
        reg: Register
    },
    // a jump offset, i.e. ->foo
    JumpTarget {
        jump: Jump,
        size: Size
    },
    // a memory reference to a label, i.e. [->foo]
    IndirectJumpTarget {
        jump: Jump
    },
    // just an arbitrary expression
    Immediate {
        value: syn::Expr,
        size: Size
    }
}

/**
 * Parsed instruction
 */

pub struct Instruction {
    pub span: Span,
    pub idents: Vec<syn::Ident>
}
