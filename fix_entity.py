import re
with open("src/db/entities/users.rs", "r") as f:
    data = f.read()

data = data.replace('pub verified_email: bool,', 'pub verified_email: bool,\n    pub use_gravatar: bool,')

with open("src/db/entities/users.rs", "w") as f:
    f.write(data)
