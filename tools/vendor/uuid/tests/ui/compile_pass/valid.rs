use uuid::{uuid, Uuid};

const _: Uuid = uuid!("00000000000000000000000000000000");
const _: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
const _: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
const _: Uuid = uuid!("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4");
const _: Uuid = uuid!("67e5504410b1426f9247bb680e5fe0c8");
const _: Uuid = uuid!("01020304-1112-2122-3132-414243444546");
const _: Uuid = uuid!("urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8");

// Nil
const _: Uuid = uuid!("00000000000000000000000000000000");
const _: Uuid = uuid!("00000000-0000-0000-0000-000000000000");

// valid hyphenated
const _: Uuid = uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
// valid short
const _: Uuid = uuid!("67e5504410b1426f9247bb680e5fe0c8");

fn main() {}
