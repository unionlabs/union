pragma solidity ^0.8.27;

import "./Store.sol";

contract UCS03ZkgmStakeImpl is Versioned, UCS03ZkgmStore {
    IIBCModulePacket public immutable IBC_HANDLER;

    constructor(
        IIBCModulePacket _ibcHandler
    ) {
        IBC_HANDLER = _ibcHandler;
    }

    function _stakingSucceeded(
        uint32 channelId,
        Stake calldata _stake
    ) internal {
        ensureStakeIsFromChannel(channelId, _stake.tokenId);
        address beneficiary = address(bytes20(_stake.beneficiary));
        _getStakeNFTManager().transferFrom(
            address(this), beneficiary, _stake.tokenId
        );
        stakes[_stake.tokenId].state = ZkgmStakeState.STAKED;
    }

    function _stakingFailed(uint32 channelId, Stake calldata _stake) internal {
        ensureStakeIsFromChannel(channelId, _stake.tokenId);
        address sender = address(bytes20(_stake.sender));
        (IZkgmERC20 governanceToken,) = _getGovernanceToken(channelId);
        governanceToken.transfer(sender, _stake.amount);
    }

    function _withdrawStakeSucceeded(
        uint32 channelId,
        WithdrawStake calldata _withdrawStake,
        WithdrawStakeAck calldata _withdrawStakeAck
    ) internal {
        ensureStakeIsFromChannel(channelId, _withdrawStake.tokenId);
        ZkgmStake storage _stake = stakes[_withdrawStake.tokenId];
        _stake.state = ZkgmStakeState.UNSTAKED;
        (IZkgmERC20 governanceToken,) = _getGovernanceToken(_stake.channelId);
        address beneficiary = address(bytes20(_withdrawStake.beneficiary));
        governanceToken.transfer(beneficiary, _stake.amount);
        if (_stake.amount < _withdrawStakeAck.amount) {
            // Mints the reward
            governanceToken.mint(
                beneficiary, _withdrawStakeAck.amount - _stake.amount
            );
        } else if (_stake.amount > _withdrawStakeAck.amount) {
            // Burn if slashing happened and rewards aren't covering it.
            governanceToken.burn(
                beneficiary, _stake.amount - _withdrawStakeAck.amount
            );
        }
    }

    function _withdrawRewardsSucceeded(
        uint32 channelId,
        WithdrawRewards calldata _withdrawRewards,
        WithdrawRewardsAck calldata _withdrawRewardsAck
    ) internal {
        ensureStakeIsFromChannel(channelId, _withdrawRewards.tokenId);
        if (_withdrawRewardsAck.amount > 0) {
            ZkgmStake storage _stake = stakes[_withdrawRewards.tokenId];
            (IZkgmERC20 governanceToken,) =
                _getGovernanceToken(_stake.channelId);
            address beneficiary = address(bytes20(_withdrawRewards.beneficiary));
            // Mints the reward
            governanceToken.mint(beneficiary, _withdrawRewardsAck.amount);
        }
        address sender = address(bytes20(_withdrawRewards.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _withdrawRewards.tokenId
        );
    }

    function _setUnstaking(uint256 tokenId, uint256 completionTime) internal {
        ZkgmStake storage _stake = stakes[tokenId];
        _stake.state = ZkgmStakeState.UNSTAKING;
        _stake.unstakingCompletion = completionTime;
    }

    function ensureStakeIsFromChannel(
        uint32 channelId,
        uint256 tokenId
    ) internal {
        if (stakes[tokenId].channelId != channelId) {
            revert ZkgmLib.ErrInvalidStakeChannelId();
        }
    }

    function acknowledgeStake(
        IBCPacket calldata ibcPacket,
        Stake calldata _stake,
        bool successful
    ) public {
        if (successful) {
            _stakingSucceeded(ibcPacket.sourceChannelId, _stake);
        } else {
            _stakingFailed(ibcPacket.sourceChannelId, _stake);
        }
    }

    function acknowledgeUnstake(
        IBCPacket calldata ibcPacket,
        Unstake calldata _unstake,
        bool successful,
        bytes calldata ack
    ) public {
        ensureStakeIsFromChannel(ibcPacket.sourceChannelId, _unstake.tokenId);
        if (successful) {
            UnstakeAck calldata unstakeAck = ZkgmLib.decodeUnstakeAck(ack);
            _setUnstaking(_unstake.tokenId, unstakeAck.completionTime);
        }
        address sender = address(bytes20(_unstake.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _unstake.tokenId
        );
    }

    function acknowledgeWithdrawStake(
        IBCPacket calldata ibcPacket,
        WithdrawStake calldata _withdrawStake,
        bool successful,
        bytes calldata ack
    ) public {
        if (successful) {
            WithdrawStakeAck calldata _withdrawStakeAck =
                ZkgmLib.decodeWithdrawStakeAck(ack);
            _withdrawStakeSucceeded(
                ibcPacket.sourceChannelId, _withdrawStake, _withdrawStakeAck
            );
        } else {
            _withdrawStakeFailed(ibcPacket.sourceChannelId, _withdrawStake);
        }
    }

    function acknowledgeWithdrawRewards(
        IBCPacket calldata ibcPacket,
        WithdrawRewards calldata _withdrawRewards,
        bool successful,
        bytes calldata ack
    ) public {
        if (successful) {
            _withdrawRewardsSucceeded(
                ibcPacket.sourceChannelId,
                _withdrawRewards,
                ZkgmLib.decodeWithdrawRewardsAck(ack)
            );
        } else {
            _withdrawRewardsFailed(ibcPacket.sourceChannelId, _withdrawRewards);
        }
    }

    function _withdrawStakeFailed(
        uint32 channelId,
        WithdrawStake calldata _withdrawStake
    ) internal {
        ensureStakeIsFromChannel(channelId, _withdrawStake.tokenId);
        address sender = address(bytes20(_withdrawStake.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _withdrawStake.tokenId
        );
    }

    function _withdrawRewardsFailed(
        uint32 channelId,
        WithdrawRewards calldata _withdrawRewards
    ) internal {
        ensureStakeIsFromChannel(channelId, _withdrawRewards.tokenId);
        address sender = address(bytes20(_withdrawRewards.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _withdrawRewards.tokenId
        );
    }

    function timeoutStake(
        IBCPacket calldata ibcPacket,
        Stake calldata _stake
    ) public {
        _stakingFailed(ibcPacket.sourceChannelId, _stake);
    }

    function timeoutUnstake(
        IBCPacket calldata ibcPacket,
        Unstake calldata _unstake
    ) public {
        ensureStakeIsFromChannel(ibcPacket.sourceChannelId, _unstake.tokenId);
        address sender = address(bytes20(_unstake.sender));
        _getStakeNFTManager().transferFrom(
            address(this), sender, _unstake.tokenId
        );
    }

    function timeoutWithdrawStake(
        IBCPacket calldata ibcPacket,
        WithdrawStake calldata _withdrawStake
    ) public {
        _withdrawStakeFailed(ibcPacket.sourceChannelId, _withdrawStake);
    }

    function timeoutWithdrawRewards(
        IBCPacket calldata ibcPacket,
        WithdrawRewards calldata _withdrawRewards
    ) public {
        _withdrawRewardsFailed(ibcPacket.sourceChannelId, _withdrawRewards);
    }
}
