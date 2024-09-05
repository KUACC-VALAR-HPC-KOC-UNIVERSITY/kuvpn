# KUVPN v0.6.0

KUVPN is a simple Rust program to retrieve the DSID cookie and execute the OpenConnect command to connect to the VPN for Koç University.

## Features

- [x] Login to vpn.ku.edu.tr in linux/mac.
- [x] Retrieve DSID cookie
- [x] Execute OpenConnect command
- [x] Customizable URL
- [x] Remembers your login session safely
- [x] Nix build for reliable building
- [x] Logging for debugging tool

# Prerequisites
- OpenConnect
   - On ubuntu: `sudo apt install openconnect`
- Optional: Chromium/Chrome (KUVPN will attempt to download it if not found)

# Option 1: Install (Binary) (Recommended)

This command will download kuvpn, and install it to your `/usr/bin/`
```
curl --proto '=https' --tlsv1.2 -sSfL https://github.com/ealtun21/kuvpn/releases/download/v0.6.0/kuvpn-x86_64-unknown-linux-musl -o /usr/bin/kuvpn && chmod +x /usr/bin/kuvpn
```

# Option 2: Build & Install (Source Code)

First, install Rustup using this command or your package manager. While you can use Cargo from a package manager, it is not recommended:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Afterward, you can install KUVPN from source using this command:
```
cargo install --git https://github.com/ealtun21/kuvpn
```

# Usage

You may simply run: 
```
kuvpn
```

For help you may run:
```
kuvpn --help
```

For more information on what it does while running, you may use:
```
kuvpn --level info
```

kuvpn --help:
```
Simple program to retrieve DSID cookie and execute OpenConnect command

Usage: kuvpn [OPTIONS]

Options:
  -u, --url <URL>
          The URL to the page where we will start logging in and looking for DSID
          
          [default: https://vpn.ku.edu.tr]

  -l, --level <LEVEL>
          The level of logging
          
          [default: error]

          Possible values:
          - off:   No logs
          - info:  Informational messages
          - warn:  Warning messages
          - debug: Debugging messages
          - error: Error messages
          - trace: Detailed stacktrace messages

  -d, --dsid
          Gives the user the dsid without running openconnect

  -c, --clean
          Delete session information

  -a, --agent <AGENT>
          User agent for browser
          
          [default: Mozilla/5.0]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


## Contributing

We welcome contributions from everyone! If you have an idea, fix, or improvement, please feel free to get involved.
