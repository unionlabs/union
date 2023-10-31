use core::{arch::x86_64::*, cmp, mem::size_of};

use super::sse2;

const VECTOR_SIZE: usize = size_of::<__m256i>();
const VECTOR_ALIGN: usize = VECTOR_SIZE - 1;

// The number of bytes to loop at in one iteration of memchr/memrchr.
const LOOP_SIZE: usize = 4 * VECTOR_SIZE;

// The number of bytes to loop at in one iteration of memchr2/memrchr2 and
// memchr3/memrchr3. There was no observable difference between 128 and 64
// bytes in benchmarks. memchr3 in particular only gets a very slight speed up
// from the loop unrolling.
const LOOP_SIZE2: usize = 2 * VECTOR_SIZE;

#[target_feature(enable = "avx2")]
pub unsafe fn memchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    // For a high level explanation for how this algorithm works, see the
    // sse2 implementation. The avx implementation here is the same, but with
    // 256-bit vectors instead of 128-bit vectors.

    // This routine is called whenever a match is detected. It is specifically
    // marked as unlineable because it improves the codegen of the unrolled
    // loop below. Inlining this seems to cause codegen with some extra adds
    // and a load that aren't necessary. This seems to result in about a 10%
    // improvement for the memchr1/crate/huge/never benchmark.
    //
    // Interestingly, I couldn't observe a similar improvement for memrchr.
    #[cold]
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn matched(
        start_ptr: *const u8,
        ptr: *const u8,
        eqa: __m256i,
        eqb: __m256i,
        eqc: __m256i,
        eqd: __m256i,
    ) -> usize {
        let mut at = sub(ptr, start_ptr);
        let mask = _mm256_movemask_epi8(eqa);
        if mask != 0 {
            return at + forward_pos(mask);
        }

        at += VECTOR_SIZE;
        let mask = _mm256_movemask_epi8(eqb);
        if mask != 0 {
            return at + forward_pos(mask);
        }

        at += VECTOR_SIZE;
        let mask = _mm256_movemask_epi8(eqc);
        if mask != 0 {
            return at + forward_pos(mask);
        }

        at += VECTOR_SIZE;
        let mask = _mm256_movemask_epi8(eqd);
        debug_assert!(mask != 0);
        at + forward_pos(mask)
    }

    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = start_ptr;

    if haystack.len() < VECTOR_SIZE {
        // For small haystacks, defer to the SSE2 implementation. Codegen
        // suggests this completely avoids touching the AVX vectors.
        return sse2::memchr(n1, haystack);
    }

    let vn1 = _mm256_set1_epi8(n1 as i8);
    let loop_size = cmp::min(LOOP_SIZE, haystack.len());
    if let Some(i) = forward_search1(start_ptr, end_ptr, ptr, vn1) {
        return Some(i);
    }

    ptr = ptr.add(VECTOR_SIZE - (start_ptr as usize & VECTOR_ALIGN));
    debug_assert!(ptr > start_ptr && end_ptr.sub(VECTOR_SIZE) >= start_ptr);
    while loop_size == LOOP_SIZE && ptr <= end_ptr.sub(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

        let a = _mm256_load_si256(ptr as *const __m256i);
        let b = _mm256_load_si256(ptr.add(VECTOR_SIZE) as *const __m256i);
        let c = _mm256_load_si256(ptr.add(2 * VECTOR_SIZE) as *const __m256i);
        let d = _mm256_load_si256(ptr.add(3 * VECTOR_SIZE) as *const __m256i);
        let eqa = _mm256_cmpeq_epi8(vn1, a);
        let eqb = _mm256_cmpeq_epi8(vn1, b);
        let eqc = _mm256_cmpeq_epi8(vn1, c);
        let eqd = _mm256_cmpeq_epi8(vn1, d);
        let or1 = _mm256_or_si256(eqa, eqb);
        let or2 = _mm256_or_si256(eqc, eqd);
        let or3 = _mm256_or_si256(or1, or2);

        if _mm256_movemask_epi8(or3) != 0 {
            return Some(matched(start_ptr, ptr, eqa, eqb, eqc, eqd));
        }
        ptr = ptr.add(loop_size);
    }
    while ptr <= end_ptr.sub(VECTOR_SIZE) {
        debug_assert!(sub(end_ptr, ptr) >= VECTOR_SIZE);

        if let Some(i) = forward_search1(start_ptr, end_ptr, ptr, vn1) {
            return Some(i);
        }
        ptr = ptr.add(VECTOR_SIZE);
    }
    if ptr < end_ptr {
        debug_assert!(sub(end_ptr, ptr) < VECTOR_SIZE);
        ptr = ptr.sub(VECTOR_SIZE - sub(end_ptr, ptr));
        debug_assert_eq!(sub(end_ptr, ptr), VECTOR_SIZE);

        return forward_search1(start_ptr, end_ptr, ptr, vn1);
    }
    None
}

