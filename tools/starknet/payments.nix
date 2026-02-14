_: {
  perSystem =
    {
      pkgs,
      pkgsUnstable,
      crane,
      rust,
      dbg,
      ...
    }:
    let
      pkgs' = import pkgsUnstable.path {
        inherit (pkgs) system;
        overlays = [
          (final: prev: {
            python3 = prev.python312.override {
              packageOverrides = pyFinal: pyPrev: {
                ecdsa = pyPrev.buildPythonPackage rec {
                  pname = "ecdsa";
                  version = "0.18.0";
                  format = "setuptools";

                  src = pyPrev.fetchPypi {
                    inherit pname version;
                    hash = "sha256-GQNIBBVZ4hsiodZc7khSgsoRpvgdUD/duE1QF+ntHkk=";
                  };

                  propagatedBuildInputs = [ pyPrev.six ];
                  # Only needed for tests
                  nativeCheckInputs = [ pkgs.openssl ];
                };
                cairo-lang = pyPrev.buildPythonPackage {
                  pname = "cairo-lang";
                  version = "0.14.0.1";
                  format = "setuptools";
                  # pyproject = true;
                  doCheck = false;

                  src = pkgs.fetchurl {
                    url = "https://files.pythonhosted.org/packages/3a/76/7edf7675b6b3eca77aa304a0ff5bba045a351716a72b0f2a892117ca4838/cairo-lang-0.14.0.1.zip";
                    hash = "sha256-1TRzJF+NDX1i13rVwXFZWfYc4Si7Np6OdmIECqvtmN8=";
                  };
                };
              };
            };
          })
        ];
      };
    in

    {
      packages = {
        starknet-scripts = pkgs.mkRootDrv "starknet-scripts" {
          attest-to-message =
            pkgs'.writers.writePython3Bin "payments-attest-to-message"
              {
                libraries = with pkgs'.python3.pkgs; [
                  cairo-lang
                  ecdsa
                  mpmath
                  sympy
                  numpy
                ];
              }
              ''
                from starkware.crypto.signature.signature import sign, private_to_stark_key
                import os

                # Stark curve order
                STARK_ORDER = (
                  3618502788666131213697322783095070105526743751716087489154079457884512865583
                )

                # 1) Generate private key
                priv = int(os.environ["PRIVATE_KEY"])
                if priv > STARK_ORDER:
                    print("Private the exceeds the curve order")
                    exit(1)
                # priv = {private-key}

                # 2) Read message hash
                h = int(os.environ["MSGHASH_DEC"])
                # h = {msg-hash}
                # Stark-compatible hash (< 2**251)
                MOD = 1 << 251
                h251 = h % MOD
                # 3) Sign
                r, s = sign(h251, priv)

                # 4) Derive public key (single integer, Stark curve)
                pub = private_to_stark_key(priv)

                # 5) Print everything
                print("private_key =", hex(priv))
                print("public_key  =", hex(pub))
                print("signed_hash =", hex(h251))
                print("r           =", hex(r))
                print("s           =", hex(s))
              '';
        };
      };
    };
}
