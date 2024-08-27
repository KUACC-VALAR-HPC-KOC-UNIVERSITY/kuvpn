{
  description = "KUVPN";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
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
          version = "0.4.3";

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
          ];
          # Runtime dependencies for the shell environment
          nativeBuildInputs = [
            pkgs.rust-analyzer
            pkgs.lldb_18
            pkgs.chromium
            pkgs.chromedriver
          ];
        };

        # To run your package using `nix run`
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/kuvpn";
          runtimeDependencies = [
            pkgs.chromium
            pkgs.chromedriver
          ];
        };
      }
    );
}
