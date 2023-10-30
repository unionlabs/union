use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    response::IntoResponse,
};
use axum_macros::debug_handler;

fn main() {}

#[debug_handler]
fn concrete_future() -> std::future::Ready<Result<impl IntoResponse, ()>> {
    std::future::ready(Ok(()))
}

#[debug_handler]
fn impl_future() -> impl std::future::Future<Output = Result<impl IntoResponse, ()>> {
    std::future::ready(Ok(()))
}

// === no args ===

#[debug_handler]
async fn handler_no_arg_one() -> Result<impl IntoResponse, ()> {
    Ok(())
}

#[debug_handler]
async fn handler_no_arg_two() -> Result<(), impl IntoResponse> {
    Err(())
}

#[debug_handler]
async fn handler_no_arg_three() -> Result<impl IntoResponse, impl IntoResponse> {
    Ok::<_, ()>(())
}

#[debug_handler]
async fn handler_no_arg_four() -> Result<impl IntoResponse, impl IntoResponse> {
    Err::<(), _>(())
}

// === args ===

#[debug_handler]
async fn handler_one(foo: String) -> Result<impl IntoResponse, ()> {
    dbg!(foo);
    Ok(())
}

#[debug_handler]
async fn handler_two(foo: String) -> Result<(), impl IntoResponse> {
    dbg!(foo);
    Err(())
}

#[debug_handler]
async fn handler_three(foo: String) -> Result<impl IntoResponse, impl IntoResponse> {
    dbg!(foo);
    Ok::<_, ()>(())
}

#[debug_handler]
async fn handler_four(foo: String) -> Result<impl IntoResponse, impl IntoResponse> {
    dbg!(foo);
    Err::<(), _>(())
}

// === no args with receiver ===

struct A;

impl A {
    #[debug_handler]
    async fn handler_no_arg_one(self) -> Result<impl IntoResponse, ()> {
        Ok(())
    }

    #[debug_handler]
    async fn handler_no_arg_two(self) -> Result<(), impl IntoResponse> {
        Err(())
    }

    #[debug_handler]
    async fn handler_no_arg_three(self) -> Result<impl IntoResponse, impl IntoResponse> {
        Ok::<_, ()>(())
    }

    #[debug_handler]
    async fn handler_no_arg_four(self) -> Result<impl IntoResponse, impl IntoResponse> {
        Err::<(), _>(())
    }
}

// === args with receiver ===

impl A {
    #[debug_handler]
    async fn handler_one(self, foo: String) -> Result<impl IntoResponse, ()> {
        dbg!(foo);
        Ok(())
    }

    #[debug_handler]
    async fn handler_two(self, foo: String) -> Result<(), impl IntoResponse> {
        dbg!(foo);
        Err(())
    }

    #[debug_handler]
    async fn handler_three(self, foo: String) -> Result<impl IntoResponse, impl IntoResponse> {
        dbg!(foo);
        Ok::<_, ()>(())
    }

    #[debug_handler]
    async fn handler_four(self, foo: String) -> Result<impl IntoResponse, impl IntoResponse> {
        dbg!(foo);
        Err::<(), _>(())
    }
}

#[async_trait]
impl<B> FromRequest<B> for A
where
    B: Send + 'static,
{
    type Rejection = ();

    async fn from_request(_req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        unimplemented!()
    }
}
