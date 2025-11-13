pub fn peel_u8(data: &mut &[u8]) -> Option<u8> {
    let out = data.get(0)?;

    *data = &data[1..];

    Some(*out)
}

pub fn peel_u32(data: &mut &[u8]) -> Option<u32> {
    let out = u32::from_le_bytes(data.get(0..4)?.try_into().unwrap());

    *data = &data[4..];

    Some(out)
}

pub fn peel_bytes<'a>(data: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = peel_u32(data)? as usize;

    let out = &data.get(..len)?;

    *data = &data[len..];

    Some(out)
}

pub fn peel_str<'a>(data: &mut &'a [u8]) -> Option<&'a str> {
    str::from_utf8(peel_bytes(data)?).ok()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn eat_bytes_works() {
        let i = &mut hex!("03000000" "aabbcc").as_slice();

        let o = peel_bytes(i).unwrap();

        assert!(i.is_empty());

        assert_eq!(o, hex!("aabbcc"));
    }
}
