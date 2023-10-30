use mac_address::MacAddressIterator;

fn main() {
    for addr in MacAddressIterator::new().unwrap() {
        println!("{}", addr);
    }
}
