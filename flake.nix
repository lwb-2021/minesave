{
  description = "Minesave flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        name = "minesave";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-std"
            "rust-analyzer"
            "rustfmt"
          ];
        };
        nativeDeps = with pkgs; [
          libgcc.libgcc

          libGL
          wayland
          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr

          fontconfig
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config

            # pkgs.rustfmt
          ]
          ++ nativeDeps;

          RUST_BACKTRACE = 1;

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath nativeDeps}:$LD_LIBRARY_PATH"
          '';
        };

        packages = rec {
          minesave =
            (pkgs.makeRustPlatform {
              cargo = rustToolchain;
              rustc = rustToolchain;
            }).buildRustPackage
              rec {
                pname = "${name}";
                version = "0.1.0";

                src = ./.;

                cargoLock.lockFile = src + /Cargo.lock;

                cargoSha256 = nixpkgs.lib.fakeSha256;
                nativeBuildInputs = with pkgs; [
                  pkg-config

                  makeWrapper
                ];
                buildInputs = nativeDeps;
                RUSTFLAGS = "--cfg=web_sys_unstable_apis";

                postInstall = ''
                  wrapProgram "$out/bin/${name}" --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath nativeDeps}"
                '';
              };
          default = minesave;
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/${name}";
        };
      }
    );
}
