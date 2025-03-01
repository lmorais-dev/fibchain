// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {ImageID} from "./ImageID.sol";

contract Fibonacci {
    /// @notice RISC Zero verifier contract address.
    IRiscZeroVerifier public immutable verifier;

    /// @notice Image ID of the zkVM guest program that will do verifications.
    bytes32 public constant imageId = ImageID.FIBONACCI_ID;

    /// @notice Counter of how many times a fibonacci number was successfully validated.
    uint256 public counter;

    constructor(IRiscZeroVerifier _verifier) {
        verifier = _verifier;
        counter = 0;
    }

    function increaseCounter(uint128 fibonacciNum, bytes calldata seal) public {
        bytes memory journal = abi.encode(fibonacciNum);
        verifier.verify(seal, imageId, sha256(journal));
        counter = counter + 1;
    }

    function get() public view returns (uint256) {
        return counter;
    }
}