#[target_feature(enable = "avx2")]
pub unsafe fn memchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    #[cold]
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn matched(
        start_ptr: *const u8,
        ptr: *const u8,
        eqa1: __m256i,
        eqa2: __m256i,
        eqb1: __m256i,
        eqb2: __m256i,
    ) -> usize {
        let mut at = sub(ptr, start_ptr);
        let mask1 = _mm256_movemask_epi8(eqa1);
        let mask2 = _mm256_movemask_epi8(eqa2);
        if mask1 != 0 || mask2 != 0 {
            return at + forward_pos2(mask1, mask2);
        }

        at += VECTOR_SIZE;
        let mask1 = _mm256_movemask_epi8(eqb1);
        let mask2 = _mm256_movemask_epi8(eqb2);
        at + forward_pos2(mask1, mask2)
    }

    let vn1 = _mm256_set1_epi8(n1 as i8);
    let vn2 = _mm256_set1_epi8(n2 as i8);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_SIZE2, len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = start_ptr;

    if haystack.len() < VECTOR_SIZE {
        while ptr < end_ptr {
            if *ptr == n1 || *ptr == n2 {
                return Some(sub(ptr, start_ptr));
            }
            ptr = ptr.offset(1);
        }
        return None;
    }

    if let Some(i) = forward_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
        return Some(i);
    }

    ptr = ptr.add(VECTOR_SIZE - (start_ptr as usize & VECTOR_ALIGN));
    debug_assert!(ptr > start_ptr && end_ptr.sub(VECTOR_SIZE) >= start_ptr);
    while loop_size == LOOP_SIZE2 && ptr <= end_ptr.sub(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

        let a = _mm256_load_si256(ptr as *const __m256i);
        let b = _mm256_load_si256(ptr.add(VECTOR_SIZE) as *const __m256i);
        let eqa1 = _mm256_cmpeq_epi8(vn1, a);
        let eqb1 = _mm256_cmpeq_epi8(vn1, b);
        let eqa2 = _mm256_cmpeq_epi8(vn2, a);
        let eqb2 = _mm256_cmpeq_epi8(vn2, b);
        let or1 = _mm256_or_si256(eqa1, eqb1);
        let or2 = _mm256_or_si256(eqa2, eqb2);
        let or3 = _mm256_or_si256(or1, or2);
        if _mm256_movemask_epi8(or3) != 0 {
            return Some(matched(start_ptr, ptr, eqa1, eqa2, eqb1, eqb2));
        }
        ptr = ptr.add(loop_size);
    }
    while ptr <= end_ptr.sub(VECTOR_SIZE) {
        if let Some(i) = forward_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
            return Some(i);
        }
        ptr = ptr.add(VECTOR_SIZE);
    }
    if ptr < end_ptr {
        debug_assert!(sub(end_ptr, ptr) < VECTOR_SIZE);
        ptr = ptr.sub(VECTOR_SIZE - sub(end_ptr, ptr));
        debug_assert_eq!(sub(end_ptr, ptr), VECTOR_SIZE);

        return forward_search2(start_ptr, end_ptr, ptr, vn1, vn2);
    }
    None
}

