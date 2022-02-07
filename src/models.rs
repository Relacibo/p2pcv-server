use crate::schema::users;
use uuid::Uuid;

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}

#[derive(AsChangeset, Identifiable)]
#[table_name = "users"]
#[primary_key(uuid)]
pub struct EditUser<'a> {
    pub uuid: Uuid,
    pub name: Option<&'a str>,
    pub email: Option<&'a str>,
}
