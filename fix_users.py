import re
with open("src/db/users.rs", "r") as f:
    data = f.read()

# Add avatar_hash
data = data.replace('pub user_name: String,\n    pub created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,', 'pub user_name: String,\n    pub avatar_hash: Option<String>,\n    pub created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,')

# Add fields to FriendEntryRow
friend_entry_old = """struct FriendEntryRow {
    id: Uuid,
    user_name: String,
    created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    friends_created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
}"""
friend_entry_new = """struct FriendEntryRow {
    id: Uuid,
    user_name: String,
    use_gravatar: bool,
    email: String,
    created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
    friends_created_at: sea_orm::entity::prelude::DateTimeWithTimeZone,
}"""
data = data.replace(friend_entry_old, friend_entry_new)

# Update From<user::Model>
from_old = """    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            user_name: value.user_name,
            created_at: value.created_at,
        }
    }"""
from_new = """    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            user_name: value.user_name,
            avatar_hash: if value.use_gravatar { Some(format!("{:x}", md5::compute(value.email.trim().to_lowercase()))) } else { None },
            created_at: value.created_at,
        }
    }"""
data = data.replace(from_old, from_new)

# Update list_friends SQL
sql_old = 'SELECT users.id, user_name, users.created_at, tmp.created_at_ret AS friends_created_at'
sql_new = 'SELECT users.id, user_name, users.use_gravatar, users.email, users.created_at, tmp.created_at_ret AS friends_created_at'
data = data.replace(sql_old, sql_new)

# Update list_friends mapping
mapping_old = """                friend: PublicUser {
                    id: row.id,
                    user_name: row.user_name,
                    created_at: row.created_at,
                },"""
mapping_new = """                friend: PublicUser {
                    id: row.id,
                    user_name: row.user_name,
                    avatar_hash: if row.use_gravatar { Some(format!("{:x}", md5::compute(row.email.trim().to_lowercase()))) } else { None },
                    created_at: row.created_at,
                },"""
data = data.replace(mapping_old, mapping_new)

# Add use_gravatar default to NewUserWithId
data = data.replace('verified_email: Set(self.verified_email),', 'verified_email: Set(self.verified_email),\n            use_gravatar: Set(false),')


with open("src/db/users.rs", "w") as f:
    f.write(data)
