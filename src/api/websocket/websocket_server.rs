use std::collections::HashMap;

use actix::prelude::*;
use uuid::Uuid;

use crate::error::AppError;

// https://actix.rs/docs/websockets/
// https://github.com/actix/examples/blob/master/websockets/chat/src/server.rs

#[derive(Debug)]
pub struct WebsocketServer {
    sessions: HashMap<usize, Recipient<WebsocketServerMessages>>,
    user_sessions: HashMap<Uuid, Vec<usize>>,
    // rooms: HashMap<String, Vec<usize>>,
}

impl WebsocketServer {
    pub fn broadcast_to_user(
        &self,
        user_id: Uuid,
        msg: WebsocketServerMessages,
    ) -> Result<(), AppError> {
        let WebsocketServer {
            sessions,
            user_sessions,
            ..
        } = self;
        let us = user_sessions.get(&user_id);
        if us.is_none() {
            return Ok(());
        }
        for s in us.unwrap() {
            if let Some(rec) = sessions.get(s) {
                rec.do_send(msg);
            }
        }
        Ok(())
    }
}

/// Make actor from `WebsocketServer`
impl Actor for WebsocketServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub user_id: Uuid,
    pub addr: Recipient<WebsocketServerMessages>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message, Clone, Debug)]
#[rtype(usize)]
pub struct WebsocketServerMessages {}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for WebsocketServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // notify all users in same room
        self.send_message("main", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.rooms.entry("main".to_owned()).or_default().insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("main", &format!("Total visitors {count}"), 0);

        // send id back
        id
    }
}
