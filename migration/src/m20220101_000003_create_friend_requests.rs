use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000003_create_friend_requests"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FriendRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FriendRequests::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FriendRequests::SenderId).uuid().not_null())
                    .col(ColumnDef::new(FriendRequests::ReceiverId).uuid().not_null())
                    .col(ColumnDef::new(FriendRequests::Message).string().null())
                    .col(
                        ColumnDef::new(FriendRequests::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(FriendRequests::Table, FriendRequests::SenderId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(FriendRequests::Table, FriendRequests::ReceiverId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE friend_requests ADD CONSTRAINT friend_requests_unique UNIQUE (sender_id, receiver_id)",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE friend_requests ADD CONSTRAINT friend_requests_check CHECK (sender_id != receiver_id)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FriendRequests::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum FriendRequests {
    Table,
    Id,
    SenderId,
    ReceiverId,
    Message,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
