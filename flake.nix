{
  description = "tummycrypt/tcfs - FOSS self-hosted odrive replacement";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [
            "x86_64-unknown-linux-gnu"
            "aarch64-unknown-linux-gnu"
            "x86_64-apple-darwin"
            "aarch64-apple-darwin"
          ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common build inputs for all crates
        commonBuildInputs = with pkgs; [
          protobuf
          pkg-config
          openssl
        ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
          fuse3
          rocksdb
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          # macOS: FUSE-T provides libfuse3-compatible headers via macfuse-stubs
          # Users install FUSE-T separately: https://github.com/macos-fuse-t/fuse-t
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Build workspace
        workspace = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          buildInputs = commonBuildInputs;
          nativeBuildInputs = with pkgs; [ pkg-config protobuf ];
        } // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          ROCKSDB_INCLUDE_DIR = "${pkgs.rocksdb}/include";
          ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
        };

      in {
        packages = {
          default = workspace;
          tcfsd = workspace;
          tcfs-cli = workspace;
          tcfs-tui = workspace;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = commonBuildInputs ++ (with pkgs; [
            rustToolchain

            # Proto codegen
            protobuf

            # Security tooling
            age
            sops

            # Infrastructure
            opentofu
            kubectl
            kubernetes-helm
            kustomize

            # Build tooling
            go-task
            cargo-watch
            cargo-deny
            cargo-audit

            # NATS
            natscli

            # Dev tools
            git
            yq-go
          ]);

          shellHook = ''
            echo "tcfs devShell (tummycrypt monorepo)"
            echo "  task --list      # show available tasks"
            echo "  cargo build      # build workspace"
            echo "  task dev         # start local stack + watch"
          '';

        } // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          ROCKSDB_INCLUDE_DIR = "${pkgs.rocksdb}/include";
          ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
        };
      }
    ) // {
      # NixOS modules (system-level)
      nixosModules.tcfsd = import ./nix/modules/tcfs-daemon.nix;

      # Home Manager modules (user-level)
      homeManagerModules.tcfs = import ./nix/modules/tcfs-user.nix;
    };
}
