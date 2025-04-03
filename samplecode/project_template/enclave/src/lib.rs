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

#![crate_name = "sample"]
#![crate_type = "staticlib"]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate pallet_token_extended_recovery;

use sgx_types::*;
use std::io::{ self, Write };
use std::slice;
use std::string::String;
use std::vec::Vec;

// Fetch PROXY_PALLET_ID and randomness
use pallet_token_extended_recovery::{ PROXY_PALLET_ID, randomness, SgxRandomness };

#[cfg(feature = "sgx")]
use sgx_trts::trts::rsgx_read_rand;

struct TestResult {
    passed: bool,
    message: String,
}

fn run_tests() -> Vec<TestResult> {
    let mut results = Vec::new();
    // Verify PROXY_PALLET_ID bytes are accessible
    let proxy_bytes = PROXY_PALLET_ID.0;
    let proxy_bytes_test = TestResult {
        passed: proxy_bytes == *b"TokenPrx",
        message: String::from("PROXY_PALLET_ID bytes match 'TokenPrx'"),
    };
    results.push(proxy_bytes_test);

    results
}

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
