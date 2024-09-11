{
  description = "KUVPN";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            pkg-config
            rust-bin.beta.latest.default
            pkgs.openssl
            pkgs.libclang
            pkgs.libiconv
          ];
        };

        # Define the default package (to fix nix run)
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "kuvpn";
          version = "0.6.2";

          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          cargoBuildFlags = [
            "--release"
          ];
          
          nativeBuildInputs = [ pkgs.pkg-config pkgs.rustfmt ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libclang.dev}/lib/pkgconfig";
          LIBCLANG_PATH = "${pkgs.libclang}/lib";

          meta = with pkgs.lib; {
            description = "KUVPN - A Rust-based VPN application";
            license = licenses.mit;
            maintainers = [ maintainers.ealtun21 ];
            platforms = platforms.linux;
          };
        };

        # To run your package using `nix run`
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/kuvpn";
        };
      }
    );
}
