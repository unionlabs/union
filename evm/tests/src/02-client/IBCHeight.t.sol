pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import {IBCHeightLib} from "../../../contracts/core/02-client/IBCHeight.sol";
import {IbcCoreClientV1Height} from
    "../../../contracts/proto/ibc/core/client/v1/client.sol";

// Required to have coverage counted.
contract IBCHeightProxy {
    function toUint128(IbcCoreClientV1Height.Data memory self)
        public
        pure
        returns (uint128)
    {
        return IBCHeightLib.toUint128(self);
    }

    function fromUint128(uint128 composite)
        public
        pure
        returns (IbcCoreClientV1Height.Data memory)
    {
        return IBCHeightLib.fromUint128(composite);
    }

    function isZero(IbcCoreClientV1Height.Data memory self)
        public
        pure
        returns (bool)
    {
        return IBCHeightLib.isZero(self);
    }

    function lt(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) public pure returns (bool) {
        return IBCHeightLib.lt(self, other);
    }

    function lte(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) public pure returns (bool) {
        return IBCHeightLib.lte(self, other);
    }

    function eq(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) public pure returns (bool) {
        return IBCHeightLib.eq(self, other);
    }

    function gt(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) public pure returns (bool) {
        return IBCHeightLib.gt(self, other);
    }

    function gte(
        IbcCoreClientV1Height.Data memory self,
        IbcCoreClientV1Height.Data memory other
    ) public pure returns (bool) {
        return IBCHeightLib.gte(self, other);
    }
}

contract IBCHeightTests is Test {
    IBCHeightProxy proxy;

    constructor() {
        proxy = new IBCHeightProxy();
    }

    function test_toUint128_fromUint128_iso(
        uint64 revisionNumber,
        uint64 revisionHeight
    ) public view {
        IbcCoreClientV1Height.Data memory height = IbcCoreClientV1Height.Data({
            revision_number: revisionNumber,
            revision_height: revisionHeight
        });
        assertTrue(proxy.eq(height, proxy.fromUint128(proxy.toUint128(height))));
    }

    function test_isZero() public view {
        assertTrue(
            proxy.isZero(
                IbcCoreClientV1Height.Data({
                    revision_number: 0,
                    revision_height: 0
                })
            )
        );
    }

    function test_eq(
        uint64 revisionNumber,
        uint64 revisionHeight
    ) public view {
        assertTrue(
            proxy.eq(
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumber,
                    revision_height: revisionHeight
                }),
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumber,
                    revision_height: revisionHeight
                })
            )
        );
    }

    function test_lt(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public view {
        vm.assume(revisionNumberB <= revisionNumberA);
        vm.assume(revisionHeightB < revisionHeightA);
        assertTrue(
            proxy.lt(
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                }),
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberA,
                    revision_height: revisionHeightA
                })
            )
        );
    }

    function test_lte(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public view {
        vm.assume(revisionNumberB <= revisionNumberA);
        vm.assume(revisionHeightB <= revisionHeightA);
        assertTrue(
            proxy.lte(
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                }),
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberA,
                    revision_height: revisionHeightA
                })
            )
        );
    }

    function test_gt(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public view {
        vm.assume(revisionNumberB >= revisionNumberA);
        vm.assume(revisionHeightB > revisionHeightA);
        assertTrue(
            proxy.gt(
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                }),
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberA,
                    revision_height: revisionHeightA
                })
            )
        );
    }

    function test_gte(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public view {
        vm.assume(revisionNumberB >= revisionNumberA);
        vm.assume(revisionHeightB >= revisionHeightA);
        assertTrue(
            proxy.gte(
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                }),
                IbcCoreClientV1Height.Data({
                    revision_number: revisionNumberA,
                    revision_height: revisionHeightA
                })
            )
        );
    }
}
