use crate::{
    api::auth::auth::Auth,
    app_error::AppError,
    app_result::EndpointResult,
    db::{
        db_conn::DbPool,
        user::{PublicUser, User},
    },
};
use actix_web::web::{self, Data, Json, Path, ServiceConfig};
use uuid::Uuid;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(list_users)
            .service(delete_user)
            .service(get_user),
    );
}

#[get("")]
pub async fn list_users(pool: Data<DbPool>) -> Result<Json<Vec<PublicUser>>, AppError> {
    let mut db = pool.get().await?;
    let res = User::list(&mut db).await?;
    Ok(Json(res))
}

#[delete("/{uuid}")]
pub async fn delete_user(
    pool: Data<DbPool>,
    user_id: Path<Uuid>,
    auth: Auth,
) -> EndpointResult<()> {
    auth.should_be_user(*user_id)?;
    let mut db = pool.get().await?;
    User::delete(&mut db, *user_id).await?;
    Ok(Json(()))
}

#[get("/{uuid}")]
pub async fn get_user(
    pool: Data<DbPool>,
    auth: Auth,
    user_id: Path<Uuid>,
) -> Result<Json<User>, AppError> {
    auth.should_be_user(*user_id)?;
    let mut db = pool.get().await?;
    let res = User::get(&mut db, *user_id).await?;
    Ok(Json(res))
}
