# KUVPN

KUVPN is a simple Rust program to retrieve the DSID cookie and execute the OpenConnect command to connect to the VPN for Koç University.

## Getting Started

These instructions will help you set up and run the project on your local machine.

### Prerequisites

Ensure you have the following installed on your system:

1. **Rust and Cargo**: Install Rust and Cargo using [rustup](https://rustup.rs/).

2. **ChromeDriver**:
   - **ChromeDriver**: Install from package manager or website (together with chromium/chrome)

3. **Openconnect**:
   - **openconnect**: Most likely will already be installed, can usaully be installed with system package manager.

### Installation

1. **Clone and Install KUVPN**:
    ```bash
    cargo install --git https://github.com/ealtun21/kuvpn
    ```

### Usage

Run the program with default parameters:

```bash
kuvpn
```

Or specify the URL or and port for chromedriver:

```bash
kuvpn --url https://vpn.ku.edu.tr --port 9515
```

### Command Line Options

```
Simple program to retrieve DSID cookie and execute OpenConnect command

Usage: kuvpn [OPTIONS]

Options:
  -u, --url <URL>          URL to visit [default: https://vpn.ku.edu.tr]
  -p, --port <PORT>        Port to use for WebDriver [default: 9515]
  -h, --help               Print help
  -V, --version            Print version
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Ubuntu Quick Install

Follow these steps to quickly set up KUVPN on Ubuntu:

1. **Install Rust and Cargo using rustup**:

   To install Rust and Cargo, you will use the rustup script. Open a terminal and run:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   After running the script, follow the on-screen instructions to complete the installation. Once the installation is finished, you will need to restart your terminal session for the changes to take effect.

   **Alternatively, you can use the following command to reload your shell without closing the terminal:**

   ```bash
   source $HOME/.cargo/env
   ```

   This command updates your current shell session with the new environment variables set by Rust.

2. **Install Chromium and ChromeDriver**:

   If you plan to use the Chrome browser for KUVPN, you will need to install Chromium and its corresponding ChromeDriver:

   ```bash
   sudo apt-get install chromium-chromedriver
   ```

3. **Install KUVPN**:

   With Rust and Cargo installed, you can now install KUVPN directly from the Git repository:

   ```bash
   cargo install --git https://github.com/ealtun21/kuvpn
   ```

4. **Run KUVPN**:

   Once KUVPN is installed, you can run it with the default settings:

   ```bash
   kuvpn
   ```

   Or, if you want to customize the connection parameters, use:

   ```bash
   kuvpn --url https://vpn.ku.edu.tr --port 9515
   ```

That's it! You're all set up and ready to use KUVPN on Ubuntu. Enjoy!

## TODO List

- [x] Add Nix build for more reliable building
- [x] Can remember session
- [ ] Create AppImage
- [ ] Create another version with an embedded browser for those who prefer
- [ ] Add debug mode
- [ ] Write features on README

## Contributing

We welcome contributions from everyone! If you have an idea, fix, or improvement, please feel free to get involved.
