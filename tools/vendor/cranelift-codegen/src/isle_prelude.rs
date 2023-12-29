//! Shared ISLE prelude implementation for optimization (mid-end) and
//! lowering (backend) ISLE environments.

/// Helper macro to define methods in `prelude.isle` within `impl Context for
/// ...` for each backend. These methods are shared amongst all backends.
#[macro_export]
#[doc(hidden)]
macro_rules! isle_common_prelude_methods {
    () => {
        /// We don't have a way of making a `()` value in isle directly.
        #[inline]
        fn unit(&mut self) -> Unit {
            ()
        }

        #[inline]
        fn u8_as_u32(&mut self, x: u8) -> Option<u32> {
            Some(x.into())
        }

        #[inline]
        fn u8_as_u64(&mut self, x: u8) -> Option<u64> {
            Some(x.into())
        }

        #[inline]
        fn u16_as_u64(&mut self, x: u16) -> Option<u64> {
            Some(x.into())
        }

        #[inline]
        fn u32_as_u64(&mut self, x: u32) -> Option<u64> {
            Some(x.into())
        }

        #[inline]
        fn i64_as_u64(&mut self, x: i64) -> Option<u64> {
            Some(x as u64)
        }

        #[inline]
        fn u64_add(&mut self, x: u64, y: u64) -> Option<u64> {
            Some(x.wrapping_add(y))
        }

        #[inline]
        fn u64_sub(&mut self, x: u64, y: u64) -> Option<u64> {
            Some(x.wrapping_sub(y))
        }

        #[inline]
        fn u64_mul(&mut self, x: u64, y: u64) -> Option<u64> {
            Some(x.wrapping_mul(y))
        }

        #[inline]
        fn u64_sdiv(&mut self, x: u64, y: u64) -> Option<u64> {
            let x = x as i64;
            let y = y as i64;
            x.checked_div(y).map(|d| d as u64)
        }

        #[inline]
        fn u64_udiv(&mut self, x: u64, y: u64) -> Option<u64> {
            x.checked_div(y)
        }

        #[inline]
        fn u64_and(&mut self, x: u64, y: u64) -> Option<u64> {
            Some(x & y)
        }

        #[inline]
        fn u64_or(&mut self, x: u64, y: u64) -> Option<u64> {
            Some(x | y)
        }

        #[inline]
        fn u64_xor(&mut self, x: u64, y: u64) -> Option<u64> {
            Some(x ^ y)
        }

        #[inline]
        fn u64_not(&mut self, x: u64) -> Option<u64> {
            Some(!x)
        }

        #[inline]
        fn u64_is_zero(&mut self, value: u64) -> bool {
            0 == value
        }

        #[inline]
        fn u64_is_odd(&mut self, x: u64) -> Option<bool> {
            Some(x & 1 == 1)
        }

        #[inline]
        fn u64_sextend_u32(&mut self, x: u64) -> Option<u64> {
            Some(x as u32 as i32 as i64 as u64)
        }

        #[inline]
        fn u64_uextend_u32(&mut self, x: u64) -> Option<u64> {
            Some(x & 0xffff_ffff)
        }

        #[inline]
        fn ty_bits(&mut self, ty: Type) -> Option<u8> {
            use std::convert::TryInto;
            Some(ty.bits().try_into().unwrap())
        }

        #[inline]
        fn ty_bits_u16(&mut self, ty: Type) -> u16 {
            ty.bits() as u16
        }

        #[inline]
        fn ty_bits_u64(&mut self, ty: Type) -> u64 {
            ty.bits() as u64
        }

        #[inline]
        fn ty_bytes(&mut self, ty: Type) -> u16 {
            u16::try_from(ty.bytes()).unwrap()
        }

        #[inline]
        fn ty_mask(&mut self, ty: Type) -> u64 {
            match ty.bits() {
                1 => 1,
                8 => 0xff,
                16 => 0xffff,
                32 => 0xffff_ffff,
                64 => 0xffff_ffff_ffff_ffff,
                _ => unimplemented!(),
            }
        }

        fn fits_in_16(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 16 && !ty.is_dynamic_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn fits_in_32(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 32 && !ty.is_dynamic_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn lane_fits_in_32(&mut self, ty: Type) -> Option<Type> {
            if !ty.is_vector() && !ty.is_dynamic_vector() {
                None
            } else if ty.lane_type().bits() <= 32 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn fits_in_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 64 && !ty.is_dynamic_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_int_ref_scalar_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 64 && !ty.is_float() && !ty.is_vector() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_32(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 32 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_32_or_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 32 || ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_8_or_16(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 8 || ty.bits() == 16 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn int_fits_in_32(&mut self, ty: Type) -> Option<Type> {
            match ty {
                I8 | I16 | I32 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn ty_int_ref_64(&mut self, ty: Type) -> Option<Type> {
            match ty {
                I64 | R64 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn ty_int(&mut self, ty: Type) -> Option<Type> {
            ty.is_int().then(|| ty)
        }

        #[inline]
        fn ty_scalar_float(&mut self, ty: Type) -> Option<Type> {
            match ty {
                F32 | F64 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn ty_float_or_vec(&mut self, ty: Type) -> Option<Type> {
            match ty {
                F32 | F64 => Some(ty),
                ty if ty.is_vector() => Some(ty),
                _ => None,
            }
        }

        fn ty_vector_float(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.lane_type().is_float() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vector_not_float(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && !ty.lane_type().is_float() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec64_ctor(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec64(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec128(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 128 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn_vec64(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && dynamic_to_fixed(ty).bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn_vec128(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && dynamic_to_fixed(ty).bits() == 128 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec64_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 64 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_vec128_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 128 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn u64_from_imm64(&mut self, imm: Imm64) -> u64 {
            imm.bits() as u64
        }

        #[inline]
        fn u64_from_bool(&mut self, b: bool) -> u64 {
            if b {
                u64::MAX
            } else {
                0
            }
        }

        #[inline]
        fn multi_lane(&mut self, ty: Type) -> Option<(u32, u32)> {
            if ty.lane_count() > 1 {
                Some((ty.lane_bits(), ty.lane_count()))
            } else {
                None
            }
        }

        #[inline]
        fn dynamic_lane(&mut self, ty: Type) -> Option<(u32, u32)> {
            if ty.is_dynamic_vector() {
                Some((ty.lane_bits(), ty.min_lane_count()))
            } else {
                None
            }
        }

        #[inline]
        fn dynamic_int_lane(&mut self, ty: Type) -> Option<u32> {
            if ty.is_dynamic_vector() && crate::machinst::ty_has_int_representation(ty.lane_type())
            {
                Some(ty.lane_bits())
            } else {
                None
            }
        }

        #[inline]
        fn dynamic_fp_lane(&mut self, ty: Type) -> Option<u32> {
            if ty.is_dynamic_vector()
                && crate::machinst::ty_has_float_or_vec_representation(ty.lane_type())
            {
                Some(ty.lane_bits())
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn64_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && ty.min_bits() == 64 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_dyn128_int(&mut self, ty: Type) -> Option<Type> {
            if ty.is_dynamic_vector() && ty.min_bits() == 128 && ty.lane_type().is_int() {
                Some(ty)
            } else {
                None
            }
        }

        fn u64_from_ieee32(&mut self, val: Ieee32) -> u64 {
            val.bits().into()
        }

        fn u64_from_ieee64(&mut self, val: Ieee64) -> u64 {
            val.bits()
        }

        fn u8_from_uimm8(&mut self, val: Uimm8) -> u8 {
            val
        }

        fn not_vec32x2(&mut self, ty: Type) -> Option<Type> {
            if ty.lane_bits() == 32 && ty.lane_count() == 2 {
                None
            } else {
                Some(ty)
            }
        }

        fn not_i64x2(&mut self, ty: Type) -> Option<()> {
            if ty == I64X2 {
                None
            } else {
                Some(())
            }
        }

        fn trap_code_division_by_zero(&mut self) -> TrapCode {
            TrapCode::IntegerDivisionByZero
        }

        fn trap_code_integer_overflow(&mut self) -> TrapCode {
            TrapCode::IntegerOverflow
        }

        fn trap_code_bad_conversion_to_integer(&mut self) -> TrapCode {
            TrapCode::BadConversionToInteger
        }

        fn nonzero_u64_from_imm64(&mut self, val: Imm64) -> Option<u64> {
            match val.bits() {
                0 => None,
                n => Some(n as u64),
            }
        }

        #[inline]
        fn u32_add(&mut self, a: u32, b: u32) -> u32 {
            a.wrapping_add(b)
        }

        #[inline]
        fn s32_add_fallible(&mut self, a: u32, b: u32) -> Option<u32> {
            let a = a as i32;
            let b = b as i32;
            a.checked_add(b).map(|sum| sum as u32)
        }

        #[inline]
        fn u32_nonnegative(&mut self, x: u32) -> Option<u32> {
            if (x as i32) >= 0 {
                Some(x)
            } else {
                None
            }
        }

        #[inline]
        fn u32_lteq(&mut self, a: u32, b: u32) -> Option<()> {
            if a <= b {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn u8_lteq(&mut self, a: u8, b: u8) -> Option<()> {
            if a <= b {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn u8_lt(&mut self, a: u8, b: u8) -> Option<()> {
            if a < b {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn imm64(&mut self, x: u64) -> Option<Imm64> {
            Some(Imm64::new(x as i64))
        }

        #[inline]
        fn simm32(&mut self, x: Imm64) -> Option<u32> {
            let x64: i64 = x.into();
            let x32: i32 = x64.try_into().ok()?;
            Some(x32 as u32)
        }

        #[inline]
        fn uimm8(&mut self, x: Imm64) -> Option<u8> {
            let x64: i64 = x.into();
            let x8: u8 = x64.try_into().ok()?;
            Some(x8)
        }

        #[inline]
        fn offset32(&mut self, x: Offset32) -> Option<u32> {
            let x: i32 = x.into();
            Some(x as u32)
        }

        #[inline]
        fn u8_and(&mut self, a: u8, b: u8) -> u8 {
            a & b
        }

        #[inline]
        fn lane_type(&mut self, ty: Type) -> Type {
            ty.lane_type()
        }

        #[inline]
        fn offset32_to_u32(&mut self, offset: Offset32) -> u32 {
            let offset: i32 = offset.into();
            offset as u32
        }

        fn range(&mut self, start: usize, end: usize) -> Range {
            (start, end)
        }

        fn range_view(&mut self, (start, end): Range) -> RangeView {
            if start >= end {
                RangeView::Empty
            } else {
                RangeView::NonEmpty {
                    index: start,
                    rest: (start + 1, end),
                }
            }
        }

        #[inline]
        fn mem_flags_trusted(&mut self) -> MemFlags {
            MemFlags::trusted()
        }

        #[inline]
        fn intcc_unsigned(&mut self, x: &IntCC) -> IntCC {
            x.unsigned()
        }

        #[inline]
        fn signed_cond_code(&mut self, cc: &condcodes::IntCC) -> Option<condcodes::IntCC> {
            match cc {
                IntCC::Equal
                | IntCC::UnsignedGreaterThanOrEqual
                | IntCC::UnsignedGreaterThan
                | IntCC::UnsignedLessThanOrEqual
                | IntCC::UnsignedLessThan
                | IntCC::NotEqual => None,
                IntCC::SignedGreaterThanOrEqual
                | IntCC::SignedGreaterThan
                | IntCC::SignedLessThanOrEqual
                | IntCC::SignedLessThan => Some(*cc),
            }
        }
    };
}
