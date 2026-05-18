use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuthProviders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuthProviders::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthProviders::UserId).uuid().not_null())
                    .col(ColumnDef::new(AuthProviders::Provider).string().not_null())
                    .col(
                        ColumnDef::new(AuthProviders::ProviderUserId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AuthProviders::DisplayName).string().null())
                    .col(
                        ColumnDef::new(AuthProviders::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AuthProviders::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_auth_providers_user_id")
                            .from(AuthProviders::Table, AuthProviders::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE auth_providers ALTER COLUMN id SET DEFAULT gen_random_uuid()",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AuthProviders::Table)
                    .name("auth_providers_provider_user_id_unique")
                    .col(AuthProviders::Provider)
                    .col(AuthProviders::ProviderUserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(AuthProviders::Table)
                    .name("auth_providers_user_id_provider_unique")
                    .col(AuthProviders::UserId)
                    .col(AuthProviders::Provider)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TRIGGER update_auth_providers_updated_at BEFORE UPDATE ON auth_providers
                 FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO auth_providers (id, user_id, provider, provider_user_id, created_at, updated_at)
                 SELECT gen_random_uuid(), user_id, 'google', id, created_at, created_at FROM google_users",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO auth_providers (id, user_id, provider, provider_user_id, display_name, created_at, updated_at)
                 SELECT gen_random_uuid(), user_id, 'lichess', id, username, created_at, updated_at FROM lichess_users",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(GoogleUsers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LichessUsers::Table).to_owned())
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS update_auth_providers_updated_at ON auth_providers",
            )
            .await?;
        manager
            .drop_table(Table::drop().table(AuthProviders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuthProviders {
    Table,
    Id,
    UserId,
    Provider,
    ProviderUserId,
    DisplayName,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum GoogleUsers {
    Table,
}

#[derive(DeriveIden)]
enum LichessUsers {
    Table,
}
