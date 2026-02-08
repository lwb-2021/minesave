{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "git+https://mirrors.tuna.tsinghua.edu.cn/git/nixpkgs.git?ref=nixpkgs-unstable&shallow=1";
    # nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };

        deps = with pkgs; [
          pkg-config
          glib
          cairo
          gdk-pixbuf
          pango
          gtk4
        ];
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          buildInputs = deps;
        };
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
            ]
            ++ deps;
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
