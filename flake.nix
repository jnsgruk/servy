{
  description = "servy web server flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsForSystem = system: (import nixpkgs { inherit system; });
    in
    {
      packages = forAllSystems (
        system:
        let
          inherit (pkgsForSystem system)
            buildEnv
            cacert
            callPackage
            dockerTools
            fetchFromGitHub
            hugo
            lib
            openssl
            pkg-config
            rustPlatform
            stdenv
            ;

          version = self.shortRev or (builtins.substring 0 7 self.dirtyRev);
        in
        rec {
          default = servy;

          jnsgruk-container = callPackage ./nix/jnsgruk-container.nix {
            inherit
              buildEnv
              cacert
              dockerTools
              jnsgruk
              lib
              version
              ;
          };

          jnsgruk-content = callPackage ./nix/jnsgruk-content.nix {
            inherit
              fetchFromGitHub
              hugo
              lib
              stdenv
              ;
          };

          jnsgruk = callPackage ./nix/jnsgruk.nix {
            inherit
              hugo
              jnsgruk-content
              lib
              openssl
              pkg-config
              rustPlatform
              version
              ;
          };

          servy = callPackage ./nix/servy.nix {
            inherit
              lib
              openssl
              pkg-config
              rustPlatform
              version
              ;
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsForSystem system;
        in
        {
          default = pkgs.mkShell {
            name = "servy";
            NIX_CONFIG = "experimental-features = nix-command flakes";
            SERVY_ASSETS_DIR = "./tests/servy_assets";
            inputsFrom = [ self.packages.${system}.servy ];
            buildInputs = with pkgs; [
              rustc
              cargo
              clippy
              rust-analyzer
            ];
          };
        }
      );
    };
}
