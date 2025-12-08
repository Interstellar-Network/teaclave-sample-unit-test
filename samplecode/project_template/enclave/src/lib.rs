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
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate pallet_key_manager;

use sgx_types::*;
use std::io::{self, Write};
use std::slice;

// Import standalone SGX testing utilities
use pallet_key_manager::sgx_utils;

/// Tests SGX sealing functionality for the key-manager pallet.
///
/// This function verifies that the SGX sealing/unsealing logic works correctly:
/// 1. Seal/unseal roundtrip with valid AAD
/// 2. AAD binding (unsealing with wrong AAD should fail)
///
/// If any test fails, the function panics with a descriptive error message,
/// which will cause the enclave call to fail and be caught by CI.
fn test_lib() {
    println!("[SGX Test] Starting key-manager SGX sealing tests...");

    // Test 1: Seal/Unseal Roundtrip
    test_seal_unseal_roundtrip();

    // Test 2: AAD Binding (wrong AAD should fail)
    test_aad_binding();

    println!("[SGX Test] ✓ All key-manager SGX sealing tests passed!");
}

/// Test 1: Verify seal → unseal returns the original seed
fn test_seal_unseal_roundtrip() {
    println!("[SGX Test] Test 1: Seal/Unseal Roundtrip");

    let seed = [42u8; 32];
    let aad = [1u8; 32]; // Mock account ID

    // Seal the seed
    let sealed = sgx_utils::seal_seed(&seed, &aad)
        .expect("Sealing should succeed");

    // Verify sealed data is larger than plaintext (due to MAC and metadata)
    assert!(
        sealed.len() > 32,
        "Sealed data should be larger than plaintext seed"
    );

    // Unseal the seed
    let unsealed = sgx_utils::unseal_seed(&sealed, &aad)
        .expect("Unsealing should succeed");

    // Verify roundtrip
    assert_eq!(
        seed, unsealed,
        "Unsealed seed should match original seed"
    );

    println!("  ✓ Seal/unseal roundtrip verified (sealed size: {} bytes)", sealed.len());
}

/// Test 2: Verify AAD binding prevents unsealing with wrong AAD
fn test_aad_binding() {
    println!("[SGX Test] Test 2: AAD Binding");

    let seed = [123u8; 32];
    let aad1 = [1u8; 32]; // Account 1
    let aad2 = [2u8; 32]; // Account 2

    // Seal with AAD1
    let sealed = sgx_utils::seal_seed(&seed, &aad1)
        .expect("Sealing should succeed");

    // Try to unseal with AAD2 - should fail
    let result = sgx_utils::unseal_seed(&sealed, &aad2);

    assert!(
        result.is_err(),
        "Unsealing with wrong AAD should fail"
    );

    println!("  ✓ AAD mismatch correctly prevented unsealing");

    // Verify unsealing with correct AAD still works
    let unsealed = sgx_utils::unseal_seed(&sealed, &aad1)
        .expect("Unsealing with correct AAD should succeed");

    assert_eq!(seed, unsealed, "Unsealed seed should match original");
    println!("  ✓ Unsealing with correct AAD still works");
}

#[no_mangle]
pub extern "C" fn ecall_test(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // Run SGX sealing tests
    // If any test panics, this will return SGX_ERROR and fail the CI build
    test_lib();

    println!("Message from the enclave");

    sgx_status_t::SGX_SUCCESS
}
