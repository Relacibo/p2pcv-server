use axum::response::sse::Event;
use dashmap::DashMap;
use serde::Serialize;
use tokio::sync::mpsc;
use uuid::Uuid;

pub type SseSender = mpsc::Sender<Result<Event, std::convert::Infallible>>;

#[derive(Default)]
pub struct SseRegistry(DashMap<Uuid, SseSender>);

impl SseRegistry {
    pub fn register(&self, user_id: Uuid, tx: SseSender) {
        self.0.insert(user_id, tx);
    }

    pub fn unregister(&self, user_id: &Uuid) {
        self.0.remove(user_id);
    }

    pub async fn send_to<T: Serialize>(&self, user_id: &Uuid, event_name: &str, data: &T) -> bool {
        let Some(tx) = self.0.get(user_id) else {
            return false;
        };
        let json = serde_json::to_string(data).unwrap_or_default();
        let event = Event::default().event(event_name).data(json);
        let ok = tx.send(Ok(event)).await.is_ok();
        drop(tx);
        if !ok {
            self.0.remove(user_id);
        }
        ok
    }

    pub async fn broadcast_to<T: Serialize>(
        &self,
        user_ids: &[Uuid],
        event_name: &str,
        data: &T,
    ) {
        let json = serde_json::to_string(data).unwrap_or_default();
        for user_id in user_ids {
            if let Some(tx) = self.0.get(user_id) {
                let event = Event::default()
                    .event(event_name)
                    .data(json.clone());
                let ok = tx.send(Ok(event)).await.is_ok();
                drop(tx);
                if !ok {
                    self.0.remove(user_id);
                }
            }
        }
    }
}
