use ibc_union_spec::ClientId;
use pinocchio::program_error::ProgramError;
use unionlabs_primitives::Bytes;

pub fn parse_string(data: &[u8]) -> Result<(usize, String), ProgramError> {
    let string_len = u32::from_le_bytes(data[0..4].try_into().unwrap());

    Ok((
        (string_len + 1) as usize,
        String::from_utf8(data[4..4 + (string_len as usize)].to_vec())
            .map_err(|_| ProgramError::InvalidArgument)?,
    ))
}

pub fn parse_bytes(data: &[u8]) -> (usize, Bytes) {
    let bytes_len = u32::from_le_bytes(data[0..4].try_into().unwrap());

    (
        (bytes_len + 1) as usize,
        Bytes::new(data[4..4 + (bytes_len as usize)].to_vec()),
    )
}

pub fn parse_client_id(data: &[u8]) -> Result<ClientId, ProgramError> {
    let client_id = u32::from_le_bytes(data[0..4].try_into().unwrap());

    ClientId::from_raw(client_id).ok_or(ProgramError::InvalidArgument)
}

pub fn write_client_id(data: &mut [u8], client_id: ClientId) {
    client_id
        .raw()
        .to_le_bytes()
        .into_iter()
        .enumerate()
        .for_each(|(i, e)| data[i] = e);
}
