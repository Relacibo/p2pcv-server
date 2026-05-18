use std::sync::Arc;

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{AppState, api::auth::session::auth::Auth, error::AppError};

pub async fn sse_handler(
    State(state): State<Arc<AppState>>,
    auth: Auth,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, AppError> {
    let user_id = auth.user_id;
    let (tx, rx) = mpsc::channel(64);
    state.sse_registry.register(user_id, tx);
    let stream = ReceiverStream::new(rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
