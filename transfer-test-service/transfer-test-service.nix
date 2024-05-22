{ ... }: {
  perSystem = { self', pkgs, ensureAtRepositoryRoot, ... }: {
    packages = {
      transfer-test-service = pkgs.writeShellApplication {
        name = "transfer-test-service";
        runtimeInputs = [ pkgs.rustc pkgs.cargo ];
        text = ''
          echo "log_me_hard_bro"
          ${ensureAtRepositoryRoot}
          echo "Rustc version: $(rustc --version)"
          echo "Which rustc: $(which rustc)"
          cargo run --manifest-path transfer-test-service/Cargo.toml
        '';
      };
    };
  };
}
