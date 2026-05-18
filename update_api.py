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
data = data.replace('logout: builder.mutation<void, void>({', endpoint.strip() + '\n    logout: builder.mutation<void, void>({')

hook = "useGuestLoginMutation,"
data = data.replace('useSignInMutation,', 'useSignInMutation,\n  useGuestLoginMutation,')

with open("/home/reinhard/git/p2p-chessvariants/src/api/api.ts", "w") as f:
    f.write(data)
