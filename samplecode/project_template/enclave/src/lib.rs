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

use sgx_types::*;
use std::io::{self, Write};
use std::slice;

fn test_lib() {
    println!("Testing extended-recovery pallet in SGX...");
    test_random_generation();
    println!("Extended-recovery pallet tests completed successfully!");
}

#[no_mangle]
pub extern "C" fn ecall_test(some_string: *const u8, some_len: usize) -> sgx_status_t {
    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    test_lib();

    println!("Message from the enclave");

    sgx_status_t::SGX_SUCCESS
}

extern crate pallet_token_extended_recovery;

fn test_random_generation() {
    println!("Testing extended-recovery salt generation...");
        // Test 1: Generate salt and verify length
        let sgx_salt = pallet_token_extended_recovery::sgx_util::generate_salt();
        println!("SGX Salt: {:?}", &sgx_salt[..8]);
        let salt_length_ok = sgx_salt.len() == 16;
        println!("SGX Salt length test (16 bytes): {}", if salt_length_ok { "PASSED" } else { "FAILED" });

        // Test 2: Generate two salts and check they’re different
        let sgx_salt2 = pallet_token_extended_recovery::sgx_util::generate_salt();
        println!("SGX Salt2: {:?}", &sgx_salt2[..8]);
        let salts_different = sgx_salt != sgx_salt2;
        println!("SGX Salt uniqueness test: {}", if salts_different { "PASSED" } else { "FAILED" });

        // Test 3: Generate token and verify outputs
        let skcd_cid = b"test_cid";
        let tx_msg = b"test_message";
        let digits = b"1234";
        match pallet_token_extended_recovery::sgx_util::generate_token(skcd_cid, tx_msg, digits) {
            Ok((token_id, key, salt)) => {
                println!("Token ID (first 8 bytes): {:?}", &token_id[..8]);
                println!("Key (first 8 bytes): {:?}", &key[..8]);
                println!("Salt (first 8 bytes): {:?}", &salt[..8]);
                let token_id_len_ok = token_id.len() == 32; // Blake2_256 produces 32 bytes
                let key_len_ok = key.len() == 32;
                let salt_len_ok = salt.len() == 16;
                println!("Token ID length test (32 bytes): {}", if token_id_len_ok { "PASSED" } else { "FAILED" });
                println!("Key length test (32 bytes): {}", if key_len_ok { "PASSED" } else { "FAILED" });
                println!("Salt length test (16 bytes): {}", if salt_len_ok { "PASSED" } else { "FAILED" });
            }
            Err(()) => println!("SGX Token generation test: FAILED (generation error)"),
        }
}