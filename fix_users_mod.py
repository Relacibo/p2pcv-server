import re
with open("/home/reinhard/git/p2pcv-server/src/api/users/mod.rs", "r") as f:
    data = f.read()

data = data.replace('routing::get,', 'routing::{get, put},')
data = data.replace('response::IntoResponse,', 'response::IntoResponse,')

if 'IntoResponse' not in data:
    data = data.replace('http::StatusCode,', 'http::StatusCode,\n    response::IntoResponse,')

with open("/home/reinhard/git/p2pcv-server/src/api/users/mod.rs", "w") as f:
    f.write(data)
