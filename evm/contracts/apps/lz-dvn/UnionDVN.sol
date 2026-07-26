pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";
import "@openzeppelin-upgradeable/contracts/utils/PausableUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/utils/ReentrancyGuardUpgradeable.sol";

import "../Base.sol";
import "../../core/04-channel/IIBCPacket.sol";
import "../../internal/Versioned.sol";

import "./ILayerZeroDVN.sol";
import "./IReceiveUlnE2.sol";
import "./ILayerZeroEndpointV2.sol";
import "./PacketV1Codec.sol";

/// @notice LayerZero V2 DVN that carries verification attestations over
/// Union's IBC-based general message passing.
///
/// Topology
/// --------
/// The same `UnionDVN` contract is deployed on every EVM chain in the set.
/// A pair of Union IBC channels is opened between each chain pair, with the
/// `UnionDVN` contract bound as the channel's module on both ends.
///
/// Source-chain flow
/// -----------------
/// 1. An OApp calls `ILayerZeroEndpointV2.send(...)`.
/// 2. The SendLibrary (configured in the OApp's UlnConfig) invokes this
///    contract's `assignJob(param, options)` with `msg.value = getFee(...)`.
/// 3. We require `msg.sender` to be a trusted SendLibrary (admin-configured).
/// 4. We look up the Union channel for `param.dstEid`, ABI-encode the
///    `(packetHeader, payloadHash, confirmations)` tuple, and forward it
///    through `IIBCPacket.sendPacket(...)`.
///
/// Destination-chain flow
/// ----------------------
/// 1. The Union relayer delivers the packet; the IBC handler calls
///    `onRecvPacket` on this contract.
/// 2. We require the source channel to be trusted (admin-configured).
/// 3. We decode the job, parse the packet header for `(srcEid, receiver)`,
///    query the ReceiveLibrary via `ILayerZeroEndpointV2.getReceiveLibrary`,
///    and invoke `IReceiveUlnE2.verify(packetHeader, payloadHash, confirmations)`.
///    `msg.sender` to the ReceiveLibrary is this contract's address, which
///    must be listed in the OApp's UlnConfig as one of its DVNs for
///    verification to count.
///
/// Security model
/// --------------
/// - A trusted SendLibrary set (admin) is required to authorize `assignJob`.
///   Without this, a malicious OApp could configure a rogue SendLibrary that
///   fabricates packets and impersonates `sender` to trick peer OApps.
/// - A trusted source-channel set (admin) is required to authorize
///   `onRecvPacket`. An open Union IBC channel whose counterparty we have not
///   vetted is not safe to accept verifications from.
/// - Union's own consensus-light-client proof is what gives each message
///   integrity; this contract only gates which source channels count.
contract UnionDVN is
    IBCAppBase,
    Initializable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable,
    ReentrancyGuardUpgradeable,
    ILayerZeroDVN,
    Versioned
{
    /// @dev Returned by `onRecvPacket` on success.
    bytes1 public constant ACK_SUCCESS = 0x01;

    struct DstConfig {
        uint32 unionChannelId;
        uint128 baseFee;
        uint128 feePerConfirmation;
        bool enabled;
    }

    IIBCPacket public ibcHandler;
    ILayerZeroEndpointV2 public lzEndpoint;
    uint64 public defaultTimeoutNanos;

    /// @dev LayerZero destination EID -> Union channel + fee config.
    mapping(uint32 => DstConfig) public dstConfig;

    /// @dev Whitelist of LayerZero SendLibraries allowed to call `assignJob`.
    mapping(address => bool) public trustedSendLibraries;

    /// @dev Whitelist of Union destination channels allowed to inject
    /// verifications on receive. The `channelId` here is this chain's
    /// `packet.destinationChannelId`.
    mapping(uint32 => bool) public trustedSourceChannels;

    event DstConfigSet(
        uint32 indexed dstEid,
        uint32 unionChannelId,
        uint128 baseFee,
        uint128 feePerConfirmation,
        bool enabled
    );
    event TrustedSendLibrarySet(address indexed sendLibrary, bool trusted);
    event TrustedSourceChannelSet(uint32 indexed channelId, bool trusted);
    event DefaultTimeoutSet(uint64 timeoutNanos);
    event FeesWithdrawn(address indexed to, uint256 amount);

    event JobAssigned(
        uint32 indexed dstEid,
        uint32 indexed unionChannelId,
        address indexed sender,
        bytes32 headerHash,
        bytes32 payloadHash,
        uint64 confirmations,
        uint256 fee
    );
    event VerificationDelivered(
        uint32 indexed srcEid,
        address indexed receiver,
        address indexed receiveLibrary,
        bytes32 headerHash,
        bytes32 payloadHash,
        uint64 confirmations
    );
    event VerificationTimedOut(
        uint32 indexed sourceChannelId, bytes32 indexed packetHash
    );

    error ErrUntrustedSendLibrary();
    error ErrUntrustedSourceChannel();
    error ErrDstNotEnabled();
    error ErrEidMismatch();
    error ErrInsufficientFee();
    error ErrTimeoutNotSet();
    error ErrWithdrawFailed();
    error ErrReceiveLibraryNotFound();
    error ErrZeroAddress();

    constructor() {
        _disableInitializers();
    }

    function initialize(
        IIBCPacket _ibcHandler,
        ILayerZeroEndpointV2 _lzEndpoint,
        address _authority,
        uint64 _defaultTimeoutNanos
    ) public initializer {
        if (address(_ibcHandler) == address(0)) revert ErrZeroAddress();
        if (address(_lzEndpoint) == address(0)) revert ErrZeroAddress();
        if (_defaultTimeoutNanos == 0) revert ErrTimeoutNotSet();
        __UUPSUpgradeable_init();
        __AccessManaged_init(_authority);
        __Pausable_init();
        __ReentrancyGuard_init();
        ibcHandler = _ibcHandler;
        lzEndpoint = _lzEndpoint;
        defaultTimeoutNanos = _defaultTimeoutNanos;
    }

    /// @inheritdoc IBCAppBase
    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    // -------------------------------------------------------------------
    // Admin
    // -------------------------------------------------------------------

    function setDstConfig(
        uint32 dstEid,
        uint32 unionChannelId,
        uint128 baseFee,
        uint128 feePerConfirmation,
        bool enabled
    ) external restricted {
        dstConfig[dstEid] = DstConfig({
            unionChannelId: unionChannelId,
            baseFee: baseFee,
            feePerConfirmation: feePerConfirmation,
            enabled: enabled
        });
        emit DstConfigSet(
            dstEid, unionChannelId, baseFee, feePerConfirmation, enabled
        );
    }

    function setTrustedSendLibrary(
        address sendLibrary,
        bool trusted
    ) external restricted {
        trustedSendLibraries[sendLibrary] = trusted;
        emit TrustedSendLibrarySet(sendLibrary, trusted);
    }

    function setTrustedSourceChannel(
        uint32 channelId,
        bool trusted
    ) external restricted {
        trustedSourceChannels[channelId] = trusted;
        emit TrustedSourceChannelSet(channelId, trusted);
    }

    function setDefaultTimeout(
        uint64 timeoutNanos
    ) external restricted {
        if (timeoutNanos == 0) revert ErrTimeoutNotSet();
        defaultTimeoutNanos = timeoutNanos;
        emit DefaultTimeoutSet(timeoutNanos);
    }

    function withdrawFees(
        address payable to,
        uint256 amount
    ) external restricted {
        if (to == address(0)) revert ErrZeroAddress();
        (bool ok,) = to.call{value: amount}("");
        if (!ok) revert ErrWithdrawFailed();
        emit FeesWithdrawn(to, amount);
    }

    function pause() external restricted {
        _pause();
    }

    function unpause() external restricted {
        _unpause();
    }

    // -------------------------------------------------------------------
    // ILayerZeroDVN
    // -------------------------------------------------------------------

    function getFee(
        uint32 _dstEid,
        uint64 _confirmations,
        address, /* _sender */
        bytes calldata /* _options */
    ) public view override returns (uint256 fee) {
        DstConfig memory cfg = dstConfig[_dstEid];
        if (!cfg.enabled) revert ErrDstNotEnabled();
        fee =
            uint256(cfg.baseFee) + uint256(cfg.feePerConfirmation) * _confirmations;
    }

    function assignJob(
        AssignJobParam calldata _param,
        bytes calldata _options
    )
        external
        payable
        override
        whenNotPaused
        nonReentrant
        returns (uint256 fee)
    {
        if (!trustedSendLibraries[msg.sender]) {
            revert ErrUntrustedSendLibrary();
        }

        DstConfig memory cfg = dstConfig[_param.dstEid];
        if (!cfg.enabled) revert ErrDstNotEnabled();

        // Cross-check the packet header's declared destination against the
        // dstEid parameter to reject mismatched jobs early.
        bytes memory header = _param.packetHeader;
        PacketV1Codec.assertValid(header);
        if (PacketV1Codec.dstEid(header) != _param.dstEid) {
            revert ErrEidMismatch();
        }

        fee = uint256(cfg.baseFee)
            + uint256(cfg.feePerConfirmation) * _param.confirmations;
        if (msg.value < fee) revert ErrInsufficientFee();

        uint64 timeoutTimestamp =
            uint64(block.timestamp * 1e9) + defaultTimeoutNanos;

        bytes memory payload = abi.encode(
            _param.packetHeader, _param.payloadHash, _param.confirmations
        );

        ibcHandler.sendPacket(
            cfg.unionChannelId,
            0, // deprecated timeoutHeight
            timeoutTimestamp,
            payload
        );

        emit JobAssigned(
            _param.dstEid,
            cfg.unionChannelId,
            _param.sender,
            keccak256(header),
            _param.payloadHash,
            _param.confirmations,
            fee
        );

        // Silence unused variable warning; options are passed through
        // untouched and reserved for forward compatibility.
        _options;
    }

    // -------------------------------------------------------------------
    // IIBCModule
    // -------------------------------------------------------------------

    function onRecvPacket(
        address, /* caller */
        IBCPacket calldata packet,
        address, /* relayer */
        bytes calldata /* relayerMsg */
    )
        external
        virtual
        override
        onlyIBC
        whenNotPaused
        nonReentrant
        returns (bytes memory acknowledgement)
    {
        if (!trustedSourceChannels[packet.destinationChannelId]) {
            revert ErrUntrustedSourceChannel();
        }

        (bytes memory packetHeader, bytes32 payloadHash, uint64 confirmations) =
            abi.decode(packet.data, (bytes, bytes32, uint64));

        PacketV1Codec.assertValid(packetHeader);
        uint32 srcEid = PacketV1Codec.srcEid(packetHeader);
        address receiver = PacketV1Codec.receiverAddress(packetHeader);

        (address receiveLibrary,) =
            lzEndpoint.getReceiveLibrary(receiver, srcEid);
        if (receiveLibrary == address(0)) {
            revert ErrReceiveLibraryNotFound();
        }

        IReceiveUlnE2(receiveLibrary).verify(
            packetHeader, payloadHash, confirmations
        );

        emit VerificationDelivered(
            srcEid,
            receiver,
            receiveLibrary,
            keccak256(packetHeader),
            payloadHash,
            confirmations
        );

        return abi.encodePacked(ACK_SUCCESS);
    }

    function onAcknowledgementPacket(
        address, /* caller */
        IBCPacket calldata, /* packet */
        bytes calldata, /* acknowledgement */
        address /* relayer */
    ) external virtual override onlyIBC {
        // Verification flow is one-way; the ACK carries no actionable state.
    }

    function onTimeoutPacket(
        address, /* caller */
        IBCPacket calldata packet,
        address /* relayer */
    ) external virtual override onlyIBC {
        emit VerificationTimedOut(
            packet.sourceChannelId, keccak256(packet.data)
        );
    }

    // -------------------------------------------------------------------
    // Upgrade
    // -------------------------------------------------------------------

    function _authorizeUpgrade(
        address newImplementation
    ) internal override restricted {}

    /// @dev Accept direct ETH transfers so admins can top up fee balance
    /// for testing if needed.
    receive() external payable {}
}
