mod byte_list;
mod byte_vector;

use std::fmt;

fn write_bytes_to_lower_hex<T: AsRef<[u8]>>(f: &mut fmt::Formatter<'_>, data: T) -> fmt::Result {
    if f.alternate() {
        write!(f, "0x")?;
    }
    for i in data.as_ref() {
        write!(f, "{i:02x}")?;
    }
    Ok(())
}

pub use byte_list::ByteList;
pub use byte_vector::ByteVector;
