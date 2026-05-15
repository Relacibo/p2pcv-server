use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "users")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub locale: String,
    pub verified_email: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::google_user::Entity")]
    GoogleUser,
    #[sea_orm(has_one = "super::lichess_user::Entity")]
    LichessUser,
}

impl ActiveModelBehavior for ActiveModel {}
