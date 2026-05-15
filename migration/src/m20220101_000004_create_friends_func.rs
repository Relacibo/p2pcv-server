use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000004_create_friends_func"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE FUNCTION get_friend_entries(user_id UUID)
                 RETURNS TABLE (friend_user_id_ret UUID, created_at_ret TIMESTAMPTZ) AS $$
                 BEGIN
                   RETURN QUERY
                     SELECT friend_user_id AS friend_user_id_ret, created_at as created_at_ret
                     FROM (
                       SELECT user2_id AS friend_user_id, created_at FROM friends WHERE user1_id = user_id
                       UNION
                       SELECT user1_id AS friend_user_id, created_at FROM friends WHERE user2_id = user_id
                     ) AS tmp;
                 END;
                 $$ LANGUAGE plpgsql;",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP FUNCTION IF EXISTS get_friend_entries(UUID)")
            .await?;
        Ok(())
    }
}
