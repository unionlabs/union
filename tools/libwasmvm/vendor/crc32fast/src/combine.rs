const GF2_DIM: usize = 32;

fn gf2_matrix_times(mat: &[u32; GF2_DIM], mut vec: u32) -> u32 {
    let mut sum = 0;
    let mut idx = 0;
    while vec > 0 {
        if vec & 1 == 1 {
            sum ^= mat[idx];
        }
        vec >>= 1;
        idx += 1;
    }
    return sum;
}

fn gf2_matrix_square(square: &mut [u32; GF2_DIM], mat: &[u32; GF2_DIM]) {
    for n in 0..GF2_DIM {
        square[n] = gf2_matrix_times(mat, mat[n]);
    }
}

pub(crate) fn combine(mut crc1: u32, crc2: u32, mut len2: u64) -> u32 {
    let mut row: u32;
    let mut even = [0u32; GF2_DIM]; /* even-power-of-two zeros operator */
    let mut odd = [0u32; GF2_DIM]; /* odd-power-of-two zeros operator */

    /* degenerate case (also disallow negative lengths) */
    if len2 <= 0 {
        return crc1;
    }

    /* put operator for one zero bit in odd */
    odd[0] = 0xedb88320; /* CRC-32 polynomial */
    row = 1;
    for n in 1..GF2_DIM {
        odd[n] = row;
        row <<= 1;
    }

    /* put operator for two zero bits in even */
    gf2_matrix_square(&mut even, &odd);

    /* put operator for four zero bits in odd */
    gf2_matrix_square(&mut odd, &even);

    /* apply len2 zeros to crc1 (first square will put the operator for one
       zero byte, eight zero bits, in even) */
    loop {
        /* apply zeros operator for this bit of len2 */
        gf2_matrix_square(&mut even, &odd);
        if len2 & 1 == 1 {
            crc1 = gf2_matrix_times(&even, crc1);
        }
        len2 >>= 1;

        /* if no more bits set, then done */
        if len2 == 0 {
            break;
        }

        /* another iteration of the loop with odd and even swapped */
        gf2_matrix_square(&mut odd, &even);
        if len2 & 1 == 1 {
            crc1 = gf2_matrix_times(&odd, crc1);
        }
        len2 >>= 1;

        /* if no more bits set, then done */
        if len2 == 0 {
            break;
        }
    }

    /* return combined crc */
    crc1 ^= crc2;
    return crc1;
}
