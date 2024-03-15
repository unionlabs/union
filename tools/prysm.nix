{ ... }: {
  perSystem = {pkgs, config, self', goPkgs, ... }: 
  let 
    CGO_CFLAGS = "-I${self'.packages.libblst}/include -I${self'.packages.libblst.src}/src -I${self'.packages.libblst.src}/build -I${self'.packages.bls-eth.src}/bls/include -O";
    CGO_LDFLAGS = "-z noexecstack -L${self'.packages.bls-eth}/lib -s -w";
  in
  {
    packages.prysm = goPkgs.buildGo121Module rec {
      inherit CGO_CFLAGS;
      inherit CGO_LDFLAGS;

      pname = "prysm";
      version = "5.0.1";
      src = builtins.fetchGit {
        url = "https://github.com/unionlabs/prysm";
        rev = "7ca368a432a453a434c9428b6587c95f169f74b5";
      };


      # src = pkgs.fetchFromGitHub {
      #   owner = "unionlabs";
      #   repo = pname;
      #   rev = "";
      #   hash = "sha256-QSfTDjdsd6XSYDjhDL+p/0cYlXysVHqlzVMF2Kaaa/a=";
      # };

      vendorHash = "sha256-5ToSVJ7ToDhRq3AT6I6G8FE8GqZRPCMyTogTQG4HDBY=";

      buildInputs = [pkgs.libelf];

      subPackages = [
        "cmd/beacon-chain"
        "cmd/client-stats"
        "cmd/prysmctl"
        "cmd/validator"
      ];

      doCheck = false;

      tags = "minimal develop noMainnetGenesis";

      ldflags = [
        # "-s"
        # "-w"
        # "-linkmode external"
        # "-extldflags '-static -L${pkgs.musl}/lib -s -w'"
        "-X github.com/prysmaticlabs/prysm/v4/runtime/version.gitTag=v${version}"
      ];

      meta = {
        description = "Go implementation of Ethereum proof of stake";
        homepage = "https://github.com/prysmaticlabs/prysm";
        mainProgram = "beacon-chain";
        platforms = ["x86_64-linux" "aarch64-linux"];
      };
    };
  };
}
