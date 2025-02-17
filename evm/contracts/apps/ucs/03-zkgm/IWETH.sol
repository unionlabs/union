import "@openzeppelin/token/ERC20/IERC20.sol";

interface IWETH is IERC20 {
    function deposit() external payable;
    function withdraw(
        uint256 wad
    ) external;
}
