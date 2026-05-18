import re
with open("/home/reinhard/git/p2p-chessvariants/src/api/api.ts", "r") as f:
    data = f.read()

data = data.replace('export const {\n  useUpdateUserMutation,', 'export const {')

with open("/home/reinhard/git/p2p-chessvariants/src/api/api.ts", "w") as f:
    f.write(data)
