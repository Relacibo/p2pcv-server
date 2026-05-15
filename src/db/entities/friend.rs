use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "friends")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user1_id: Uuid,
    pub user2_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::User1Id",
        to = "super::user::Column::Id"
    )]
    User1,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::User2Id",
        to = "super::user::Column::Id"
    )]
    User2,
}

impl ActiveModelBehavior for ActiveModel {}