#[target_feature(enable = "avx2")]
pub unsafe fn memchr3(
    n1: u8,
    n2: u8,
    n3: u8,
    haystack: &[u8],
) -> Option<usize> {
    #[cold]
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn matched(
        start_ptr: *const u8,
        ptr: *const u8,
        eqa1: __m256i,
        eqa2: __m256i,
        eqa3: __m256i,
        eqb1: __m256i,
        eqb2: __m256i,
        eqb3: __m256i,
    ) -> usize {
        let mut at = sub(ptr, start_ptr);
        let mask1 = _mm256_movemask_epi8(eqa1);
        let mask2 = _mm256_movemask_epi8(eqa2);
        let mask3 = _mm256_movemask_epi8(eqa3);
        if mask1 != 0 || mask2 != 0 || mask3 != 0 {
            return at + forward_pos3(mask1, mask2, mask3);
        }

        at += VECTOR_SIZE;
        let mask1 = _mm256_movemask_epi8(eqb1);
        let mask2 = _mm256_movemask_epi8(eqb2);
        let mask3 = _mm256_movemask_epi8(eqb3);
        at + forward_pos3(mask1, mask2, mask3)
    }

    let vn1 = _mm256_set1_epi8(n1 as i8);
    let vn2 = _mm256_set1_epi8(n2 as i8);
    let vn3 = _mm256_set1_epi8(n3 as i8);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_SIZE2, len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = start_ptr;

    if haystack.len() < VECTOR_SIZE {
        while ptr < end_ptr {
            if *ptr == n1 || *ptr == n2 || *ptr == n3 {
                return Some(sub(ptr, start_ptr));
            }
            ptr = ptr.offset(1);
        }
        return None;
    }

    if let Some(i) = forward_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3) {
        return Some(i);
    }

    ptr = ptr.add(VECTOR_SIZE - (start_ptr as usize & VECTOR_ALIGN));
    debug_assert!(ptr > start_ptr && end_ptr.sub(VECTOR_SIZE) >= start_ptr);
    while loop_size == LOOP_SIZE2 && ptr <= end_ptr.sub(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

        let a = _mm256_load_si256(ptr as *const __m256i);
        let b = _mm256_load_si256(ptr.add(VECTOR_SIZE) as *const __m256i);
        let eqa1 = _mm256_cmpeq_epi8(vn1, a);
        let eqb1 = _mm256_cmpeq_epi8(vn1, b);
        let eqa2 = _mm256_cmpeq_epi8(vn2, a);
        let eqb2 = _mm256_cmpeq_epi8(vn2, b);
        let eqa3 = _mm256_cmpeq_epi8(vn3, a);
        let eqb3 = _mm256_cmpeq_epi8(vn3, b);
        let or1 = _mm256_or_si256(eqa1, eqb1);
        let or2 = _mm256_or_si256(eqa2, eqb2);
        let or3 = _mm256_or_si256(eqa3, eqb3);
        let or4 = _mm256_or_si256(or1, or2);
        let or5 = _mm256_or_si256(or3, or4);
        if _mm256_movemask_epi8(or5) != 0 {
            return Some(matched(
                start_ptr, ptr, eqa1, eqa2, eqa3, eqb1, eqb2, eqb3,
            ));
        }
        ptr = ptr.add(loop_size);
    }
    while ptr <= end_ptr.sub(VECTOR_SIZE) {
        if let Some(i) =
            forward_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3)
        {
            return Some(i);
        }
        ptr = ptr.add(VECTOR_SIZE);
    }
    if ptr < end_ptr {
        debug_assert!(sub(end_ptr, ptr) < VECTOR_SIZE);
        ptr = ptr.sub(VECTOR_SIZE - sub(end_ptr, ptr));
        debug_assert_eq!(sub(end_ptr, ptr), VECTOR_SIZE);

        return forward_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3);
    }
    None
}

