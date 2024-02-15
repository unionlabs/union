{ inputs, ... }: {
  perSystem = { pkgs, ... }:
    let
      enableShared = !pkgs.stdenv.hostPlatform.isStatic;
    in
    with pkgs; {
      packages.libblst = stdenv.mkDerivation rec {
        pname = "blst";
        version = inputs.blst.shortRev;
        src = inputs.blst;
        buildPhase = ''
          ./build.sh
        '' + lib.optionalString enableShared ''
          ./build.sh -shared
        '';
        installPhase = ''
          mkdir -p $out/{lib,include}
          for lib in libblst.{a,so,dylib}; do
            if [ -f $lib ]; then
              cp $lib $out/lib/
            fi
          done
          cp bindings/{blst.h,blst_aux.h} $out/include

          for lib in blst.dll; do
            if [ -f $lib ]; then
              mkdir -p $out/bin
              cp $lib $out/bin/
            fi
          done

          mkdir -p $out/lib/pkgconfig
          cat <<EOF > $out/lib/pkgconfig/libblst.pc
          prefix=$out
          exec_prefix=\''${prefix}
          libdir=\''${exec_prefix}/lib
          includedir=\''${prefix}/include

          Name: libblst
          Description: ${meta.description}
          URL: ${meta.homepage}
          Version: ${version}

          Cflags: -I\''${includedir}
          Libs: -L\''${libdir} -lblst
          Libs.private:
          EOF
        '';
        # ensure we have the right install id set.  Otherwise the library
        # wouldn't be found during install.  The alternative would be to work
        # lib.optional stdenv.isDarwin "LDFLAGS=-Wl,-install_name,$(out)/lib/libblst.dylib";
        # into the setup.sh
        postFixup = lib.optionalString (stdenv.isDarwin && enableShared) ''
          install_name_tool -id $out/lib/libblst.dylib $out/lib/libblst.dylib
        '';
        enableParallelBuilding = true;
        doCheck = true;
        meta = with lib; {
          description = "Multilingual BLS12-381 signature library";
          homepage = "https://github.com/supranational/blst";
          license = licenses.isc;
          platforms = platforms.all;
        };
      };
    };
}
