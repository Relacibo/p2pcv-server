use std::{
    borrow::BorrowMut,
    collections::HashMap,
    sync::{
        atomic::{AtomicIsize, AtomicU16, Ordering},
        Arc,
    },
};

use actix_web::{
    web::{self, Data, ServiceConfig},
    HttpRequest, Responder,
};
use actix_ws::{CloseReason, Closed, Session};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use p2pcv_protobuf::{
    client_to_server::{
        self,
        msg::{self, C2s},
        new_game_event_response, Msg, NewGame, NewGameEventResponse,
    },
    server_to_client::{msg::S2c, NewGameResponse},
};
use prost::Message;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{api::auth::session::auth::Auth, error::AppError};
use std::fmt::Debug;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(ws).app_data(Data::new(Websockets::default()));
}

#[get("ws")]
async fn ws(
    ws_server: Data<Websockets>,
    auth: Auth,
    req: HttpRequest,
    body: web::Payload,
) -> actix_web::Result<impl Responder> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let ws_server = ws_server.into_inner();
    let user_id = auth.user_id;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let ws_session = WebsocketSession {
        id,
        session,
        user_id,
        last_pinged: Arc::new(AtomicIsize::new(now.timestamp() as isize)),
    };
    let session2 = ws_session.clone();
    ws_server.sessions.insert(id, session2);

    actix_web::rt::spawn(async move {
        let mut session = ws_session.session.clone();
        while let Some(Ok(msg)) = msg_stream.next().await {
            if handle_client_message(&ws_server, &ws_session, &mut session, msg)
                .await
                .is_err()
            {
                break;
            };
        }

        close_session(&ws_server, ws_session).await.ok();
    });

    Ok(response)
}

async fn close_session(
    ws_server: &Arc<Websockets>,
    session: WebsocketSession,
) -> Result<(), Closed> {
    ws_server.sessions.remove(&session.id);
    let WebsocketSession { session, .. } = session;
    session.close(None).await
}

async fn handle_client_message(
    ws_server: &Arc<Websockets>,
    ws_session: &WebsocketSession,
    session: &mut Session,
    msg: actix_ws::Message,
) -> Result<(), WebsocketError> {
    let WebsocketSession { id, user_id, .. } = ws_session;
    match msg {
        actix_ws::Message::Ping(bytes) => {
            ws_session.update_pinged();
            session.pong(&bytes).await?;
        }
        actix_ws::Message::Binary(msg) => {
            ws_session.update_pinged();
            let Msg {
                c2s: Some(request),
                id,
            } = client_to_server::Msg::decode(msg)?
            else {
                return Err(WebsocketError::ClientEmptyRequest);
            };
            handle_c2s(ws_server, ws_session, session, request).await?;
        }
        actix_ws::Message::Close(reason) => {
            log::info!("Session {id}: Closed by client (User Id: {user_id})");
            if let Some(CloseReason { code, description }) = reason {
                log::info!("Code: {code:?}");
                if let Some(description) = description {
                    log::info!("Description: {description}");
                }
            }
            return Err(WebsocketError::ClientDisconnect);
        }
        _ => {
            log::error!("Session {id}: Unsupported Message type - Closing (User Id: {user_id})");
            return Err(WebsocketError::UnsupportedMessageType);
        }
    };
    Ok(())
}

async fn handle_c2s(
    ws_server: &Arc<Websockets>,
    ws_session: &WebsocketSession,
    session: &mut Session,
    request: C2s,
) -> Result<(), WebsocketError> {
    match request {
        C2s::NewGame(new_game) => handle_new_game(ws_server, ws_session, session, new_game).await?,
        C2s::NewGameEventResponse(response) => {
            handle_new_game_event_response(ws_server, ws_session, session, response).await?
        }
    }
    Ok(())
}

async fn send_response(session: &mut Session, response: S2c) -> Result<(), WebsocketError> {
    let mut buf = Vec::new();
    response.encode(&mut buf);
    session.binary(buf).await?;
    Ok(())
}

async fn handle_new_game(
    ws_server: &Arc<Websockets>,
    ws_session: &WebsocketSession,
    session: &mut Session,
    new_game: NewGame,
) -> Result<(), WebsocketError> {
    let response = NewGameResponse {
        error: None,
        answer: Some(new_game_event_response::Answer::Accept as i32),
        // Debug
        peer_id: Some(Uuid::new_v4().as_bytes().to_vec()),
    };
    let res = S2c::NewGameResponse(response);
    send_response(session, res).await?;
    Ok(())
}

async fn handle_new_game_event_response(
    ws_server: &Arc<Websockets>,
    ws_session: &WebsocketSession,
    session: &mut Session,
    response: NewGameEventResponse,
) -> Result<(), WebsocketError> {
    let WebsocketSession {
        id,
        session,
        user_id,
        last_pinged,
    } = ws_session;
    let NewGameEventResponse { answer, peer_id } = response;
    let answer = new_game_event_response::Answer::try_from(answer)?;
    match answer {
        new_game_event_response::Answer::Accept => log::debug!("Accepted!"),
        new_game_event_response::Answer::Decline => log::debug!("Declined!"),
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct Websockets {
    pub sessions: dashmap::DashMap<Uuid, WebsocketSession>,
}

#[derive(Clone)]
pub struct WebsocketSession {
    pub id: Uuid,
    pub session: Session,
    pub user_id: Uuid,
    pub last_pinged: Arc<AtomicIsize>,
}

impl WebsocketSession {
    fn update_pinged(&self) {
        let now = Utc::now().timestamp() as isize;
        self.last_pinged.store(now, Ordering::Relaxed);
    }
}

impl Debug for WebsocketSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebsocketSession")
            .field("session", &"?")
            .field("last_pinged", &self.last_pinged)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum WebsocketError {
    #[error("client-disconnect")]
    ClientDisconnect,
    #[error("client-empty-request")]
    ClientEmptyRequest,
    #[error("unsupported-message-type")]
    UnsupportedMessageType,
    #[error("prost-decode")]
    ProstDecode(#[from] prost::DecodeError),
    #[error("prost-encode")]
    ProstEncode(#[from] prost::EncodeError),
    #[error("prost-unknown-enum-value")]
    ProstUnknownEnumValue(#[from] prost::UnknownEnumValue),
    #[error("actix_ws-closed")]
    WebsocketClosed(#[from] actix_ws::Closed),
}
