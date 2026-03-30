# Windows Installer Packaging

To build the KUVPN installer for Windows, follow these steps:

## 1. Build the binaries
You can build the binaries on Linux using the cross-compilation script:
```bash
./scripts/build_windows.sh
```
This will place `kuvpn-gui.exe` and `kuvpn-cli.exe` in `target/x86_64-pc-windows-gnu/release/`.

## 2. Bundle OpenConnect (Optional but Recommended)
For a better user experience, you should bundle OpenConnect.
1. Download the OpenConnect Windows binaries (e.g., from a reputable source like the official OpenConnect artifacts).
2. Create a folder `packaging/windows/openconnect/`.
3. Extract `openconnect.exe` and all its required DLLs (including `wintun.dll` if needed) into that folder.

## 3. Build the Installer
You need **Inno Setup** installed.
On Linux, you can use `iscc` under `wine`:
```bash
wine "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" packaging/windows/kuvpn.iss
```
On Windows, just open `kuvpn.iss` in the Inno Setup Compiler and click **Compile**.

The output will be `KUVPN-Setup.exe` in the `packaging/windows/` directory.
