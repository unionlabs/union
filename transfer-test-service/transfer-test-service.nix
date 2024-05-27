{ ... }: {
  perSystem = { self', pkgs, ensureAtRepositoryRoot, ... }: {
    packages = {
      transfer-test-service = pkgs.writeShellApplication {
        name = "transfer-test-service";
        runtimeInputs = [ pkgs.rustc pkgs.cargo ];
        text = ''
          ${ensureAtRepositoryRoot}
          cargo run --manifest-path transfer-test-service/Cargo.toml
        '';
      };
    };
  };
}
