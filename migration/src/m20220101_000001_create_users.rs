use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_users"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::UserName)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::DisplayName).string().not_null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Locale)
                            .string()
                            .not_null()
                            .default("en"),
                    )
                    .col(
                        ColumnDef::new(Users::VerifiedEmail)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
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
                "CREATE OR REPLACE FUNCTION update_updated_at_column()
                 RETURNS TRIGGER AS $$
                 BEGIN NEW.updated_at = CURRENT_TIMESTAMP; RETURN NEW; END;
                 $$ language 'plpgsql'",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
                 FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Friends::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Friends::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Friends::User1Id).uuid().not_null())
                    .col(ColumnDef::new(Friends::User2Id).uuid().not_null())
                    .col(
                        ColumnDef::new(Friends::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friends::Table, Friends::User1Id)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Friends::Table, Friends::User2Id)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE friends ADD CONSTRAINT friends_unique UNIQUE (user1_id, user2_id)",
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE friends ADD CONSTRAINT friends_check CHECK (user2_id > user1_id)",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Friends::Table).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER IF EXISTS update_users_updated_at ON users")
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    UserName,
    DisplayName,
    Email,
    Locale,
    VerifiedEmail,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Friends {
    Table,
    Id,
    User1Id,
    User2Id,
    CreatedAt,
}
