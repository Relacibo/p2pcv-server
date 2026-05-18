import re
with open("/home/reinhard/git/p2p-chessvariants/src/api/types/user/users.ts", "r") as f:
    data = f.read()

data = data.replace('userName: string;\n  displayName:string;', 'userName: string;\n  avatarHash?: string;\n  displayName:string;')

with open("/home/reinhard/git/p2p-chessvariants/src/api/types/user/users.ts", "w") as f:
    f.write(data)
