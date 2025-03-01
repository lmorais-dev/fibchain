// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

/// @title Fibonacci number validator
/// @notice Basic application that holds a counter of how many fibonacci numbers it validated.
interface IFibonacci {
    /// @notice Increase the global counter on a valid fibonacci number. Requires a RISC Zero proof that the number is part of the fibonacci sequence.
    function increaseCounter(uint128 fibonacciNum, bytes calldata seal) external;

    /// @notice Returns the count (current validated fibonacci numbers) stored.
    function get() external view returns (uint256);
}
