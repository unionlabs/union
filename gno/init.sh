echo
echo ===
echo REGISTER CLIENT
echo ===
echo

echo | gnokey maketx call -pkgpath "gno.land/r/union/lightclients/cometbls" -func "Register" -gas-fee 1000000ugnot -gas-wanted 20000000 -broadcast g1jg8mtutu9khhfwc4nxmuhcpftf0pajdhfvsqf5 -insecure-password-stdin -quiet

echo
echo ===
echo CREATE CLIENT
echo ===
echo

echo | gnokey maketx call -pkgpath "gno.land/r/union/core" -func "CreateClient" -args cometbls -args "YWJj" -args "YWJj" -gas-fee 1000000ugnot -gas-wanted 20000000 -broadcast g1jg8mtutu9khhfwc4nxmuhcpftf0pajdhfvsqf5 -insecure-password-stdin -quiet

