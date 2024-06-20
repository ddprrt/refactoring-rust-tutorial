use std::{collections::HashMap, net::SocketAddr};

use microservice_rust_workshop::{kv_store::stored_type::StoredType, router, SharedState};

type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let state = SharedState::<HashMap<String, StoredType>>::default();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let app = router(&state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
