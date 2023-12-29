{ self, ... }: {
  perSystem = { self', pkgs, ... }:
    let
      faucet = pkgs.buildGoModule {
        name = "faucet";
        vendorHash = "sha256-LDu9GSgMsCHTk5K7hsEhLg36SatUpgQZrOdEvuPSM84=";
        meta.mainProgram = "cosmos-faucet";
        version = "2.0.0";
        src = pkgs.fetchFromGitHub {
          name = "faucet";
          owner = "okp4";
          repo = "cosmos-faucet";
          rev = "18ac81747e6ea9503b13b65ec01388c498caaa9c";
          sha256 = "sha256-re7c0itkTmDwuvuwluLxhrpAyngzMhp9Ec9DuOGlvfc=";
        };
      };
    in
    {
      packages = {
        inherit faucet;

        faucet-image = pkgs.dockerTools.buildLayeredImage {
          name = "faucet";
          contents = [ pkgs.coreutils-full pkgs.cacert self'.packages.faucet ];
          config = {
            Entrypoint = [ (pkgs.lib.getExe self'.packages.faucet) ];
            Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };
        };
      };
    };

  flake.nixosModules.faucet = { lib, pkgs, config, ... }:
    with lib;
    let cfg = config.services.faucet;
    in {
      options.services.faucet = {
        enable = mkEnableOption "Faucet service";
        package = mkOption {
          type = types.package;
          default = self.packages.${pkgs.system}.faucet;
        };
        home = mkOption {
          type = types.str;
          default = "";
          description = "the home folder";
        };
        mnemonic = mkOption {
          type = types.str;
          default = "";
          description = "wallet mnemonic";
        };
        address = mkOption {
          type = types.str;
          default = ":8080";
          description = "graphql api address";
        };
        captcha-secret = mkOption {
          type = types.str;
          default = "";
          description = "if set, a captcha is required";
        };
        captcha-verify-url = mkOption {
          type = types.str;
          default = "https://www.google.com/recaptcha/api/siteverify";
          description = "Captcha verify URL";
        };
        chain-id = mkOption {
          type = types.str;
          default = "union-testnet-3";
        };
        denom = mkOption {
          type = types.str;
          default = "muno";
        };
        memo = mkOption {
          type = types.str;
          default = "Join the Union";
        };
        prefix = mkOption {
          type = types.str;
          default = "union1";
        };
        amount-send = mkOption {
          type = types.int;
          default = 1;
        };
        grpc-address = mkOption {
          type = types.str;
          default = "127.0.0.1:9090";
          description = "grpc address of the node";
        };
        extra-args = mkOption {
          type = types.str;
          default = "";
          description = "extra arguments to pass to the command";
        };
      };

      config = mkIf cfg.enable {
        systemd.services.faucet =
          let
            faucet-systemd-script = pkgs.writeShellApplication {
              name = "faucet-systemd";
              runtimeInputs = [ pkgs.coreutils cfg.package ];
              text =
                let
                  captcha = if cfg.captcha-secret != "" then "--captcha --captcha-secret '${cfg.captcha-secret}'" else "";
                  captcha-verify-url = if cfg.captcha-verify-url != "" then "--captcha-verify-url '${cfg.captcha-verify-url}'" else "";
                in
                ''
                  ${pkgs.lib.getExe cfg.package}  \
                      --chain-id '${cfg.chain-id}' \
                      --denom '${cfg.denom}' \
                      --memo '${cfg.memo}' \
                      --prefix '${cfg.prefix}' \
                      start \
                      ${captcha} \
                      ${captcha-verify-url} \
                      --address '${cfg.address}' \
                      --amount-send ${toString cfg.amount-send} \
                      --grpc-address ${cfg.grpc-address} \
                      --mnemonic '${cfg.mnemonic}' ${cfg.extra-args}
                '';
            };
          in
          {
            wantedBy = [ "multi-user.target" ];
            description = "Faucet";
            serviceConfig = {
              Type = "simple";
              ExecStart = pkgs.lib.getExe faucet-systemd-script;
              Restart = mkForce "always";
            };
            environment = {
              HOME = cfg.home;
            };
          };
      };
    };

}
