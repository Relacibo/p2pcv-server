use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "friend_requests")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub message: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SenderId",
        to = "super::user::Column::Id"
    )]
    Sender,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::ReceiverId",
        to = "super::user::Column::Id"
    )]
    Receiver,
}

impl ActiveModelBehavior for ActiveModel {}
