import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/JoinLobbyView.tsx", "r") as f:
    data = f.read()

# Add Guest login UI
imports = """import { Alert, Button, Paper, Stack, Text, Title, TextInput } from "@mantine/core";
import { useForm } from "@mantine/form";
import { notifications } from "@mantine/notifications";
import { useGuestLoginMutation } from "../../api/api";
import { _setToken } from "../auth/authSlice";"""
data = re.sub(r'import \{ Alert, Button, Paper, Stack, Text, Title \} from "@mantine/core";', imports, data)

guest_form_code = """
  const [guestLogin, { isLoading: isGuestLoggingIn }] = useGuestLoginMutation();

  const guestForm = useForm({
    initialValues: { displayName: "" },
    validate: {
      displayName: (v) => (v.trim().length > 0 ? null : "Display name is required"),
    },
  });

  const handleGuestJoin = async (values: { displayName: string }) => {
    if (!parsed) return;
    try {
      const res = await guestLogin(values).unwrap();
      dispatch(_setToken(res.token));
      // Auth is updated in store, handleJoin will be triggered or user can click Join again
      notifications.show({ title: "Joined as guest", message: "You can now connect to the lobby.", color: "blue" });
    } catch (e: any) {
      setError(e.message || "Failed to join as guest");
    }
  };

  const handleJoin = async () => {
    if (!parsed) return;
    setJoining(true);
"""
data = data.replace('const handleJoin = async () => {\n    if (!parsed) return;\n    setJoining(true);', guest_form_code.strip())

guest_ui = """
        {!token ? (
          <form onSubmit={guestForm.onSubmit(handleGuestJoin)}>
            <Stack>
              <Alert color="yellow">You are not logged in. Join as a guest by entering a display name.</Alert>
              <TextInput
                label="Display Name"
                placeholder="Guest Player"
                {...guestForm.getInputProps("displayName")}
              />
              <Button type="submit" loading={isGuestLoggingIn}>
                Continue as Guest
              </Button>
            </Stack>
          </form>
        ) : (
          <Button onClick={handleJoin} loading={joining}>
            Join Lobby
          </Button>
        )}
"""
old_ui = re.compile(r'\{\!token && \(\s*<Alert color="yellow">You must be logged in to join a lobby\.</Alert>\s*\)\}\s*<Button onClick=\{handleJoin\} loading=\{joining\} disabled=\{\!token\}>\s*Join Lobby\s*</Button>', re.DOTALL)
data = old_ui.sub(guest_ui.strip(), data)

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/JoinLobbyView.tsx", "w") as f:
    f.write(data)
