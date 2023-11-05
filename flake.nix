{
  description = "userspace process manager and controller";

  inputs = {
   nixpkgs.url = "nixpkgs/nixos-unstable";

   # rust
   fenix = {
     url = "github:nix-community/fenix";
     inputs.nixpkgs.follows = "nixpkgs";
   };

   # Provides some nice helpers for multiple system compatibility
   flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    # Calls the provided function for each "default system", which
    # is the standard set.
    flake-utils.lib.eachDefaultSystem
      (system:
        # instantiate the package set for the supported system, with our
        # rust overlay
        let
          pkgs = import nixpkgs { inherit system; };
          fenixPkgs = fenix.packages.${system};
          flakePkgs = {
            rust = fenixPkgs.fromToolchainFile {
              dir = ./.;
              sha256 = "sha256-rLP8+fTxnPHoR96ZJiCa/5Ans1OojI7MLsmSqR2ip8o=";
            };
          };
        in
        # "unpack" the pkgs attrset into the parent namespace
        with pkgs;
        {
          devShell = mkShell {
            nativeBuildInputs = [ bashInteractive ];
            buildInputs = [
              bashInteractive
              binutils
              cargo-edit
              gawk
              gnumake
              gnused
              jq
              nixFlakes
              nodejs_18
              openssl
              pkg-config
              postgresql_15
              flakePkgs.rust
              fenixPkgs.rust-analyzer
            ]
            # Things that don't work yet or don't work at all on 64-bit arm (i.e. M1s)
            ++ lib.optional (! pkgs.stdenv.isAarch64) [
              gdb
            ]
            # Alternatives for 64-bit arm macs
            ++ lib.optional (pkgs.stdenv.isAarch64) [
              lldb
            ]
            ++ lib.optional pkgs.stdenv.isDarwin [
              # The linker on MacOS requires the presence of certain "frameworks" from the
              # Apple SDK. A linker error like "framework not found: SystemConfiguration"
              # indicates that we need one of those frameworks for linking. To find the
              # available frameworks, pop open a nix repl (`nix repl`), run `:l <nixpkgs>`,
              # then type `pkgs.darwin.apple_sdk.frameworks.`, and then let tab-completion
              # show you what's available.
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
              libiconv  # dependency on MacOS
            ];
          };
        });
}
