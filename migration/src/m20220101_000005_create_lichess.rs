use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000005_create_lichess"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LichessUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LichessUsers::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LichessUsers::Username).string().not_null())
                    .col(
                        ColumnDef::new(LichessUsers::UserId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(LichessUsers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(LichessUsers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(LichessUsers::Table, LichessUsers::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(LichessAccessTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LichessAccessTokens::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LichessAccessTokens::AccessToken)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LichessAccessTokens::Expires)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LichessAccessTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER update_lichess_users_updated_at BEFORE UPDATE ON lichess_users
                 FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS update_lichess_users_updated_at ON lichess_users",
            )
            .await?;
        manager
            .drop_table(Table::drop().table(LichessAccessTokens::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LichessUsers::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum LichessUsers {
    Table,
    Id,
    Username,
    UserId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum LichessAccessTokens {
    Table,
    Id,
    AccessToken,
    Expires,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
