{
  description = "KUVPN";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        rustPlatform = pkgs.rustPlatform;
      in
      {
        # Define the default package
        packages.default = rustPlatform.buildRustPackage rec {
          pname = "kuvpn";
          version = "0.5.0";

          src = ./.;

          name = "${pname}-${version}";

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          cargoHash = "sha256-yejviZYX11G/KtfJFFQv6bGq0jD+04Rz3/6Wf2lL8zs=";

          # No need for buildInputs for runtime dependencies
          buildInputs = [ ];

          # Explicitly set environment variables for pkg-config
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

          meta = with lib; {
            description = "KUVPN - A Rust-based VPN application";
            license = licenses.mit;
            maintainers = with lib.maintainers; [ "ealtun21" ];
            platforms = platforms.linux;
          };
        };

        # Define the devShell
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.rustfmt
            pkgs.cargo
            pkgs.chromedriver
            pkgs.openconnect
            pkgs.chromium # or pkgs.google-chrome, pkgs.google-chrome-stable, pkgs.google-chrome-beta, pkgs.google-chrome-unstable
          ];

          nativeBuildInputs = [
            pkgs.rust-analyzer
            pkgs.lldb_18
          ];
        };

        # To run your package using `nix run`
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/kuvpn";
        };

        # Define Chromium and Chromedriver as apps
        apps.chromium = {
          type = "app";
          program = "${pkgs.chromium}/bin/chromium";
        };

        apps.chromedriver = {
          type = "app";
          program = "${pkgs.chromedriver}/bin/chromedriver";
        };
      }
    );
}
