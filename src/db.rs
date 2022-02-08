use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State};
use std::ops::Deref;
use rocket::outcome::{try_outcome, Outcome::{*}};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_pool(db_url: String) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::new(manager).expect("db pool failure")
}

pub struct DbConnection(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for DbConnection {
    type Error = ();
    async fn from_request(request: &'a Request<'_>) -> request::Outcome<DbConnection, ()> {
        let pool = try_outcome!(request.guard::<&State<DbPool>>().await);
        match pool.get() {
            Ok(conn) => Success(DbConnection(conn)),
            Err(_) => Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConnection {
    type Target = PgConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
