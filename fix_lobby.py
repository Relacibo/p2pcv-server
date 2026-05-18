import re
with open("src/api/lobby/mod.rs", "r") as f:
    data = f.read()

# Replace any repeated derive/serde combinations
pattern = r'(#\[derive\(Deserialize\)\]\n#\[serde\(rename_all = "camelCase"\)\]\n)+'
data = re.sub(pattern, '#[derive(Deserialize)]\n#[serde(rename_all = "camelCase")]\n', data)

with open("src/api/lobby/mod.rs", "w") as f:
    f.write(data)
