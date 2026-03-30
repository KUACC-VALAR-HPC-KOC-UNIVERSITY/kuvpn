# kuvpn — CLI Documentation

[← Back to README](../README.md) · [GUI Documentation →](gui.md)

`kuvpn` is the command-line frontend for KUVPN. It automates the Microsoft Azure AD / MFA login flow in a headless browser, retrieves the DSID cookie, and launches OpenConnect to establish the VPN tunnel.

---

## Installation

### Linux

**Recommended:**

```bash
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash
```

The script will ask what to install — choose CLI. It downloads the right static binary for your platform, places it at `~/.local/bin/kuvpn`, adds it to your PATH if needed, and optionally installs OpenConnect.

<details><summary>Non-interactive (for scripting / automation)</summary>

```bash
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --cli
```

</details>

<details><summary>Manual install</summary>

Download **`kuvpn-linux-x86_64`** (or `aarch64`) from the [Releases](https://github.com/ealtun21/kuvpn-actions/releases/latest) page, make it executable, and move it onto your PATH:

```bash
# x86_64
chmod +x kuvpn-linux-x86_64 && mv kuvpn-linux-x86_64 ~/.local/bin/kuvpn

# aarch64
chmod +x kuvpn-linux-aarch64 && mv kuvpn-linux-aarch64 ~/.local/bin/kuvpn
```

</details>

### macOS

**Recommended:**

```bash
curl -sSfL https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash
```

The script will ask what to install — choose CLI. It downloads the right static binary for your platform, places it at `~/.local/bin/kuvpn`, adds it to your PATH if needed, and optionally installs OpenConnect.

<details><summary>Non-interactive (for scripting / automation)</summary>

```bash
curl -sSfL https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --cli
```

</details>

<details><summary>Manual install</summary>

Download **`kuvpn-macos-x86_64`** (Intel) or **`kuvpn-macos-aarch64`** (Apple Silicon) from the [Releases](https://github.com/ealtun21/kuvpn-actions/releases/latest) page, make it executable, and move it onto your PATH:

```bash
# Intel
chmod +x kuvpn-macos-x86_64 && mv kuvpn-macos-x86_64 ~/.local/bin/kuvpn

# Apple Silicon
chmod +x kuvpn-macos-aarch64 && mv kuvpn-macos-aarch64 ~/.local/bin/kuvpn
```

</details>

### Windows

**Recommended** — one-line terminal installer (run in PowerShell):

```powershell
irm https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.ps1 | iex
```

The script downloads and runs the latest installer silently. The CLI (`kuvpn`) is bundled and added to your PATH — no extra setup required.

<details><summary>Manual install</summary>

Download and run **`KUVPN-Setup-windows-x86_64.exe`** from the [Releases](https://github.com/ealtun21/kuvpn-actions/releases/latest) page. The installer bundles the CLI and adds it to your PATH automatically.

</details>

---

## Basic Usage

```bash
kuvpn
```

That's it for most users. `kuvpn` will open a headless browser, complete the Azure AD / MFA login automatically, and start OpenConnect. You'll see a progress spinner and live log output while it connects.

Press **Ctrl+C** to disconnect.

---

## Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--mode` | `-m` | `full-auto` | Login mode: `full-auto`, `visual`, or `manual` — see [Login Modes](#login-modes) |
| `--url` | | `https://vpn.ku.edu.tr` | VPN portal URL |
| `--domain` | | `vpn.ku.edu.tr` | Domain used for DSID cookie matching |
| `--email` | `-e` | *(none)* | Pre-fill your university email to speed up login |
| `--log` | `-l` | `error` | Log level: `off`, `error`, `warn`, `info`, `debug`, `trace` |
| `--dsid` | `-d` | `false` | Print the DSID cookie and exit without starting OpenConnect |
| `--history` | | `false` | Print connection history and exit |
| `--clean` | `-c` | `false` | Delete saved session data and exit |
| `--run-command` | | *(auto-detected)* | Override the privilege escalation tool (`sudo`, `pkexec`, or a custom script) |
| `--openconnect-path` | | `openconnect` | Path or command name for the OpenConnect binary |
| `--interface-name` | | `kuvpn0` | Name for the TUN interface created by OpenConnect |
| `--tunnel-mode` | | `full` | Tunnel mode: `full` (all traffic via VPN) or `manual` (custom vpnc-script) |
| `--vpnc-script` | | *(none)* | Path to a custom vpnc-script. Required when `--tunnel-mode manual` is set. |

---

## Connection History

```bash
kuvpn --history
```

Displays a timestamped list of past connection events (Connected, Reconnected, Disconnected, Cancelled, Error) with session durations. Reconnected entries show the duration of the previous session segment as `(prev: Xm Ys)`.

---

## Login Modes

### Full Auto (default)
The browser runs headlessly. KUVPN detects each page of the login flow and fills in fields automatically. Works for most users who have their session cached.

### Visual Auto (`--mode visual`)
Opens a visible browser window but still attempts to automate the login. Useful for debugging or when a CAPTCHA appears.

### Manual (`--mode manual`)
Opens a visible browser and does nothing — you complete the login yourself. KUVPN waits until it detects the DSID cookie, then takes over and starts OpenConnect.

```bash
kuvpn --mode manual
```

After you log in once this way, the session is saved and future runs can use Full Auto again.

---

## Troubleshooting

### Cookie rejected by server

```
Unexpected 302 result from server
Cookie was rejected by server; exiting.
```

Your saved session has expired. Clear it and reconnect:

```bash
kuvpn --clean
kuvpn
```

### Stuck at startup / downloading Chrome

`kuvpn` uses a headless Chromium browser. If it seems to hang on the first run it may be downloading Chromium. Check with:

```bash
kuvpn -l debug
```

If the download fails repeatedly, install Chrome or Chromium via your package manager — KUVPN will detect and use the system installation.

### Can't log in automatically

Try Manual mode so you can see what's happening:

```bash
kuvpn --mode manual
```

Once you complete the login manually, the session is saved for future runs.

### OpenConnect not found

Make sure OpenConnect is installed and on your PATH. The installer can do this for you, or install manually:

```bash
# Debian / Ubuntu
sudo apt install openconnect

# Fedora / RHEL
sudo dnf install openconnect

# Arch
sudo pacman -S openconnect

# macOS
brew install openconnect
```

If it's installed in a non-standard location, point `kuvpn` at it:

```bash
kuvpn --openconnect-path /usr/local/sbin/openconnect
```