#[target_feature(enable = "avx2")]
pub unsafe fn memrchr(n1: u8, haystack: &[u8]) -> Option<usize> {
    let vn1 = _mm256_set1_epi8(n1 as i8);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_SIZE, len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = end_ptr;

    if haystack.len() < VECTOR_SIZE {
        while ptr > start_ptr {
            ptr = ptr.offset(-1);
            if *ptr == n1 {
                return Some(sub(ptr, start_ptr));
            }
        }
        return None;
    }

    ptr = ptr.sub(VECTOR_SIZE);
    if let Some(i) = reverse_search1(start_ptr, end_ptr, ptr, vn1) {
        return Some(i);
    }

    ptr = (end_ptr as usize & !VECTOR_ALIGN) as *const u8;
    debug_assert!(start_ptr <= ptr && ptr <= end_ptr);
    while loop_size == LOOP_SIZE && ptr >= start_ptr.add(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

        ptr = ptr.sub(loop_size);
        let a = _mm256_load_si256(ptr as *const __m256i);
        let b = _mm256_load_si256(ptr.add(VECTOR_SIZE) as *const __m256i);
        let c = _mm256_load_si256(ptr.add(2 * VECTOR_SIZE) as *const __m256i);
        let d = _mm256_load_si256(ptr.add(3 * VECTOR_SIZE) as *const __m256i);
        let eqa = _mm256_cmpeq_epi8(vn1, a);
        let eqb = _mm256_cmpeq_epi8(vn1, b);
        let eqc = _mm256_cmpeq_epi8(vn1, c);
        let eqd = _mm256_cmpeq_epi8(vn1, d);
        let or1 = _mm256_or_si256(eqa, eqb);
        let or2 = _mm256_or_si256(eqc, eqd);
        let or3 = _mm256_or_si256(or1, or2);
        if _mm256_movemask_epi8(or3) != 0 {
            let mut at = sub(ptr.add(3 * VECTOR_SIZE), start_ptr);
            let mask = _mm256_movemask_epi8(eqd);
            if mask != 0 {
                return Some(at + reverse_pos(mask));
            }

            at -= VECTOR_SIZE;
            let mask = _mm256_movemask_epi8(eqc);
            if mask != 0 {
                return Some(at + reverse_pos(mask));
            }

            at -= VECTOR_SIZE;
            let mask = _mm256_movemask_epi8(eqb);
            if mask != 0 {
                return Some(at + reverse_pos(mask));
            }

            at -= VECTOR_SIZE;
            let mask = _mm256_movemask_epi8(eqa);
            debug_assert!(mask != 0);
            return Some(at + reverse_pos(mask));
        }
    }
    while ptr >= start_ptr.add(VECTOR_SIZE) {
        ptr = ptr.sub(VECTOR_SIZE);
        if let Some(i) = reverse_search1(start_ptr, end_ptr, ptr, vn1) {
            return Some(i);
        }
    }
    if ptr > start_ptr {
        debug_assert!(sub(ptr, start_ptr) < VECTOR_SIZE);
        return reverse_search1(start_ptr, end_ptr, start_ptr, vn1);
    }
    None
}

