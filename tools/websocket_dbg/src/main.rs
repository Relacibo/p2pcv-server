use std::{env, sync::Arc};

use dotenvy::dotenv;
use env_logger::Env;
use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use p2pcv_protobuf::{
    client_to_server::{self, msg::C2s, NewGame},
    server_to_client::{self, msg::S2c, NewGameEvent},
};
use prost::Message;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, client::IntoClientRequest, protocol::CloseFrame},
    MaybeTlsStream, WebSocketStream,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::from_filename("../../.env").ok();

    let websocket_uri = env::var("DEBUG_WEBSOCKET_URI").expect("DEBUG_WEBSOCKET_URI missing!");
    let user_auth_token = env::var("DEBUG_USER_TOKEN").expect("DEBUG_USER_TOKEN missing!");
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let mut request = websocket_uri.into_client_request().unwrap();
    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {user_auth_token}").parse().unwrap(),
    );
    let (ws_stream, _) = connect_async(request).await?;

    log::debug!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let splitted: Arc<(Mutex<_>, Mutex<_>)> = Arc::new((Mutex::new(write), Mutex::new(read)));
    let splitted2 = splitted.clone();

    let receiver_user_id = uuid::uuid!("12345678-dead-beef-b116-b0000000b135");
    let variant_id = uuid::uuid!("fa21a473-dead-beef-b116-b0000000b135");

    let handle = tokio::spawn(async move {
        let (write, read) = splitted2.as_ref();
        let mut read_lock = read.lock().await;
        while let Some(message) = read_lock.next().await {
            drop(read_lock);
            let message = match message {
                Ok(message) => message,
                Err(err) => {
                    log::error!("Something is wrong with the message!");
                    log::error!("{err}");
                    read_lock = read.lock().await;
                    continue;
                }
            };
            if let Err(err) = handle_server_message(write, message).await {
                log::error!("Error in message handler!");
                log::error!("{err}");
                read_lock = read.lock().await;
                continue;
            };
            read_lock = read.lock().await;
        }
    });

    let new_game_request = NewGame {
        receiver_user_id: receiver_user_id.as_bytes().to_vec(),
        variant_id: variant_id.as_bytes().to_vec(),
        variant_version: "1.0.0".to_owned(),
    };
    let request = C2s::NewGame(new_game_request);

    let (write, _) = splitted.as_ref();

    send_c2s(write, request).await?;

    ping_server(write).await?;

    handle.await?;

    // pin_mut!(stdin_to_ws, handle);
    // future::select(future1, handle).await;
    Ok(())
}

async fn send_to_server(
    write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
    msg: tungstenite::Message,
) -> anyhow::Result<()> {
    write.lock().await.send(msg).await?;
    Ok(())
}

async fn ping_server(
    write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
) -> anyhow::Result<()> {
    send_to_server(write, tungstenite::Message::Ping(b"Hi".to_vec())).await?;
    Ok(())
}

async fn send_c2s(
    write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
    message: C2s,
) -> anyhow::Result<()> {
    let mut buf = Vec::new();
    message.encode(&mut buf);
    send_to_server(write, tungstenite::Message::Binary(buf)).await?;
    Ok(())
}

async fn handle_server_message(
    write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
    msg: tungstenite::Message,
) -> anyhow::Result<()> {
    match msg {
        tungstenite::Message::Binary(msg) => {
            let server_to_client::Msg { id, s2c } = server_to_client::Msg::decode(msg.as_ref())?;
            let Some(s2c) = s2c else {
                return Err(anyhow::anyhow!("Server message is empty"));
            };
            handle_s2c(write, s2c).await?;
        }
        tungstenite::Message::Pong(pong) => {
            log::debug!("{}", String::from_utf8_lossy(&pong));
        }
        tungstenite::Message::Close(close_frame) => {
            log::info!("Session closed by server");
            if let Some(close_frame) = close_frame {
                let CloseFrame { code, reason } = close_frame;
                log::debug!("Code: {code}");
                log::debug!("Reason: {reason}");
            }
            return Err(anyhow::anyhow!("Closed by server!"));
        }
        _ => {
            log::error!("Unsupported Message type - Closing");
            return Err(anyhow::anyhow!("Message type not supported!"));
        }
    }
    Ok(())
}

async fn handle_s2c(
    write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
    msg: S2c,
) -> anyhow::Result<()> {
    match msg {
        S2c::NewGameEvent(e) => {
            log::debug!("{e:?}")
        }
        S2c::NewGameResponse(r) => {
            log::debug!("{r:?}")
        }
    }
    Ok(())
}
