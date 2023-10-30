use mac_address::mac_address_by_name;

fn main() {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let name = "eth0";

    #[cfg(any(target_os = "freebsd"))]
    let name = "em0";

    #[cfg(any(target_os = "openbsd"))]
    let name = "fxp0";

    #[cfg(target_os = "windows")]
    let name = "Ethernet";

    match mac_address_by_name(name) {
        Ok(Some(ma)) => {
            println!("MAC addr of {} = {}", name, ma);
            println!("bytes = {:?}", ma.bytes());
        }
        Ok(None) => println!("Interface \"{}\" not found", name),
        Err(e) => println!("{:?}", e),
    }
}