#[target_feature(enable = "avx2")]
pub unsafe fn memrchr2(n1: u8, n2: u8, haystack: &[u8]) -> Option<usize> {
    let vn1 = _mm256_set1_epi8(n1 as i8);
    let vn2 = _mm256_set1_epi8(n2 as i8);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_SIZE2, len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = end_ptr;

    if haystack.len() < VECTOR_SIZE {
        while ptr > start_ptr {
            ptr = ptr.offset(-1);
            if *ptr == n1 || *ptr == n2 {
                return Some(sub(ptr, start_ptr));
            }
        }
        return None;
    }

    ptr = ptr.sub(VECTOR_SIZE);
    if let Some(i) = reverse_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
        return Some(i);
    }

    ptr = (end_ptr as usize & !VECTOR_ALIGN) as *const u8;
    debug_assert!(start_ptr <= ptr && ptr <= end_ptr);
    while loop_size == LOOP_SIZE2 && ptr >= start_ptr.add(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

        ptr = ptr.sub(loop_size);
        let a = _mm256_load_si256(ptr as *const __m256i);
        let b = _mm256_load_si256(ptr.add(VECTOR_SIZE) as *const __m256i);
        let eqa1 = _mm256_cmpeq_epi8(vn1, a);
        let eqb1 = _mm256_cmpeq_epi8(vn1, b);
        let eqa2 = _mm256_cmpeq_epi8(vn2, a);
        let eqb2 = _mm256_cmpeq_epi8(vn2, b);
        let or1 = _mm256_or_si256(eqa1, eqb1);
        let or2 = _mm256_or_si256(eqa2, eqb2);
        let or3 = _mm256_or_si256(or1, or2);
        if _mm256_movemask_epi8(or3) != 0 {
            let mut at = sub(ptr.add(VECTOR_SIZE), start_ptr);
            let mask1 = _mm256_movemask_epi8(eqb1);
            let mask2 = _mm256_movemask_epi8(eqb2);
            if mask1 != 0 || mask2 != 0 {
                return Some(at + reverse_pos2(mask1, mask2));
            }

            at -= VECTOR_SIZE;
            let mask1 = _mm256_movemask_epi8(eqa1);
            let mask2 = _mm256_movemask_epi8(eqa2);
            return Some(at + reverse_pos2(mask1, mask2));
        }
    }
    while ptr >= start_ptr.add(VECTOR_SIZE) {
        ptr = ptr.sub(VECTOR_SIZE);
        if let Some(i) = reverse_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
            return Some(i);
        }
    }
    if ptr > start_ptr {
        debug_assert!(sub(ptr, start_ptr) < VECTOR_SIZE);
        return reverse_search2(start_ptr, end_ptr, start_ptr, vn1, vn2);
    }
    None
}

#[target_feature(enable = "avx2")]
pub unsafe fn memrchr3(
    n1: u8,
    n2: u8,
    n3: u8,
    haystack: &[u8],
) -> Option<usize> {
    let vn1 = _mm256_set1_epi8(n1 as i8);
    let vn2 = _mm256_set1_epi8(n2 as i8);
    let vn3 = _mm256_set1_epi8(n3 as i8);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_SIZE2, len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = end_ptr;

    if haystack.len() < VECTOR_SIZE {
        while ptr > start_ptr {
            ptr = ptr.offset(-1);
            if *ptr == n1 || *ptr == n2 || *ptr == n3 {
                return Some(sub(ptr, start_ptr));
            }
        }
        return None;
    }

    ptr = ptr.sub(VECTOR_SIZE);
    if let Some(i) = reverse_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3) {
        return Some(i);
    }

    ptr = (end_ptr as usize & !VECTOR_ALIGN) as *const u8;
    debug_assert!(start_ptr <= ptr && ptr <= end_ptr);
    while loop_size == LOOP_SIZE2 && ptr >= start_ptr.add(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % VECTOR_SIZE);

        ptr = ptr.sub(loop_size);
        let a = _mm256_load_si256(ptr as *const __m256i);
        let b = _mm256_load_si256(ptr.add(VECTOR_SIZE) as *const __m256i);
        let eqa1 = _mm256_cmpeq_epi8(vn1, a);
        let eqb1 = _mm256_cmpeq_epi8(vn1, b);
        let eqa2 = _mm256_cmpeq_epi8(vn2, a);
        let eqb2 = _mm256_cmpeq_epi8(vn2, b);
        let eqa3 = _mm256_cmpeq_epi8(vn3, a);
        let eqb3 = _mm256_cmpeq_epi8(vn3, b);
        let or1 = _mm256_or_si256(eqa1, eqb1);
        let or2 = _mm256_or_si256(eqa2, eqb2);
        let or3 = _mm256_or_si256(eqa3, eqb3);
        let or4 = _mm256_or_si256(or1, or2);
        let or5 = _mm256_or_si256(or3, or4);
        if _mm256_movemask_epi8(or5) != 0 {
            let mut at = sub(ptr.add(VECTOR_SIZE), start_ptr);
            let mask1 = _mm256_movemask_epi8(eqb1);
            let mask2 = _mm256_movemask_epi8(eqb2);
            let mask3 = _mm256_movemask_epi8(eqb3);
            if mask1 != 0 || mask2 != 0 || mask3 != 0 {
                return Some(at + reverse_pos3(mask1, mask2, mask3));
            }

            at -= VECTOR_SIZE;
            let mask1 = _mm256_movemask_epi8(eqa1);
            let mask2 = _mm256_movemask_epi8(eqa2);
            let mask3 = _mm256_movemask_epi8(eqa3);
            return Some(at + reverse_pos3(mask1, mask2, mask3));
        }
    }
    while ptr >= start_ptr.add(VECTOR_SIZE) {
        ptr = ptr.sub(VECTOR_SIZE);
        if let Some(i) =
            reverse_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3)
        {
            return Some(i);
        }
    }
    if ptr > start_ptr {
        debug_assert!(sub(ptr, start_ptr) < VECTOR_SIZE);
        return reverse_search3(start_ptr, end_ptr, start_ptr, vn1, vn2, vn3);
    }
    None
}

