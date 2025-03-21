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

// TODO? Ideally we want to run some basic tests, but it would require more work:
// - AT LEAST: add some missing "import" in Enclave.edl
// - resolve "undefined reference" errors for each of those
// - FIX runtime error: [-] ECALL Enclave Failed SGX_ERROR_STACK_OVERRUN!
fn test_lib() {
    // This WOULD FAIL, cf docstring if this fn
    // let response = http_grpc_client::sp_offchain_fetch_from_remote_grpc_web(
    //     None,
    //     "https://www.google.com",
    //     &http_grpc_client::RequestMethod::Get,
    //     None,
    //     core::time::Duration::from_millis(1000),
    // )
    // .unwrap();

    // let response = http_grpc_client::http_req_fetch_from_remote_grpc_web(
    //     None,
    //     "http://postman-echo.com/get?hello=world",
    //     &http_grpc_client::RequestMethod::Get,
    //     None,
    //     core::time::Duration::from_millis(1000),
    // )
    // .unwrap();
    println!("Testing extended-recovery pallet in SGX...");
    // Test random number generation
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

extern crate rand;
extern crate rand_chacha;
    

fn test_random_generation() {
    println!("Testing random number generation in SGX...");
    
    use rand::Rng;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};
    
    let mut rng = ChaChaRng::from_entropy();
    let mut salt1 = [0u8; 32];
    rng.fill(&mut salt1);
    0
    let mut rng2 = ChaChaRng::from_entropy();
    let mut salt2 = [0u8; 32];
    rng2.fill(&mut salt2);
    
    println!("Salt1 first 8 bytes: {:?}", &salt1[0..8]);
    println!("Salt2 first 8 bytes: {:?}", &salt2[0..8]);
    
    let different = salt1 != salt2;
    println!("Random generation test: {}", if different { "PASSED" } else { "FAILED" });
}