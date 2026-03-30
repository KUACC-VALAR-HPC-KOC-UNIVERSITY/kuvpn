# KUVPN — GUI Documentation

[← Back to README](../README.md) · [CLI Documentation →](cli.md)

**KUVPN** is the graphical frontend. It lives in your system tray and automatically brings itself to focus when it needs your input.

---

## Installation

### Linux

**Recommended:**

```bash
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash
```

The script will ask what to install — choose GUI. It also checks for OpenConnect and offers to install it.

<details><summary>Non-interactive (for scripting / automation)</summary>

```bash
wget -qO- https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --gui
```

</details>

<details><summary>Manual install</summary>

Download **`KUVPN-linux-x86_64.AppImage`** (or `aarch64`) from the [Releases](https://github.com/ealtun21/kuvpn-actions/releases/latest) page, make it executable, and run it:

```bash
chmod +x KUVPN-linux-x86_64.AppImage && ./KUVPN-linux-x86_64.AppImage
```

</details>

### macOS

**Recommended:**

```bash
curl -sSfL https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash
```

The script will ask what to install — choose GUI. It mounts the DMG, copies to Applications, and removes the quarantine flag automatically.

<details><summary>Non-interactive (for scripting / automation)</summary>

```bash
curl -sSfL https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.sh | bash -s -- --gui
```

</details>

<details><summary>Manual install</summary>

Download **`KUVPN-macOS-x86_64.dmg`** (Intel) or **`KUVPN-macOS-aarch64.dmg`** (Apple Silicon) from the [Releases](https://github.com/ealtun21/kuvpn-actions/releases/latest) page.

Open the DMG and drag **KUVPN.app** to your Applications folder, then run:

```bash
sudo xattr -r -d com.apple.quarantine /Applications/KUVPN.app
```

This removes the macOS quarantine flag that would otherwise block the app from opening (since it is not notarized through the App Store).

</details>

### Windows

**Recommended** — one-line terminal installer (run in PowerShell):

```powershell
irm https://raw.githubusercontent.com/ealtun21/kuvpn-actions/main/install.ps1 | iex
```

The script downloads and runs the latest installer silently. OpenConnect and Wintun are bundled — no extra setup required.

<details><summary>Manual install</summary>

Download and run **`KUVPN-Setup-windows-x86_64.exe`** from the [Releases](https://github.com/ealtun21/kuvpn-actions/releases/latest) page. The installer bundles OpenConnect and Wintun — no extra setup required.

</details>

---

## First Launch

When KUVPN opens you will see four tabs: **Connection**, **History**, **Console**, and **Settings**.

You can connect right away — just switch to the **Connection** tab and click **Join Network**. On the first run KUVPN will ask for your university email during login and save it automatically for future connections.

If you'd like to review or change settings (login mode, theme, advanced options) you can do so in the **Settings** tab at any time. For options beyond the basics (OpenConnect path, escalation tool, tunnel mode, etc.) switch the **Basic / Advanced** toggle in the top-right corner of the Settings tab.

---

## Connecting

1. Switch to the **Connection** tab.
2. Click **Join Network**.
3. The status changes to *Connecting* and the Console tab will show live log output.
4. If MFA is required, KUVPN brings the window to the front so you can respond.
5. Once connected, the status shows *Connected* and the tray icon turns green.

To disconnect, click **Disconnect** or use the tray menu.

---

## Login Modes

The segmented control in Settings → **Login Mode** controls how much KUVPN automates the login:

| Mode | Description |
|------|-------------|
| **Full Auto** | Browser runs headlessly. KUVPN fills in all fields automatically. Best for everyday use once your session is established. |
| **Visual Auto** | Browser window is visible but automation still runs. Useful for debugging or when you want to watch what's happening. |
| **Manual** | Browser window opens and you complete the login yourself. KUVPN waits for the DSID cookie, then starts OpenConnect. Use this the first time or when auto-login fails. |

After a successful manual login the session is saved, so Full Auto will work on future connects.

---

## System Tray

KUVPN minimises to the system tray when you close the window (if **Close to Tray** is set to Yes in Settings).

The tray icon reflects the current connection state:

| Icon | State |
|------|-------|
| Shield (normal) | Idle / ready |
| Shield + green checkmark | Connected |
| Shield + red X | Disconnected / error |

Right-clicking the tray icon gives you a menu to show/hide the window, connect, disconnect, or quit.

---

## Settings Reference

Settings are divided into **Basic** and **Advanced** sections. Use the **Basic / Advanced** toggle in the Settings tab header to switch between them.

### Basic settings (always visible)

| Setting | Description |
|---------|-------------|
| Family | Color palette for the app theme (e.g. Default, Ocean, Rose) |
| Tone | Dark or Light variant of the selected color family |
| KU Email | Pre-fill your university email for faster auto-login |
| Login Mode | Full Auto / Visual Auto / Manual (see [Login Modes](#login-modes)) |
| Close to Tray | **Yes**: closing the window minimises to tray and keeps the VPN running. **No**: closing the window exits the app and disconnects. |
| Auto-hide | **Yes**: the window hides automatically after a login prompt resolves, if it was brought up from the tray to show that prompt. |
| Window Style | **System**: native OS window borders. **Custom**: frameless window with a built-in titlebar that matches the app theme. |

### Advanced settings (visible when Advanced mode is on)

| Setting | Description |
|---------|-------------|
| Rounding | Corner radius style for buttons and cards (Square → Pill) |
| Shadow | Drop-shadow depth for cards and buttons (None → Elevated) |
| Gateway URL | The VPN portal URL. Default: `https://vpn.ku.edu.tr` |
| DSID Domain | Domain for DSID cookie matching. Default: `vpn.ku.edu.tr` |
| OC Path | Path to the `openconnect` binary. Leave blank to auto-detect. Click **Test** to verify. |
| Tunnel Mode | `Full` routes all traffic through the VPN. `Manual` lets you supply a custom vpnc-script for advanced routing. |
| VPN Script | Path to a custom vpnc-script (only shown in Manual tunnel mode). Click **Test** to validate before connecting. |
| Log Level | Controls how much is shown in the Console tab |
| Elevation | Privilege escalation tool: `sudo` or `pkexec` (Linux/macOS only) |

### Actions

At the bottom of the Settings tab:

- **Wipe Session** — deletes the saved login session. Use this if you get a "Cookie was rejected" error.
- **Reset Defaults** — restores all settings to their defaults.

---

## History

The **History** tab shows a log of past connection events:

| Entry | Description |
|-------|-------------|
| Connected | A new session was established |
| Reconnected | An automatic reconnect after a tunnel drop (`prev: Xm Ys` shows how long the previous segment lasted) |
| Disconnected | Clean disconnect. Shows total session duration. If the session included reconnects, shows "after N reconnect attempt(s)". |
| Cancelled | User-cancelled before tunnel came up |
| Error | Connection failed |

Click **Clear** to wipe the history.

---

## Automatic Reconnect

If the VPN tunnel drops unexpectedly, KUVPN automatically tries to reconnect up to 3 times with a short delay between each attempt. The status bar shows **Reconnecting... (attempt N/3)**. You can cancel at any time by clicking **Disconnect**.

When a stale session causes the initial tunnel to fail, KUVPN detects it, clears the session data, and retries automatically. A separator line is inserted in the console log so you can see what happened before and after the retry.

---

## Troubleshooting

### "Cookie was rejected by server"

Your saved session has expired. Go to **Settings → Wipe Session**, then reconnect.

### Auto-login fails / stuck in a login loop

Switch to **Manual** mode in Settings → Login Mode, connect, and complete the login yourself. Once the session is saved, switch back to Full Auto.

### OpenConnect not found (red ✗ on Test button)

OpenConnect is not installed or is not in the expected location. Enable **Advanced** mode in Settings and check the **OC Path** field. Install OpenConnect:

- **Linux:** `sudo apt install openconnect` (or `dnf`, `pacman`, etc.)
- **macOS:** `brew install openconnect`
- **Windows:** Reinstall using the Setup installer from the Releases page.

Or enter the full path to your `openconnect` binary in the OC Path field and click **Test**.

### App won't open on macOS ("app is damaged")

Run this command in Terminal:

```bash
sudo xattr -r -d com.apple.quarantine /Applications/KUVPN.app
```

This is a one-time step required because the app is not notarized through the Apple App Store.

### Connection shows "Connected" but VPN traffic is not working

This should be automatically detected and trigger a reconnect. If it persists, click **Disconnect** and reconnect manually. Check the Console tab for error details.

### Logs

The **Console** tab shows real-time output from the login and VPN process. Enable **Advanced** mode in Settings and increase the **Log Level** to `debug` or `trace` for more detail. Use **Copy Logs** to copy everything to your clipboard for bug reports.
