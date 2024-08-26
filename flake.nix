{
  description = "KUVPN";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      lib = pkgs.lib;
      rustPlatform = pkgs.rustPlatform;
    in {
      # Define the default package
      packages.default = rustPlatform.buildRustPackage rec {
        pname = "kuvpn";
        version = "0.2.1";

        src = ./.;

        name = "${pname}-${version}";

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        cargoHash = "sha256-yejviZYX11G/KtfJFFQv6bGq0jD+04Rz3/6Wf2lL8zs=";

        buildInputs = [
          pkgs.openssl
          pkgs.pkg-config
          pkgs.chromium
          pkgs.chromedriver
        ];

        nativeBuildInputs = [
          pkgs.openssl
          pkgs.pkg-config
        ];

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
          pkgs.openssl
          pkgs.pkg-config
          pkgs.rustc
          pkgs.cargo
          pkgs.chromium
          pkgs.chromedriver
        ];
      };

      # To run your package using `nix run`
      apps.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/kuvpn";
      };
    });
}
