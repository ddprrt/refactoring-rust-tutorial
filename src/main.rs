use std::{collections::HashMap, net::SocketAddr};

use microservice_rust_workshop::{router, types::StoredType, SharedState};

type BoxError = Box<dyn std::error::Error>;
type TestStorage = HashMap<String, StoredType>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let state = SharedState::<TestStorage>::default();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let app = router(&state);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
