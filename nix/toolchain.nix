{inputs, ...}: {
  perSystem = {
    pkgs,
    system,
    ...
  }: {
    _module.args = {
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [(import inputs.rust-overlay)];
      };
      rustToolchain =
        pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {extensions = ["miri" "rust-src"];});

      withMold = stdenv:
        if stdenv.hostPlatform.isLinux
        then pkgs.stdenvAdapters.useMoldLinker stdenv
        else stdenv;
    };
  };
}
