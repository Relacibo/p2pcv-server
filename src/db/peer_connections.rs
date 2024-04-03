use diesel::{QueryDsl, QueryResult, Queryable, QueryableByName, Selectable, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use super::{
    model::{peer_connections as db_peer_connections, users as db_users},
    users::User,
};

#[derive(Serialize, Queryable, QueryableByName, PartialEq, Debug, Clone, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = db_peer_connections)]
pub struct PeerConnection {
    id: Uuid,
}

impl User {
    pub async fn list_peer_ids(&self, conn: &mut AsyncPgConnection) -> QueryResult<Vec<Uuid>> {
        User::list_peer_ids_by_user_id(conn, self.id).await
    }
    pub async fn list_peer_ids_by_user_id(
        conn: &mut AsyncPgConnection,
        query_user_id: Uuid,
    ) -> QueryResult<Vec<Uuid>> {
        use db_peer_connections::dsl::*;
        peer_connections.find(query_user_id).select(id).get_results(conn).await
    }
}