#[target_feature(enable = "avx2")]
unsafe fn forward_search1(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: __m256i,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= VECTOR_SIZE);
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(VECTOR_SIZE));

    let chunk = _mm256_loadu_si256(ptr as *const __m256i);
    let mask = _mm256_movemask_epi8(_mm256_cmpeq_epi8(chunk, vn1));
    if mask != 0 {
        Some(sub(ptr, start_ptr) + forward_pos(mask))
    } else {
        None
    }
}

#[target_feature(enable = "avx2")]
unsafe fn forward_search2(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: __m256i,
    vn2: __m256i,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= VECTOR_SIZE);
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(VECTOR_SIZE));

    let chunk = _mm256_loadu_si256(ptr as *const __m256i);
    let eq1 = _mm256_cmpeq_epi8(chunk, vn1);
    let eq2 = _mm256_cmpeq_epi8(chunk, vn2);
    if _mm256_movemask_epi8(_mm256_or_si256(eq1, eq2)) != 0 {
        let mask1 = _mm256_movemask_epi8(eq1);
        let mask2 = _mm256_movemask_epi8(eq2);
        Some(sub(ptr, start_ptr) + forward_pos2(mask1, mask2))
    } else {
        None
    }
}

#[target_feature(enable = "avx2")]
unsafe fn forward_search3(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: __m256i,
    vn2: __m256i,
    vn3: __m256i,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= VECTOR_SIZE);
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(VECTOR_SIZE));

    let chunk = _mm256_loadu_si256(ptr as *const __m256i);
    let eq1 = _mm256_cmpeq_epi8(chunk, vn1);
    let eq2 = _mm256_cmpeq_epi8(chunk, vn2);
    let eq3 = _mm256_cmpeq_epi8(chunk, vn3);
    let or = _mm256_or_si256(eq1, eq2);
    if _mm256_movemask_epi8(_mm256_or_si256(or, eq3)) != 0 {
        let mask1 = _mm256_movemask_epi8(eq1);
        let mask2 = _mm256_movemask_epi8(eq2);
        let mask3 = _mm256_movemask_epi8(eq3);
        Some(sub(ptr, start_ptr) + forward_pos3(mask1, mask2, mask3))
    } else {
        None
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reverse_search1(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: __m256i,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= VECTOR_SIZE);
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(VECTOR_SIZE));

    let chunk = _mm256_loadu_si256(ptr as *const __m256i);
    let mask = _mm256_movemask_epi8(_mm256_cmpeq_epi8(vn1, chunk));
    if mask != 0 {
        Some(sub(ptr, start_ptr) + reverse_pos(mask))
    } else {
        None
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reverse_search2(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: __m256i,
    vn2: __m256i,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= VECTOR_SIZE);
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(VECTOR_SIZE));

    let chunk = _mm256_loadu_si256(ptr as *const __m256i);
    let eq1 = _mm256_cmpeq_epi8(chunk, vn1);
    let eq2 = _mm256_cmpeq_epi8(chunk, vn2);
    if _mm256_movemask_epi8(_mm256_or_si256(eq1, eq2)) != 0 {
        let mask1 = _mm256_movemask_epi8(eq1);
        let mask2 = _mm256_movemask_epi8(eq2);
        Some(sub(ptr, start_ptr) + reverse_pos2(mask1, mask2))
    } else {
        None
    }
}

