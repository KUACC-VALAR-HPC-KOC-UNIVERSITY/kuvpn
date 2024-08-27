# KUVPN

KUVPN is a simple Rust program to retrieve the DSID cookie and execute the OpenConnect command to connect to the VPN for Koç University.

## Getting Started

These instructions will help you set up and run the project on your local machine.

### Prerequisites

Ensure you have the following installed on your system:

1. **Rust and Cargo**: Install Rust and Cargo using [rustup](https://rustup.rs/).

2. **ChromeDriver or GeckoDriver**:
   - **ChromeDriver**: Required if you choose to use the Chrome browser.
   - **GeckoDriver**: Required if you choose to use the Gecko browser (Firefox).

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

Or specify the URL, browser, and port:

```bash
kuvpn --url https://vpn.ku.edu.tr --browser gecko --port 9515
```

#### Using Environment Variables for Auto-Login

KUVPN supports automatic login by using environment variables to store your email and password. This feature allows the program to automatically fill in your credentials and proceed with the login process.

##### Setting Up Environment Variables

1. **Set the Environment Variables**:

   You need to set two environment variables: `KUVPN_EMAIL` and `KUVPN_PASSWORD`. You can do this by adding the following lines to your shell profile file (e.g., `.bashrc`, `.zshrc`, etc.):

   ```bash
   export KUVPN_USERNAME="your-email@example.com"
   export KUVPN_PASSWORD="your-password"
   ```

   Replace `"your-email@example.com"` and `"your-password"` with your actual email and password.

2. **Reload Your Shell Profile**:

   After adding the environment variables, reload your shell profile to apply the changes:

   ```bash
   source ~/.bashrc  # or source ~/.zshrc
   ```

#### Running KUVPN with Auto-Login

Once the environment variables are set, you can run KUVPN as usual, and it will automatically use the provided email and password for the login process:

```bash
kuvpn
```

Or with custom parameters:

```bash
kuvpn --url https://vpn.ku.edu.tr --browser chrome --port 9515
```

That's it! KUVPN will now handle the login process automatically using the credentials stored in the environment variables.

### Example

```bash
kuvpn --url https://vpn.ku.edu.tr --browser chrome --port 9515
```

This command will start the Chrome WebDriver, navigate to the specified URL, retrieve the DSID cookie, and construct the OpenConnect command.

### Command Line Options

```
Simple program to retrieve DSID cookie and execute OpenConnect command

Usage: kuvpn [OPTIONS]

Options:
  -u, --url <URL>          URL to visit [default: https://vpn.ku.edu.tr]
  -b, --browser <BROWSER>  Browser to use [default: chrome] [possible values: chrome, gecko, none]
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
   kuvpn --url https://vpn.ku.edu.tr --browser chrome --port 9515
   ```

That's it! You're all set up and ready to use KUVPN on Ubuntu. Enjoy!

## TODO List

- [x] Add automatic typing of email and password via environment variable as well as clicking next
- [x] Add Nix build for more reliable building
- [ ] Create AppImage
- [ ] Create another version with an embedded browser for those who prefer
- [ ] Add debug mode
- [ ] Write features on README
- [x] Explain how to use env varibles for auto-login

## Contributing

We welcome contributions from everyone! If you have an idea, fix, or improvement, please feel free to get involved.
