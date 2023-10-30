use crate::iter::Bytes;

pub enum Scan {
    /// Returned when an implementation finds a noteworthy token.
    Found,
    /// Returned when an implementation couldn't keep running because the input was too short.
    TooShort,
}


pub unsafe fn parse_uri_batch_32(bytes: &mut Bytes) -> Scan {
    while bytes.as_ref().len() >= 32 {
        let advance = match_url_char_32_avx(bytes.as_ref());
        bytes.advance(advance);

        if advance != 32 {
            return Scan::Found;
        }
    }
    Scan::TooShort
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
#[allow(non_snake_case, overflowing_literals)]
unsafe fn match_url_char_32_avx(buf: &[u8]) -> usize {
    debug_assert!(buf.len() >= 32);

    /*
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    */
    use core::arch::x86_64::*;

    let ptr = buf.as_ptr();

    let LSH: __m256i = _mm256_set1_epi8(0x0f);

    // See comment in sse42::match_url_char_16_sse.

    let URI: __m256i = _mm256_setr_epi8(
        0xf8, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc,
        0xfc, 0xfc, 0xfc, 0xfc, 0xf4, 0xfc, 0xf4, 0x7c,
        0xf8, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc,
        0xfc, 0xfc, 0xfc, 0xfc, 0xf4, 0xfc, 0xf4, 0x7c,
    );
    let ARF: __m256i = _mm256_setr_epi8(
        0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    );

    let data = _mm256_lddqu_si256(ptr as *const _);
    let rbms = _mm256_shuffle_epi8(URI, data);
    let cols = _mm256_and_si256(LSH, _mm256_srli_epi16(data, 4));
    let bits = _mm256_and_si256(_mm256_shuffle_epi8(ARF, cols), rbms);

    let v = _mm256_cmpeq_epi8(bits, _mm256_setzero_si256());
    let r = 0xffff_ffff_0000_0000 | _mm256_movemask_epi8(v) as u64;

    _tzcnt_u64(r) as usize
}

#[cfg(target_arch = "x86")]
unsafe fn match_url_char_32_avx(_: &[u8]) -> usize {
    unreachable!("AVX2 detection should be disabled for x86");
}

pub unsafe fn match_header_value_batch_32(bytes: &mut Bytes) -> Scan {
    while bytes.as_ref().len() >= 32 {
        let advance = match_header_value_char_32_avx(bytes.as_ref());
        bytes.advance(advance);

        if advance != 32 {
            return Scan::Found;
        }
    }
    Scan::TooShort
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline]
#[allow(non_snake_case)]
unsafe fn match_header_value_char_32_avx(buf: &[u8]) -> usize {
    debug_assert!(buf.len() >= 32);

    /*
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    */
    use core::arch::x86_64::*;

    let ptr = buf.as_ptr();

    // %x09 %x20-%x7e %x80-%xff
    let TAB: __m256i = _mm256_set1_epi8(0x09);
    let DEL: __m256i = _mm256_set1_epi8(0x7f);
    let LOW: __m256i = _mm256_set1_epi8(0x20);

    let dat = _mm256_lddqu_si256(ptr as *const _);
    // unsigned comparison dat >= LOW
    let low = _mm256_cmpeq_epi8(_mm256_max_epu8(dat, LOW), dat);
    let tab = _mm256_cmpeq_epi8(dat, TAB);
    let del = _mm256_cmpeq_epi8(dat, DEL);
    let bit = _mm256_andnot_si256(del, _mm256_or_si256(low, tab));
    let rev = _mm256_cmpeq_epi8(bit, _mm256_setzero_si256());
    let res = 0xffff_ffff_0000_0000 | _mm256_movemask_epi8(rev) as u64;

    _tzcnt_u64(res) as usize
}

#[cfg(target_arch = "x86")]
unsafe fn match_header_value_char_32_avx(_: &[u8]) -> usize {
    unreachable!("AVX2 detection should be disabled for x86");
}

#[test]
fn avx2_code_matches_uri_chars_table() {
    match super::detect() {
        super::AVX_2 | super::AVX_2_AND_SSE_42 => {},
        _ => return,
    }

    unsafe {
        assert!(byte_is_allowed(b'_', parse_uri_batch_32));

        for (b, allowed) in crate::URI_MAP.iter().cloned().enumerate() {
            assert_eq!(
                byte_is_allowed(b as u8, parse_uri_batch_32), allowed,
                "byte_is_allowed({:?}) should be {:?}", b, allowed,
            );
        }
    }
}

#[test]
fn avx2_code_matches_header_value_chars_table() {
    match super::detect() {
        super::AVX_2 | super::AVX_2_AND_SSE_42 => {},
        _ => return,
    }

    unsafe {
        assert!(byte_is_allowed(b'_', match_header_value_batch_32));

        for (b, allowed) in crate::HEADER_VALUE_MAP.iter().cloned().enumerate() {
            assert_eq!(
                byte_is_allowed(b as u8, match_header_value_batch_32), allowed,
                "byte_is_allowed({:?}) should be {:?}", b, allowed,
            );
        }
    }
}

#[cfg(test)]
unsafe fn byte_is_allowed(byte: u8, f: unsafe fn(bytes: &mut Bytes<'_>) -> Scan) -> bool {
    let slice = [
        b'_', b'_', b'_', b'_',
        b'_', b'_', b'_', b'_',
        b'_', b'_', b'_', b'_',
        b'_', b'_', b'_', b'_',
        b'_', b'_', b'_', b'_',
        b'_', b'_', b'_', b'_',
        b'_', b'_', byte, b'_',
        b'_', b'_', b'_', b'_',
    ];
    let mut bytes = Bytes::new(&slice);

    f(&mut bytes);

    match bytes.pos() {
        32 => true,
        26 => false,
        _ => unreachable!(),
    }
}
