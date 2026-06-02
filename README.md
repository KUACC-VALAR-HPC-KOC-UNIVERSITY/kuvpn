# KUVPN v3.0.0

KUVPN is a VPN client for Koç University that automates the Microsoft Azure AD / MFA browser login to retrieve a DSID cookie, then hands off to OpenConnect to establish the VPN tunnel.

> **KUVPN** is the graphical app (system tray, GUI window).
> **kuvpn** is the command-line tool (run `kuvpn` in a terminal).
> Both connect to the same VPN — pick whichever fits your workflow.

---

## Documentation

<table>
<tr>
<td align="center" width="33%">

### [Install](https://github.com/ealtun21/kuvpn-actions#installation)
Get up and running on any platform

</td>
<td align="center" width="33%">

### [GUI Docs](https://github.com/ealtun21/kuvpn-actions/blob/main/docs/gui.md)
Graphical app — KUVPN

</td>
<td align="center" width="33%">

### [CLI Docs](https://github.com/ealtun21/kuvpn-actions/blob/main/docs/cli.md)
Command-line tool — kuvpn

</td>
</tr>
</table>

---

## Installation

**Linux:**

```bash
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash
```

**macOS:**

```bash
curl -sSfL https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash
```

The script will ask what to install (GUI, CLI, or both), set up your PATH, and check for OpenConnect.

**Windows** (run in PowerShell):

```powershell
irm https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.ps1 | iex
```

Or download and run [`KUVPN-Setup-windows-x86_64.exe`](https://github.com/ealtun21/kuvpn-actions/releases/latest) manually from the Releases page.

<details><summary>Non-interactive flags (for scripting / automation)</summary>

```bash
# Linux — install both
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --all

# Linux — GUI only
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --gui

# Linux — CLI only
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --cli

# macOS — install both
curl -sSfL https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --all
```

</details>

## Features

- **Two ways to connect** — graphical app for everyday desktop use, or a single static CLI binary for terminals, scripts, and SSH sessions. Same VPN, same saved session.
- **Hands-free Azure AD / MFA login** — automated browser walks through the entire login: email, password, Authenticator push, number matching, and OTP fallback if push isn't available.
- **Three login modes** — Full Auto (headless), Visual Auto (visible browser, still automated), and Manual (you drive the login). Sessions are saved so the next connect goes straight through.
- **Number-matching front and centre** — when Microsoft asks you to type a code into Authenticator, KUVPN brings the window forward and shows the code in big, readable text.
- **System tray with one-click actions** — connect, disconnect, show window, wipe session, or copy logs from the tray menu. The tray icon reflects current state (idle, connecting, connected, error).
- **Connection history** — past sessions with timestamps, durations, and reconnect counts, in both the GUI History tab and `kuvpn --history`.
- **Auto-reconnect that watches the tunnel** — detects when the VPN interface itself drops (not just the OpenConnect process) and retries up to 3 times. Stale saved sessions are wiped and re-authenticated automatically.
- **Conflict detection** — refuses to start if another full-tunnel VPN is already routing your traffic (e.g. a Tailscale exit node), so you don't end up half-connected to two networks.
- **Routing flexibility** — full tunnel for "everything through KU", or supply your own vpnc-script for split tunneling and custom DNS.
- **20 themes** — 10 color families (Crimson, Slate, Ocean, Forest, Rose, Violet, Ember, Frost, Sand, Pebble), each with light and dark variants, plus rounding and shadow controls.
- **Cross-platform** — Linux (x86_64, aarch64), macOS (Intel and Apple Silicon), and Windows (x86_64). OpenConnect and the Wintun driver are bundled in the Windows installer.

---

## Screenshots

### Graphical Interface (KUVPN)

| Main Window | History | Settings |
|:---:|:---:|:---:|
| <img src="docs/screenshots/connected.png" width="300"> | <img src="docs/screenshots/gui-history.png" width="300"> | <img src="docs/screenshots/settings.png" width="300"> |

| MFA Authentication | Email Automation | Live Logs |
|:---:|:---:|:---:|
| <img src="docs/screenshots/connecting-mfa.png" width="300"> | <img src="docs/screenshots/connecting-email.png" width="300"> | <img src="docs/screenshots/session-logger.png" width="300"> |

### Command Line Interface (kuvpn)

| Connected | Connecting | MFA Prompt |
|:---:|:---:|:---:|
| <img src="docs/screenshots/cli-connected.png" width="300"> | <img src="docs/screenshots/cli-connecting.png" width="300"> | <img src="docs/screenshots/cli-mfa.png" width="300"> |

| History | Session Management | Disconnected |
|:---:|:---:|:---:|
| <img src="docs/screenshots/cli-history.png" width="300"> | <img src="docs/screenshots/cli-clear-session.png" width="300"> | <img src="docs/screenshots/cli-disconnected.png" width="300"> |

### Themes

KUVPN features 10 unique color families, each with Light and Dark modes (20 themes total).

| | Dark | Light |
| --- | :---: | :---: |
| **Crimson (Default)** | <img src="docs/screenshots/gui-theme-dark.png" width="300"> | <img src="docs/screenshots/gui-theme-light.png" width="300"> |
| **Slate** | <img src="docs/screenshots/slate-theme-dark.png" width="300"> | <img src="docs/screenshots/slate-theme-light.png" width="300"> |
| **Ocean** | <img src="docs/screenshots/ocean-theme-dark.png" width="300"> | <img src="docs/screenshots/ocean-theme-light.png" width="300"> |
| **Forest** | <img src="docs/screenshots/forest-theme-dark.png" width="300"> | <img src="docs/screenshots/forest-theme-light.png" width="300"> |
| **Rose** | <img src="docs/screenshots/rose-theme-dark.png" width="300"> | <img src="docs/screenshots/rose-theme-light.png" width="300"> |
| **Violet** | <img src="docs/screenshots/violet-theme-dark.png" width="300"> | <img src="docs/screenshots/violet-theme-light.png" width="300"> |
| **Ember** | <img src="docs/screenshots/ember-theme-dark.png" width="300"> | <img src="docs/screenshots/ember-theme-light.png" width="300"> |
| **Frost** | <img src="docs/screenshots/frost-theme-dark.png" width="300"> | <img src="docs/screenshots/frost-theme-light.png" width="300"> |
| **Sand** | <img src="docs/screenshots/sand-theme-dark.png" width="300"> | <img src="docs/screenshots/sand-theme-light.png" width="300"> |
| **Pebble** | <img src="docs/screenshots/pebble-theme-dark.png" width="300"> | <img src="docs/screenshots/pebble-theme-light.png" width="300"> |

---

## License

MIT — see [LICENSE](LICENSE).

## Contributing

Issues and pull requests are welcome.
