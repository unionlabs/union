# Introduction

This project contains the client and coordinator to conduct Groth16 multi-party computation for the circuit SRS.
Three components are in play:
- Supabase : host the state machine in postgresql and exposes api and storage services to upload contributions.
- Coordinator: contact Supabase and verify contribution to step the state machine.
- Client: pure function that accepts the current contributor id and generate then upload a contribution payload.

## Supabase

Hosts the database, storage services and state machine of the MPC round. Provides instant API on top of them.

## Coordinator

The coordinator is in charge of verifying contributions. When a contribution is deemed valid, it dispatches the value to Supabase (insert an entry), effectively stepping the MPC state machine.

## Client

Exposes an API to contribute at `localhost:4919`:
- `OPTIONS /contribute`
- `POST /contribute` a `Contribute` object in body. Returns :
  - a `202 Accepted` if the contribution started.
  - a `503 Unavailable` if the client is busy (likely already contributing).
- `GET /contribute` returns :
  - a `200 Ok` if everything is ok with the body containing an encoded `Status` representing the client status (idle, contributing etc...).
  - a `500 InternalServerError` if the contribution failed unexpectedly, the body contains the error message.

### Structures

#### Contribute
```json
{
    "supabase_project": "<supabase_project_url>",
    "bucket": "<supabase_bucket_to_push_contribution>",
    "jwt": "<supabase_logged_in_user_jwt>",
    "api_key": "<supabase_anon_api_key>",
    "contributor_id": "<logged_in_user_uuid>",
    "payload_id": "<logged_in_user_payload_uuid>"
}
```

#### Status
```rust
#[serde(rename_all = "camelCase")]
pub enum Status {
    Idle,
    Initializing,
    DownloadStarted(String),
    Downloading(String, u8),
    DownloadEnded(String),
    ContributionStarted,
    ContributionEnded,
    UploadStarted(String),
    UploadEnded(String),
    Failed(String),
    Successful,
}
```
