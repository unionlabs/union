use clap::{Parser, Subcommand};
use mpc_shared::{phase2_verify, supabase::SupabaseMPCApi};

const SUPABASE_PROJECT: &str = "https://bffcolwcakqrhlznyjns.supabase.co";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Start {
        #[arg(short, long)]
        jwt: String,
        #[arg(short, long)]
        api_key: String,
    },
}

#[derive(thiserror::Error, Debug, Clone)]
enum Error {
    #[error("current contributor not found.")]
    ContributorNotFound,
    #[error("current payload not found.")]
    CurrentPayloadNotFound,
    #[error("next payload not found.")]
    NextPayloadNotFound,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    match args.command {
        Command::Start { jwt, api_key } => {
            let client = SupabaseMPCApi::new(SUPABASE_PROJECT.into(), api_key, jwt);
            let progress = |percent| async move { println!("downloaded: {:.2}%", percent) };
            loop {
                println!("downloading current payload...");
                let current_payload = client
                    .current_payload()
                    .await?
                    .ok_or(Error::CurrentPayloadNotFound)?;
                let payload_current = client
                    .download_payload(&current_payload.id, &current_payload.id, progress)
                    .await?;
                let current_contributor = {
                    match client
                        .current_contributor()
                        .await?
                        .ok_or(Error::ContributorNotFound)
                    {
                        Ok(contributor) => {
                            println!("awaiting contribution of {}...", &contributor.id);
                            if client.contribution_submitted(&contributor.id).await? {
                                contributor
                            } else {
                                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                                continue;
                            }
                        }
                        Err(_) => {
                            println!("awaiting contributor to join queue...");
                            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                            continue;
                        }
                    }
                };
                println!("detected contribution submission, downloading...");
                let next_payload = client
                    .contributor_payload(&current_contributor.id)
                    .await?
                    .ok_or(Error::NextPayloadNotFound)?;
                let payload_next = client
                    .download_payload(&next_payload.id, &next_payload.id, progress)
                    .await?;
                println!("verifying payload...");
                if phase2_verify(&payload_current, &payload_next).is_ok() {
                    println!("verification succeeded.");
                    client
                        .insert_contribution(current_contributor.id.clone(), true)
                        .await?;
                    tokio::fs::remove_file(&current_payload.id).await?;
                } else {
                    println!("verification failed.");
                    client
                        .insert_contribution(current_contributor.id.clone(), false)
                        .await?;
                }
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        }
    }
}
