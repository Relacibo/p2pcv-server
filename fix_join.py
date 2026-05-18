import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/JoinLobbyView.tsx", "r") as f:
    data = f.read()

data = data.replace('import { _setToken } from "../auth/authSlice";', 'import { login } from "../auth/authSlice";')
data = data.replace('dispatch(_setToken(res.token));', 'dispatch(login({ token: res.token, user: res.user }));')

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/JoinLobbyView.tsx", "w") as f:
    f.write(data)
