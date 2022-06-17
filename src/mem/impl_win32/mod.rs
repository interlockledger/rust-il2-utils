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
//! This module provides the Windows implementation of the functions
//! of [`super`].
#[cfg(test)]
mod tests;

use super::{ProtectedValue, SecretBytes};
use core::ffi::c_void;
use windows::Win32::Security::Cryptography::{
    CryptProtectMemory, CryptUnprotectMemory, CRYPTPROTECTMEMORY_BLOCK_SIZE,
    CRYPTPROTECTMEMORY_SAME_PROCESS,
};
use windows::Win32::System::Memory::{VirtualLock, VirtualUnlock};

#[inline]
pub fn lock_mem_core(ptr: *const c_void, size: usize) -> bool {
    unsafe { VirtualLock(ptr as *mut c_void, size).as_bool() }
}

#[inline]
pub fn unlock_mem_core(ptr: *const c_void, size: usize) -> bool {
    unsafe { VirtualUnlock(ptr as *mut c_void, size).as_bool() }
}

#[inline]
pub fn lock_supported_core() -> bool {
    true
}

//=============================================================================
// Win32ProtectedValue
//-----------------------------------------------------------------------------
/// This is the implementation of the [`ProtectedValue`] for Windows that uses
/// `CryptProtectMemory()` and `CryptUnprotectMemory()` to protect the values
/// against memory scans attacks.
pub struct Win32ProtectedValue {
    protected_data: SecretBytes,
}

impl Win32ProtectedValue {
    /// Returns the size of the buffer required to store the protected value.
    ///
    /// Arguments:
    /// - `data_size`: The size of the value to be protected.
    ///
    /// Returns the size of the buffer required to store the protected value.
    pub fn protected_size(data_size: usize) -> usize {
        let block_size = CRYPTPROTECTMEMORY_BLOCK_SIZE as usize;
        data_size + (block_size - (data_size % block_size))
    }

    /// Creates a new [`Win32ProtectedValue`].
    ///
    /// Arguments:
    /// - `value`: The value to be protected.
    ///
    /// Returns the new instance of  [`Win32ProtectedValue`].
    pub fn new(value: &[u8]) -> Self {
        let data_size = Win32ProtectedValue::protected_size(value.len());
        let mut ret = Self {
            protected_data: SecretBytes::new(data_size, true),
        };
        ret.protected_data.mut_value()[..value.len()].copy_from_slice(value);
        ret.protected_data.set_len(value.len());
        unsafe {
            if !CryptProtectMemory(
                ret.protected_data.mut_buffer().as_mut_ptr() as *mut c_void,
                ret.protected_data.buffer_len() as u32,
                CRYPTPROTECTMEMORY_SAME_PROCESS,
            )
            .as_bool()
            {
                panic!("Unable execute CryptProtectMemory().");
            }
        }
        ret
    }
}

impl ProtectedValue for Win32ProtectedValue {
    fn get_secret(&self) -> SecretBytes {
        let mut ret = self.protected_data.clone();
        unsafe {
            if !CryptUnprotectMemory(
                ret.mut_buffer().as_mut_ptr() as *mut c_void,
                ret.buffer_len() as u32,
                CRYPTPROTECTMEMORY_SAME_PROCESS,
            )
            .as_bool()
            {
                panic!("Unable execute CryptUnprotectMemory().");
            }
        }
        ret
    }
}