#[target_feature(enable = "avx2")]
unsafe fn reverse_search3(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: __m256i,
    vn2: __m256i,
    vn3: __m256i,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= VECTOR_SIZE);
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(VECTOR_SIZE));

    let chunk = _mm256_loadu_si256(ptr as *const __m256i);
    let eq1 = _mm256_cmpeq_epi8(chunk, vn1);
    let eq2 = _mm256_cmpeq_epi8(chunk, vn2);
    let eq3 = _mm256_cmpeq_epi8(chunk, vn3);
    let or = _mm256_or_si256(eq1, eq2);
    if _mm256_movemask_epi8(_mm256_or_si256(or, eq3)) != 0 {
        let mask1 = _mm256_movemask_epi8(eq1);
        let mask2 = _mm256_movemask_epi8(eq2);
        let mask3 = _mm256_movemask_epi8(eq3);
        Some(sub(ptr, start_ptr) + reverse_pos3(mask1, mask2, mask3))
    } else {
        None
    }
}

/// Compute the position of the first matching byte from the given mask. The
/// position returned is always in the range [0, 31].
///
/// The mask given is expected to be the result of _mm256_movemask_epi8.
fn forward_pos(mask: i32) -> usize {
    // We are dealing with little endian here, where the most significant byte
    // is at a higher address. That means the least significant bit that is set
    // corresponds to the position of our first matching byte. That position
    // corresponds to the number of zeros after the least significant bit.
    mask.trailing_zeros() as usize
}

/// Compute the position of the first matching byte from the given masks. The
/// position returned is always in the range [0, 31]. Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm256_movemask_epi8,
/// where at least one of the masks is non-zero (i.e., indicates a match).
fn forward_pos2(mask1: i32, mask2: i32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0);

    forward_pos(mask1 | mask2)
}

/// Compute the position of the first matching byte from the given masks. The
/// position returned is always in the range [0, 31]. Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm256_movemask_epi8,
/// where at least one of the masks is non-zero (i.e., indicates a match).
fn forward_pos3(mask1: i32, mask2: i32, mask3: i32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0 || mask3 != 0);

    forward_pos(mask1 | mask2 | mask3)
}

/// Compute the position of the last matching byte from the given mask. The
/// position returned is always in the range [0, 31].
///
/// The mask given is expected to be the result of _mm256_movemask_epi8.
fn reverse_pos(mask: i32) -> usize {
    // We are dealing with little endian here, where the most significant byte
    // is at a higher address. That means the most significant bit that is set
    // corresponds to the position of our last matching byte. The position from
    // the end of the mask is therefore the number of leading zeros in a 32
    // bit integer, and the position from the start of the mask is therefore
    // 32 - (leading zeros) - 1.
    VECTOR_SIZE - (mask as u32).leading_zeros() as usize - 1
}

/// Compute the position of the last matching byte from the given masks. The
/// position returned is always in the range [0, 31]. Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm256_movemask_epi8,
/// where at least one of the masks is non-zero (i.e., indicates a match).
fn reverse_pos2(mask1: i32, mask2: i32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0);

    reverse_pos(mask1 | mask2)
}

/// Compute the position of the last matching byte from the given masks. The
/// position returned is always in the range [0, 31]. Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm256_movemask_epi8,
/// where at least one of the masks is non-zero (i.e., indicates a match).
fn reverse_pos3(mask1: i32, mask2: i32, mask3: i32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0 || mask3 != 0);

    reverse_pos(mask1 | mask2 | mask3)
}

/// Subtract `b` from `a` and return the difference. `a` should be greater than
/// or equal to `b`.
fn sub(a: *const u8, b: *const u8) -> usize {
    debug_assert!(a >= b);
    (a as usize) - (b as usize)
}
