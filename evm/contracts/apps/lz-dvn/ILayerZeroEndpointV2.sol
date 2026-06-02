pragma solidity ^0.8.27;

interface ILayerZeroEndpointV2 {
    function getSendLibrary(
        address _sender,
        uint32 _dstEid
    ) external view returns (address lib);

    function getReceiveLibrary(
        address _receiver,
        uint32 _srcEid
    ) external view returns (address lib, bool isDefault);
}
