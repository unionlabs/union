pragma solidity ^0.8.23;

import {IBCHeight} from "../../../contracts/core/02-client/IBCHeight.sol";
import {IbcCoreClientV1Height} from "../../../contracts/proto/ibc/core/client/v1/client.sol";

import "../TestPlus.sol";
import "solady/utils/LibString.sol";
import "solidity-bytes-utils/BytesLib.sol";

contract IBCHeightTests is TestPlus {
    using IBCHeight for *;

    function test_ibcHeight_toUint128_fromUint128_iso(
        uint64 revisionNumber,
        uint64 revisionHeight
    ) public {
        IbcCoreClientV1Height.Data memory height = IbcCoreClientV1Height.Data({
            revision_number: revisionNumber,
            revision_height: revisionHeight
        });
        height.toUint128().fromUint128().eq(height);
    }

    function test_ibcHeight_isZero() public {
        assertTrue(
            IbcCoreClientV1Height
                .Data({revision_number: 0, revision_height: 0})
                .isZero()
        );
    }

    function test_ibcHeight_eq(
        uint64 revisionNumber,
        uint64 revisionHeight
    ) public {
        assertTrue(
            IbcCoreClientV1Height
                .Data({
                    revision_number: revisionNumber,
                    revision_height: revisionHeight
                })
                .eq(
                    IbcCoreClientV1Height.Data({
                        revision_number: revisionNumber,
                        revision_height: revisionHeight
                    })
                )
        );
    }

    function test_ibcHeight_lt(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public {
        vm.assume(revisionNumberB <= revisionNumberA);
        vm.assume(revisionHeightB < revisionHeightA);
        assertTrue(
            IbcCoreClientV1Height
                .Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                })
                .lt(
                    IbcCoreClientV1Height.Data({
                        revision_number: revisionNumberA,
                        revision_height: revisionHeightA
                    })
                )
        );
    }

    function test_ibcHeight_lte(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public {
        vm.assume(revisionNumberB <= revisionNumberA);
        vm.assume(revisionHeightB <= revisionHeightA);
        assertTrue(
            IbcCoreClientV1Height
                .Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                })
                .lte(
                    IbcCoreClientV1Height.Data({
                        revision_number: revisionNumberA,
                        revision_height: revisionHeightA
                    })
                )
        );
    }

    function test_ibcHeight_gt(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public {
        vm.assume(revisionNumberB >= revisionNumberA);
        vm.assume(revisionHeightB > revisionHeightA);
        assertTrue(
            IbcCoreClientV1Height
                .Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                })
                .gt(
                    IbcCoreClientV1Height.Data({
                        revision_number: revisionNumberA,
                        revision_height: revisionHeightA
                    })
                )
        );
    }

    function test_ibcHeight_gte(
        uint64 revisionNumberA,
        uint64 revisionHeightA,
        uint64 revisionNumberB,
        uint64 revisionHeightB
    ) public {
        vm.assume(revisionNumberB >= revisionNumberA);
        vm.assume(revisionHeightB >= revisionHeightA);
        assertTrue(
            IbcCoreClientV1Height
                .Data({
                    revision_number: revisionNumberB,
                    revision_height: revisionHeightB
                })
                .gte(
                    IbcCoreClientV1Height.Data({
                        revision_number: revisionNumberA,
                        revision_height: revisionHeightA
                    })
                )
        );
    }
}
