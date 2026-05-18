use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000011_create_lobbies"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Lobbies::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Lobbies::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Lobbies::HostUserId).uuid().not_null())
                    .col(ColumnDef::new(Lobbies::HostPeerSessionId).string().null())
                    .col(ColumnDef::new(Lobbies::ScriptUrl).string().not_null())
                    .col(
                        ColumnDef::new(Lobbies::AllowGuests)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // Stored as text: "waiting" | "in-game" | "finished"
                    .col(
                        ColumnDef::new(Lobbies::Status)
                            .string()
                            .not_null()
                            .default("waiting"),
                    )
                    .col(
                        ColumnDef::new(Lobbies::PlayerCount)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Lobbies::MinPlayers).integer().null())
                    .col(ColumnDef::new(Lobbies::MaxPlayers).integer().null())
                    .col(
                        ColumnDef::new(Lobbies::LastHeartbeat)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Lobbies::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-lobby-host")
                            .from(Lobbies::Table, Lobbies::HostUserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-lobbies-last-heartbeat")
                    .table(Lobbies::Table)
                    .col(Lobbies::LastHeartbeat)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Lobbies::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Lobbies {
    Table,
    Id,
    HostUserId,
    HostPeerSessionId,
    ScriptUrl,
    AllowGuests,
    Status,
    PlayerCount,
    MinPlayers,
    MaxPlayers,
    LastHeartbeat,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
