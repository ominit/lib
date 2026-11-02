{...}: {
  perSystem = {
    pkgs,
    rustToolchain,
    withMold,
    ...
  }: {
    devShells = {
      default =
        (pkgs.mkShell.override {
          stdenv = withMold pkgs.clangStdenv;
        }) {
          buildInputs = with pkgs; [
            cargo-nextest
            rustToolchain
            rust-analyzer
            cargo-watch
            samply

            just

            ripgrep
            findutils
            pkg-config
            openssl
          ];
        };
    };
  };
}
