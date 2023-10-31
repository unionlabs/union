pub fn main() {
    eprintln!("`check_contract` has been removed from `cosmwasm-vm` examples - please use `cosmwasm-check` instead.");
    eprintln!("See https://crates.io/crates/cosmwasm-check");
    eprintln!();
    eprintln!("> cargo install cosmwasm-check");
    eprintln!("> cosmwasm-check --help");
    eprintln!();
    std::process::exit(74);
}
