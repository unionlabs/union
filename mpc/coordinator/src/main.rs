use clap::{Parser, Subcommand};
use mpc_shared::{phase2_verify, signed_message, supabase::SupabaseMPCApi};
use pgp::{cleartext::CleartextSignedMessage, Deserializable, SignedPublicKey};

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
        url: String,
        #[arg(short, long)]
        jwt: String,
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
    #[error("contributor signature not found")]
    ContributorSignatureNotFound,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    match args.command {
        Command::Start { url, jwt } => {
            let client = SupabaseMPCApi::new(url, jwt.clone(), jwt);
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
                let next_payload = client
                    .download_payload(&next_payload.id, &next_payload.id, progress)
                    .await?;
                println!("verifying signature...");
                let contribution_signature = client
                    .contributor_signature(&current_contributor.id)
                    .await?
                    .ok_or(Error::ContributorSignatureNotFound)?;
                let signed_public_key = SignedPublicKey::from_armor_single::<&[u8]>(
                    hex::decode(contribution_signature.public_key)
                        .expect("impossible")
                        .as_ref(),
                )
                .expect("impossible")
                .0;

                // Last bytes are the sh256 of the whole contrib
                let next_payload_hash = &next_payload[&next_payload.len() - 32..];

                let public_key_is_valid = signed_public_key.verify().is_ok();
                if !public_key_is_valid {
                    println!("public key is invalid");
                }

                let raw_signature =
                    hex::decode(&contribution_signature.signature).expect("impossible");
                let signature = CleartextSignedMessage::from_armor::<&[u8]>(raw_signature.as_ref())
                    .expect("impossible")
                    .0;

                let signature_matches = signature.signed_text()
                    == signed_message(&current_payload.id, &hex::encode(next_payload_hash));
                if !signature_matches {
                    println!("signature signed text mismatch");
                }

                let signature_is_valid = signature.verify(&signed_public_key).is_ok();
                if !signature_is_valid {
                    println!("contribution signature is invalid");
                }

                println!("verifying payload...");
                let contribution_is_valid = phase2_verify(&payload_current, &next_payload).is_ok();
                if !contribution_is_valid {
                    println!("contribution is invalid");
                }

                if public_key_is_valid
                    && signature_matches
                    && signature_is_valid
                    && contribution_is_valid
                {
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
