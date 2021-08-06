/*
 * BSD 3-Clause License
 *
 * Copyright (c) 2019-2020, InterlockLedger Network
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright notice, this
 *   list of conditions and the following disclaimer.
 *
 * * Redistributions in binary form must reproduce the above copyright notice,
 *   this list of conditions and the following disclaimer in the documentation
 *   and/or other materials provided with the distribution.
 *
 * * Neither the name of the copyright holder nor the names of its
 *   contributors may be used to endorse or promote products derived from
 *   this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
use super::*;

#[test]
fn test_lock_unlock_mem_core() {
    let mut v: Vec<u8> = Vec::with_capacity(16);
    v.resize(16, 0);
    assert!(lock_mem_core(v.as_ptr() as *const c_void, v.len()));
    assert!(unlock_mem_core(v.as_ptr() as *const c_void, v.len()));
}

#[test]
fn test_lock_supported_core() {
    assert!(lock_supported_core());
}

//=============================================================================
// Win32ProtectedValue
//-----------------------------------------------------------------------------
#[test]
fn test_win32protectedvalue_protected_size() {
    let block_size = CRYPTPROTECTMEMORY_BLOCK_SIZE as usize;
    for c in 0..8 {
        for i in (block_size * c)..(block_size * (c + 1)) {
            assert_eq!(Win32ProtectedValue::protected_size(i), block_size * (c + 1));
        }
    }
}

#[test]
fn test_win32protectedvalue_new() {
    let exp: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let p = Win32ProtectedValue::new(&exp);

    assert_eq!(p.protected_data.len(), exp.len());
    assert_eq!(
        p.protected_data.buffer_len(),
        Win32ProtectedValue::protected_size(exp.len())
    );
    assert_ne!(p.protected_data.value(), &exp);
}

#[test]
fn test_win32protectedvalue_get_secret() {
    let exp: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let p = Win32ProtectedValue::new(&exp);

    let s = p.get_secret();
    assert_ne!(p.protected_data.value(), &exp);
    assert_eq!(s.value(), &exp);
}
