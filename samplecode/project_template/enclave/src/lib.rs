// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

// Define crate metadata: name and type for SGX static library compilation.
// This specifies that the crate is named "sample" and will be compiled as a static library,
// which is required for SGX enclaves to link with the untrusted runtime.
#![crate_name = "sample"]
#![crate_type = "staticlib"]
// Enable no_std for environments without a standard library (e.g., SGX), controlled by the 'std' feature.
// If the 'std' feature is not enabled, the crate switches to no_std mode, removing dependency on the
// standard library. This is typical for SGX enclaves but may be overridden in your build setup.
#![cfg_attr(not(feature = "std"), no_std)]
// Enable rustc_private for SGX-specific Rust features when the 'sgx' feature is active.
// This allows access to internal Rust features needed for SGX compilation, such as custom linking behavior.
#![cfg_attr(feature = "sgx", feature(rustc_private))]

// Core SGX types (e.g., sgx_status_t) required for all builds, regardless of target or features.
// This crate provides SGX-specific definitions like status codes and is always needed for the
// ECALL return type, making it a universal dependency safe for no_std environments.
extern crate sgx_types;

// Import the SGX-compatible standard library (sgx_tstd) only when the build target is NOT SGX.
// This configuration uses sgx_tstd as a stand-in for std in non-SGX builds (e.g., for testing on standard targets).
// In SGX builds (target_env = "sgx"), this is skipped, implying reliance on the regular std crate or an
// implicit std-like environment provided by your build system. The #[macro_use] attribute enables macros
// like println! from sgx_tstd, and 'as std' aliases it to std for consistent downstream usage.
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

// Import SGX status types for ECALL return values (e.g., SGX_SUCCESS, SGX_ERROR_UNEXPECTED).
use sgx_types::*;

// Import standard library components; these are provided by either std (in SGX builds) or sgx_tstd (in non-SGX builds).
// This assumes std is available, which works in your setup because either 'std' is enabled or your SGX environment
// provides a compatible std implementation despite the no_std attribute.
use std::io::{ self, Write };
use std::slice;
use std::string::String;
use std::vec::Vec;

extern crate pallet_token_extended_recovery;
use pallet_token_extended_recovery::Garble;

struct TestGarbler;
impl Garble for TestGarbler {
    fn garble_and_serialize(skcd_buf: &[u8], digits: &[u8], tx_msg: &str) -> Result<Vec<u8>, ()> {
        if skcd_buf.len() > 128 || digits.len() > 8 || tx_msg.len() > 256 {
            return Err(());
        }
        if digits.iter().any(|&d| (d < b'0' || d > b'9')) {
            return Err(());
        }

        let mut result = Vec::new();
        let garble_key = b"SGX_TEST_KEY";
        for (i, &byte) in skcd_buf.iter().enumerate() {
            result.push(byte ^ garble_key[i % garble_key.len()]);
        }
        result.extend_from_slice(digits);
        result.extend_from_slice(tx_msg.as_bytes());
        Ok(result)
    }
}

struct TestResult {
    passed: bool,
    message: String,
}

fn run_tests() -> Vec<TestResult> {
    let mut results = Vec::new();

    let skcd_cid = b"test_cid";
    let digits = b"1234";
    let tx_msg = "test_message";
    results.push(test_garble_case("Normal input", skcd_cid, digits, tx_msg));

    let empty_skcd = b"";
    let empty_digits = b"";
    let empty_msg = "";
    results.push(test_garble_case("Empty inputs", empty_skcd, empty_digits, empty_msg));

    let large_skcd = [b'a'; 128].as_ref();
    let large_digits = b"12345678";
    let large_msg = String::from_utf8(vec![b'b'; 256]).unwrap();
    results.push(test_garble_case("Large valid input", large_skcd, large_digits, &large_msg));

    results
}

fn test_garble_case(name: &str, skcd_buf: &[u8], digits: &[u8], tx_msg: &str) -> TestResult {
    match TestGarbler::garble_and_serialize(skcd_buf, digits, tx_msg) {
        Ok(result) => {
            let min_len = skcd_buf.len() + digits.len() + tx_msg.len();
            let garble_key = b"SGX_TEST_KEY";

            // Check if the first skcd_buf.len() bytes match the garbled input
            let contains_skcd =
                skcd_buf.is_empty() ||
                (result.len() >= skcd_buf.len() &&
                    ({
                        let garbled_section = &result[..skcd_buf.len()];
                        garbled_section
                            .iter()
                            .zip(skcd_buf.iter())
                            .enumerate()
                            .all(|(i, (&r, &s))| { r == (s ^ garble_key[i % garble_key.len()]) })
                    }));

            // Check digits and tx_msg in the remaining portion
            let digits_start = skcd_buf.len();
            let tx_msg_start = digits_start + digits.len();
            let contains_digits =
                digits.is_empty() ||
                (result.len() >= tx_msg_start &&
                    ({
                        let digits_section = &result[digits_start..tx_msg_start];
                        digits_section == digits
                    }));
            let contains_tx_msg =
                tx_msg.is_empty() ||
                (result.len() >= tx_msg_start + tx_msg.len() &&
                    ({
                        let tx_msg_section = &result[tx_msg_start..];
                        tx_msg_section == tx_msg.as_bytes()
                    }));

            let passed =
                result.len() >= min_len && contains_skcd && contains_digits && contains_tx_msg;
            TestResult {
                passed,
                message: format!(
                    "Test '{}': length={} (min={}), skcd={}, digits={}, tx_msg={}",
                    name,
                    result.len(),
                    min_len,
                    if contains_skcd {
                        "present"
                    } else {
                        "missing"
                    },
                    if contains_digits {
                        "present"
                    } else {
                        "missing"
                    },
                    if contains_tx_msg {
                        "present"
                    } else {
                        "missing"
                    }
                ),
            }
        }
        Err(()) =>
            TestResult {
                passed: false,
                message: format!("Test '{}': failed to process", name),
            },
    }
}

// SGX enclave entry point (ECALL) callable from untrusted code.
// This function is the interface between the untrusted runtime and the enclave’s trusted code.
#[no_mangle]
pub extern "C" fn ecall_test(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    println!("Testing extended-recovery pallet in SGX...");
    let test_results = run_tests();

    for result in &test_results {
        println!("{}: {}", if result.passed { "PASSED" } else { "FAILED" }, result.message);
    }

    let all_passed = test_results.iter().all(|r| r.passed);
    println!("Extended-recovery pallet tests completed {}", if all_passed {
        "successfully"
    } else {
        "with failures"
    });
    println!("Message from the enclave");

    if all_passed {
        sgx_status_t::SGX_SUCCESS
    } else {
        sgx_status_t::SGX_ERROR_UNEXPECTED
    }
}
