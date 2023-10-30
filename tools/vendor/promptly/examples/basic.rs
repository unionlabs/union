use promptly::{prompt, prompt_default, prompt_opt, Promptable};
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;

fn test<P: Promptable + Debug>(ty: &str, default: P) -> Result<(), Box<dyn Error>> {
    let res = prompt::<P, _>(ty)?;
    println!("=> {:?}", res);
    let res = prompt_opt::<P, _>(format!("Option<{}>", ty))?;
    println!("=> {:?}", res);
    let res = prompt_default::<P, _>(ty, default)?;
    println!("=> {:?}", res);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    test::<String>("String", "DefaultValue".to_string())?;
    test::<u32>("u32", 0)?;
    test::<bool>("bool", false)?;
    test::<PathBuf>("PathBuf", PathBuf::from("/home"))?;

    Ok(())
}
