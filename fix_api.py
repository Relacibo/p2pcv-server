import re
with open("/home/reinhard/git/p2p-chessvariants/src/api/api.ts", "r") as f:
    data = f.read()

endpoint = """
    guestLogin: builder.mutation<LoginResponse & { result: "success" }, { displayName: string }>({
      query: (body) => ({
        url: "auth/guest",
        method: "post",
        body,
      }),
    }),
"""
data = data.replace('serverLogout: builder.mutation<void, void>({', endpoint.strip() + '\n    serverLogout: builder.mutation<void, void>({')

with open("/home/reinhard/git/p2p-chessvariants/src/api/api.ts", "w") as f:
    f.write(data)
