pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import {MerkleTree} from "../../contracts/lib/MerkleTree.sol";

// Required to have coverage counted.
contract MerkleProxy is Test {
    function hashFromByteSlices(bytes[] memory data)
        public
        pure
        returns (bytes32)
    {
        return MerkleTree.hashFromByteSlices(data);
    }

    function optimizedBlockRoot(bytes32[14] memory data)
        public
        pure
        returns (bytes32)
    {
        return MerkleTree.optimizedBlockRoot(data);
    }
}

contract MerkleTreeTests is Test {
    MerkleProxy proxy;

    constructor() {
        proxy = new MerkleProxy();
    }

    function getOptimizedRoot(
        bytes memory a,
        bytes memory b,
        bytes memory c,
        bytes memory d,
        bytes memory e,
        bytes memory f,
        bytes memory g,
        bytes memory h,
        bytes memory i,
        bytes memory j,
        bytes memory k,
        bytes memory l,
        bytes memory m,
        bytes memory n
    ) public view returns (bytes32) {
        return proxy.optimizedBlockRoot(
            [
                MerkleTree.leafHash(a),
                MerkleTree.leafHash(b),
                MerkleTree.leafHash(c),
                MerkleTree.leafHash(d),
                MerkleTree.leafHash(e),
                MerkleTree.leafHash(f),
                MerkleTree.leafHash(g),
                MerkleTree.leafHash(h),
                MerkleTree.leafHash(i),
                MerkleTree.leafHash(j),
                MerkleTree.leafHash(k),
                MerkleTree.leafHash(l),
                MerkleTree.leafHash(m),
                MerkleTree.leafHash(n)
            ]
        );
    }

    function getNonOptimizedRoot(
        bytes memory a,
        bytes memory b,
        bytes memory c,
        bytes memory d,
        bytes memory e,
        bytes memory f,
        bytes memory g,
        bytes memory h,
        bytes memory i,
        bytes memory j,
        bytes memory k,
        bytes memory l,
        bytes memory m,
        bytes memory n
    ) public view returns (bytes32) {
        bytes[] memory inputs = new bytes[](14);
        inputs[0] = a;
        inputs[1] = b;
        inputs[2] = c;
        inputs[3] = d;
        inputs[4] = e;
        inputs[5] = f;
        inputs[6] = g;
        inputs[7] = h;
        inputs[8] = i;
        inputs[9] = j;
        inputs[10] = k;
        inputs[11] = l;
        inputs[12] = m;
        inputs[13] = n;
        return proxy.hashFromByteSlices(inputs);
    }

    function testOptimized_nonOptimized_eq(
        bytes memory a,
        bytes memory b,
        bytes memory c,
        bytes memory d,
        bytes memory e,
        bytes memory f,
        bytes memory g,
        bytes memory h,
        bytes memory i,
        bytes memory j,
        bytes memory k,
        bytes memory l,
        bytes memory m,
        bytes memory n
    ) public view {
        bytes32 optimizedRoot =
            getOptimizedRoot(a, b, c, d, e, f, g, h, i, j, k, l, m, n);
        bytes32 root =
            getNonOptimizedRoot(a, b, c, d, e, f, g, h, i, j, k, l, m, n);
        assertEq(optimizedRoot, root);
    }
}
