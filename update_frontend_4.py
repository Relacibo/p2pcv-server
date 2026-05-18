import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/ActiveLobbyView.tsx", "r") as f:
    data = f.read()

# Remove Alert
data = data.replace('import {\n  Alert,\n  Button,', 'import {\n  Button,')
data = data.replace('import {\n  IconBrandGithub,\n  IconCheck,\n  IconCopy,\n  IconUser,\n} from "@tabler/icons-react";', 'import {\n  IconBrandGithub,\n  IconCopy,\n  IconUser,\n  IconQrcode,\n} from "@tabler/icons-react";')
data = data.replace('import { QRCodeSVG } from "qrcode.react";\n', '') # prevent double
data = data.replace('import {\n  leaveLobby,', 'import { QRCodeSVG } from "qrcode.react";\nimport {\n  leaveLobby,')

alert_block = r"""        <Alert
          icon={<IconCheck size="1rem" />}
          color="green"
          title="Lobby created!"
        >
          Share the invite link below with players you want to invite.
        </Alert>

"""
data = re.sub(alert_block, "", data)

qr_code_jsx = """
        <Box>
          <Text size="sm" fw={500} mb="xs">
            Invite link
          </Text>
          <Group align="flex-start" wrap="nowrap">
            <QRCodeSVG value={inviteUrl} size={128} />
            <Stack style={{ flex: 1 }} gap="xs">
              <Code style={{ overflow: "hidden", textOverflow: "ellipsis" }}>
                {inviteUrl}
              </Code>
              <CopyButton value={inviteUrl}>
                {({ copied, copy }) => (
                  <Button
                    size="compact-sm"
                    variant="light"
                    color={copied ? "teal" : "blue"}
                    leftSection={
                      <IconCopy size="0.9rem" />
                    }
                    onClick={copy}
                  >
                    {copied ? "Copied" : "Copy"}
                  </Button>
                )}
              </CopyButton>
            </Stack>
          </Group>
        </Box>
"""

# Replace the existing Invite link box
old_invite_box = re.compile(r'<Box>\s*<Text size="sm" fw=\{500\} mb="xs">\s*Invite link\s*</Text>\s*<Group gap="xs" wrap="nowrap">.*?onClick=\{copy\}\s*>\s*\{copied \? "Copied" : "Copy"\}\s*</Button>\s*\)\}\s*</CopyButton>\s*</Group>\s*</Box>', re.DOTALL)

data = old_invite_box.sub(qr_code_jsx.strip(), data)

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/ActiveLobbyView.tsx", "w") as f:
    f.write(data)
