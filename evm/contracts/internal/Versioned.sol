library VersionedLib {
    function gitRev() internal pure returns (string memory) {
        return "dirty";
    }
}

abstract contract Versioned {
    function gitRev() public pure returns (string memory) {
        return VersionedLib.gitRev();
    }
}
