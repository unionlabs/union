use std::{path::PathBuf, process};

use git2::{Oid, Repository};

use argh::FromArgs;

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "clone")]
/// Clone
pub struct CloneCmd {
    /// commit to checkout for the SDK repo
    #[argh(option, short = 'c')]
    sdk_commit: Option<String>,

    /// tag to checkout for the IBC-go repo
    #[argh(option, short = 't')]
    sdk_tag: Option<String>,

    /// commit to checkout
    #[argh(option, short = 'i')]
    ibc_go_commit: Option<String>,

    /// commit to checkout for the SDK repo
    #[argh(option, short = 's')]
    ics_commit: Option<String>,

    /// where to checkout the repository
    #[argh(option, short = 'o')]
    out: PathBuf,
}

pub const COSMOS_SDK_URL: &str = "https://github.com/cosmos/cosmos-sdk";
pub const IBC_GO_URL: &str = "https://github.com/cosmos/ibc-go";
pub const ICS_URL: &str = "https://github.com/cosmos/interchain-security";

impl CloneCmd {
    pub fn validate(&self) {
        if self.sdk_commit.is_some() && self.sdk_tag.is_some() {
            println!("[error] The --sdk-commit and --sdk-tag options are mutually exclusive.");
            process::exit(1);
        }
    }

    pub fn sdk_subdir(&self) -> PathBuf {
        let mut sdk_path = self.out.clone();
        sdk_path.push("sdk/");
        sdk_path
    }

    pub fn ibc_subdir(&self) -> PathBuf {
        let mut ibc_path = self.out.clone();
        ibc_path.push("ibc/");
        ibc_path
    }

    pub fn ics_subdir(&self) -> PathBuf {
        let mut ics_path = self.out.clone();
        ics_path.push("ics/");
        ics_path
    }

    pub fn run(&self) {
        self.validate();

        let sdk_path = self.sdk_subdir();
        let sdk_repo = if sdk_path.exists() {
            println!(
                "[info ] Found Cosmos SDK source at '{}'",
                sdk_path.display()
            );

            Repository::open(&sdk_path).unwrap_or_else(|e| {
                println!("[error] Failed to open repository: {}", e);
                process::exit(1)
            })
        } else {
            println!("[info ] Cloning cosmos/cosmos-sdk repository...");

            let repo = Repository::clone(COSMOS_SDK_URL, &sdk_path).unwrap_or_else(|e| {
                println!("[error] Failed to clone the SDK repository: {}", e);
                process::exit(1)
            });

            println!("[info ] Cloned at '{}'", sdk_path.display());

            repo
        };

        if let Some(ref rev) = self.sdk_commit {
            checkout_commit(&sdk_repo, rev).unwrap_or_else(|e| {
                println!("[error] Failed to checkout SDK commit {}: {}", rev, e);
                process::exit(1)
            });
        } else if let Some(ref tag) = self.sdk_tag {
            checkout_tag(&sdk_repo, tag).unwrap_or_else(|e| {
                println!("[error] Failed to checkout SDK tag {}: {}", tag, e);
                process::exit(1)
            });
        }

        println!("[info ] Cloning cosmos/ibc-go repository...");

        match &self.ibc_go_commit {
            Some(ibc_go_commit) => {
                let ibc_path = self.ibc_subdir();
                let ibc_repo = if ibc_path.exists() {
                    println!("[info ] Found IBC Go source at '{}'", ibc_path.display());

                    Repository::open(&ibc_path).unwrap_or_else(|e| {
                        println!("[error] Failed to open repository: {}", e);
                        process::exit(1)
                    })
                } else {
                    Repository::clone(IBC_GO_URL, &ibc_path).unwrap_or_else(|e| {
                        println!("[error] Failed to clone the IBC Go repository: {}", e);
                        process::exit(1)
                    })
                };

                println!("[info ] Cloned at '{}'", ibc_path.display());
                checkout_commit(&ibc_repo, ibc_go_commit).unwrap_or_else(|e| {
                    println!(
                        "[error] Failed to checkout IBC Go commit {}: {}",
                        ibc_go_commit, e
                    );
                    process::exit(1)
                });
            }
            None => {
                println!(
                    "[info ] No `-i`/`--ibc_go_commit` option passed. Skipping the IBC Go repo."
                )
            }
        }
        println!("[info ] Cloning cosmos/ics repository...");

        match &self.ics_commit {
            Some(ics_commit) => {
                let ics_path = self.ics_subdir();
                let ics_repo = if ics_path.exists() {
                    println!("[info ] Found ICS source at '{}'", ics_path.display());

                    Repository::open(&ics_path).unwrap_or_else(|e| {
                        println!("[error] Failed to open repository: {}", e);
                        process::exit(1)
                    })
                } else {
                    Repository::clone(ICS_URL, &ics_path).unwrap_or_else(|e| {
                        println!("[error] Failed to clone the ICS repository: {}", e);
                        process::exit(1)
                    })
                };

                println!("[info ] Cloned at '{}'", ics_path.display());
                checkout_commit(&ics_repo, ics_commit).unwrap_or_else(|e| {
                    println!(
                        "[error] Failed to checkout ICS commit {}: {}",
                        ics_commit, e
                    );
                    process::exit(1)
                });
            }
            None => {
                println!("[info ] No `-i`/`--ics_commit` option passed. Skipping the ICS repo.")
            }
        }
    }
}

fn checkout_commit(repo: &Repository, rev: &str) -> Result<(), git2::Error> {
    let oid = Oid::from_str(rev)?;
    let commit = repo.find_commit(oid)?;

    // Create a new branch `rev` that points to `commit`
    repo.branch(rev, &commit, true)?;

    // Checkout the newly created branch
    let treeish = format!("refs/heads/{}", rev);
    let object = repo.revparse_single(&treeish)?;
    repo.checkout_tree(&object, None)?;
    repo.set_head(&treeish)?;

    println!("[info ] Checked out commit {}", rev);

    Ok(())
}

fn checkout_tag(repo: &Repository, tag_name: &str) -> Result<(), git2::Error> {
    // Find a tag with name `tag_name`
    let tag = repo
        .references()?
        .flatten()
        .filter(|r| r.is_tag())
        .flat_map(|r| r.peel_to_tag())
        .find(|t| t.name() == Some(tag_name));

    if let Some(tag) = tag {
        // Get the commit this tag points to
        let target_oid = tag.target()?.id();
        let commit = repo.find_commit(target_oid)?;

        // Create a new branch `tag_name` that points to `commit`
        repo.branch(tag_name, &commit, true)?;

        // Checkout the newly created branch
        let rev = format!("refs/heads/{}", tag_name);
        let obj = repo.revparse_single(&rev)?;
        repo.checkout_tree(&obj, None)?;
        repo.set_head(&rev)?;

        println!("[info ] Checked out tag {}", tag_name);
    } else {
        println!("[error] Could not find tag {}", tag_name);
        process::exit(1);
    }

    Ok(())
}
