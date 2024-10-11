{ ... }:
{
  perSystem = { pkgs, self', system, ... }: {
    packages = {
      clientv2 = pkgs.go.stdenv.mkDerivation {
        name = "cosmossdk.io/client/v2";
        buildInputs = [ pkgs.go pkgs.gotestsum pkgs.git pkgs.which ];
        src = ./.;
        doCheck = true;
        dontInstall = true;
        dontBuild = true;
        checkPhase = ''
          export HOME=$(pwd)
          export GOFLAGS="-mod=vendor -tags=\"cgo,ledger,test_ledger_mock,norace\""
          export GOPRIVATE="github.com/unionlabs/*";
          export GOWORK="off"

          echo "Running test for cosmossdk.io/client/v2 sub module"
          go version

          gotestsum  ./...

          echo "Finished running sub module tests."
          
          touch $out
        '';
      };
    };
  };
}
