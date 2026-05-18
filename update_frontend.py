import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/CreateLobbyView.tsx", "r") as f:
    data = f.read()

# Add allowGuests to form
data = data.replace('initialValues: { scriptUrl: "", useServerLobby: !!token },', 'initialValues: { scriptUrl: "", useServerLobby: !!token, allowGuests: true },')

# Add allowGuests checkbox
guest_checkbox = """
          {form.values.useServerLobby && (
            <Checkbox
              label="Allow unauthenticated players"
              description="Anyone with the link can join as a guest"
              {...form.getInputProps("allowGuests", { type: "checkbox" })}
            />
          )}
          <Button type="submit" loading={isCreating}>"""
data = data.replace('<Button type="submit" loading={isCreating}>', guest_checkbox)

# Update createLobby call
data = data.replace('dispatch(createLobby(normalized, !!token && useServerLobby));', 'dispatch(createLobby(normalized, !!token && useServerLobby, form.values.allowGuests));')

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/CreateLobbyView.tsx", "w") as f:
    f.write(data)
