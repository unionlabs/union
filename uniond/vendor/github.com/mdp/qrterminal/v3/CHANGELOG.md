## 3.2.0

- Update to add sixel support #29
- Update deps to latest

## 3.1.1

- Update deps to latest

## 3.1.0

- Add the ability to accept input string from stdin
- Integrate github actions for build and release
- Release support for Darwin M1/M2(aarch64)

## 3.0.0

Adjust go.mod to include required version string

## 2.0.1

Add goreleaser and release to Homebrew and Github

## 2.0.0

Add a command line tool and QuietZone around QRCode

## 1.0.1

Add go.mod

## 1.0.0

Update to add a quiet zone border to the QR Code - #5 and fixed by [WindomZ](https://github.com/WindomZ) #8

  - This can be configured with the `QuietZone int` option
  - Defaults to 4 'pixels' wide to match the QR Code spec
  - This alters the size of the barcode considerably and is therefore a breaking change, resulting in a bump to v1.0.0

## 0.2.1 

Fix direction of the qr code #6 by (https://github.com/mattn)
