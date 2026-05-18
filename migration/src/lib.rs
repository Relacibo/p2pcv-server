pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_users;
mod m20220101_000002_create_google;
mod m20220101_000003_create_friend_requests;
mod m20220101_000004_create_friends_func;
mod m20220101_000005_create_lichess;
mod m20220101_000006_create_refresh_tokens;
mod m20220101_000007_create_auth_providers;
mod m20220101_000008_drop_lichess_access_tokens;
mod m20220101_000009_add_use_gravatar_to_users;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_users::Migration),
            Box::new(m20220101_000002_create_google::Migration),
            Box::new(m20220101_000003_create_friend_requests::Migration),
            Box::new(m20220101_000004_create_friends_func::Migration),
            Box::new(m20220101_000005_create_lichess::Migration),
            Box::new(m20220101_000006_create_refresh_tokens::Migration),
            Box::new(m20220101_000007_create_auth_providers::Migration),
            Box::new(m20220101_000008_drop_lichess_access_tokens::Migration),
            Box::new(m20220101_000009_add_use_gravatar_to_users::Migration),
        ]
    }
}
