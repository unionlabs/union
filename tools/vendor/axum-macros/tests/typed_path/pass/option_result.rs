use axum_extra::routing::{TypedPath, RouterExt};
use axum::{extract::rejection::PathRejection, http::StatusCode};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/users/:id")]
struct UsersShow {
    id: String,
}

async fn option_handler(_: Option<UsersShow>) {}

async fn result_handler(_: Result<UsersShow, PathRejection>) {}

#[derive(TypedPath, Deserialize)]
#[typed_path("/users")]
struct UsersIndex;

#[axum_macros::debug_handler]
async fn result_handler_unit_struct(_: Result<UsersIndex, StatusCode>) {}

fn main() {
    axum::Router::<axum::body::Body>::new()
        .typed_get(option_handler)
        .typed_post(result_handler)
        .typed_post(result_handler_unit_struct);
}
