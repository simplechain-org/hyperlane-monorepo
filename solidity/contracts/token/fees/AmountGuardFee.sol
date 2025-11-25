// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0;

import {ITokenFee, Quote} from "../../interfaces/ITokenBridge.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title AmountGuardFee
 */
contract AmountGuardFee is ITokenFee, Ownable {
    uint256 public minTransferAmount;
    
    event MinTransferAmountSet(uint256 minAmount);
    // 0xa7aaf58f
    error BelowMinimumTransferAmount(uint256 amount, uint256 minAmount);
    
    constructor(uint256 _minAmount) {
        minTransferAmount = _minAmount;
    }
    
    function setMinTransferAmount(uint256 _minAmount) external onlyOwner {
        minTransferAmount = _minAmount;
        emit MinTransferAmountSet(_minAmount);
    }
    
    /**
     * @notice only check amount not charge fee
     */
    function quoteTransferRemote(
        uint32,
        bytes32,
        uint256 _amount
    ) external view returns (Quote[] memory quotes) {
        if (_amount < minTransferAmount) {
            revert BelowMinimumTransferAmount(_amount, minTransferAmount);
        }
        
        quotes = new Quote[](0);
    }
}