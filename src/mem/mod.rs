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
//! This module implement functions that can be used to control the page locking
//! in memory. This is useful to prevent critical values from being written into
//! the the disk by the virtual memory system.
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub mod impl_default;
#[cfg(target_os = "linux")]
pub mod impl_linux;
#[cfg(target_os = "windows")]
pub mod impl_win32;
#[cfg(test)]
mod tests;

use core::ffi::c_void;
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
use impl_default::*;
#[cfg(target_os = "linux")]
use impl_linux::*;
#[cfg(target_os = "windows")]
use impl_win32::*;
use rand::random;
use std::cmp::min;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use zeroize::Zeroize;

/// Try to lock the memory segment into memory, preventing it from
/// being moved to the disk. All calls to this function must be
/// followed by a call to [`unlock_mem()`].
///
/// Use this method with extreme care because it interferes with tne
/// OS ability to manage virtual memory.
///
/// Arguments:
/// - `ptr`: The pointer to the memory segment;
/// - `size`: The size of the ptr in units;
///
/// Retunrs true on success or false otherwise.
pub fn lock_mem<T: Sized>(ptr: *const T, size: usize) -> bool {
    if size > 0 {
        lock_mem_core(ptr as *const c_void, size * size_of::<T>())
    } else {
        false
    }
}

/// Unlocks the memory segment. It reverts the effects of [`lock_mem()`].
///
/// Arguments:
/// - `ptr`: The pointer to the memory segment;
/// - `size`: The size of the ptr in units;
///
/// Retunrs true on success or false otherwise.
pub fn unlock_mem<T: Sized>(ptr: *const T, size: usize) -> bool {
    if size > 0 {
        unlock_mem_core(ptr as *const c_void, size * size_of::<T>())
    } else {
        false
    }
}

/// Determines if this platform supports memory locking or not.
///
/// Returns true if it is supported or false otherwise.
pub fn lock_supported() -> bool {
    lock_supported_core()
}

//=============================================================================
// SecretBytes
//-----------------------------------------------------------------------------
/// This struct wraps a byte array that is guaranteed to have its contents
/// shredded upon destruction.
///
/// It also allows the locking of the value in memory if required, preventing it
/// from being moved into the disk.
///
/// This struct also implements a mechanism to set a logical length that differs
pub struct SecretBytes {
    value: Vec<u8>,
    locked: bool,
    len: usize,
}

impl SecretBytes {
    /// Creates a new `SecretBytes`.
    ///
    /// Arguments:
    /// - `size`: The size in bytes;
    /// - `locked`: Locks the value in memory;
    pub fn new(size: usize, locked: bool) -> Self {
        let mut ret = Self {
            value: Vec::<u8>::with_capacity(size),
            locked: false,
            len: size,
        };
        ret.value.resize(size, 0);
        if locked {
            ret.lock();
        }
        ret
    }

    /// Creates a new `SecretBytes` and initializes it
    /// with the given value.
    ///
    /// Arguments:
    /// - `value`: The initial value;
    /// - `locked`: Locks the value in memory;
    pub fn with_value(value: &[u8], locked: bool) -> Self {
        let mut ret = Self::new(value.len(), locked);
        ret.value.copy_from_slice(value);
        ret
    }

    /// Returns the value as a mutable byte slice.
    pub fn mut_value(&mut self) -> &mut [u8] {
        &mut self.value.as_mut_slice()[..self.len]
    }

    /// Returns the value as an immutable byte slice.
    pub fn value(&self) -> &[u8] {
        &self.value.as_slice()[..self.len]
    }

    /// Returns the buffer as a mutable byte slice. The buffer may be larger
    /// than the value itself.
    pub fn mut_buffer(&mut self) -> &mut [u8] {
        self.value.as_mut_slice()
    }

    /// Returns the buffer as an immutable byte slice. The buffer may be larger
    /// than the value itself.
    pub fn buffer(&self) -> &[u8] {
        self.value.as_slice()
    }

    /// Returns true if the value is locked in memory or false
    /// otherwise.
    pub fn locked(&self) -> bool {
        self.locked
    }

    /// Returns the logical size of this value. It may be equal
    /// or smaller than the actual buffer size.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Sets the logical size of this value. If the new size is larger
    /// than the buffer size, this method will set the logical size to the
    /// current buffer size.
    ///
    /// Arguments:
    ///
    /// - `size`: The logical size of the value.
    pub fn set_len(&mut self, size: usize) {
        self.len = min(size, self.buffer_len());
    }

    /// Returns true if this value has length 0.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the size of the inner buffer of this value.
    pub fn buffer_len(&self) -> usize {
        self.value.len()
    }

    /// Locks the value in memory, preventing it from being moved
    /// into the disk by the the virtual memory system.
    ///
    /// If this feature is not supported, this function does nothing.
    fn lock(&mut self) {
        if !self.is_empty() && !self.locked {
            self.locked = lock_mem(self.value.as_ptr(), self.value.len());
        }
    }

    /// Unlocks the value in memory.
    ///
    /// This function does nothing if the memory
    fn unlock(&mut self) {
        if self.locked {
            self.locked = !unlock_mem(self.value.as_ptr(), self.value.len());
        }
    }

    /// Verifies if the underlying platform supports memory locking.
    ///
    /// Returns true if locking is supported or false otherwise.
    pub fn lock_supported() -> bool {
        lock_supported()
    }
}

impl Clone for SecretBytes {
    fn clone(&self) -> Self {
        let mut ret = Self::with_value(self.value.as_slice(), self.locked);
        ret.set_len(self.len());
        ret
    }
}

impl Drop for SecretBytes {
    fn drop(&mut self) {
        self.value.as_mut_slice().zeroize();
        self.unlock();
    }
}

impl Deref for SecretBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl DerefMut for SecretBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mut_value()
    }
}

//=============================================================================
// ByteMaskGenerator
//-----------------------------------------------------------------------------
struct ByteMaskGenerator {
    state: u64,
}

impl ByteMaskGenerator {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u8 {
        // This code is partially based on the random implementation by Newlib
        self.state = self.state.wrapping_mul(6364136223846793005) + 1;
        ((self.state >> 32) & 0xFF) as u8
    }
}

//=============================================================================
// ProtectedValue
//-----------------------------------------------------------------------------
/// This trait implements a way to protect secret values stored in memory
/// against potential memory scan techniques. The value is stored in a
/// obfuscated and/or encrypted form that is reversed only when the actual value
/// is needed by the application.
///
/// Although not enough to provide a long term protection, it should be enough
/// to make memory scan techniques way more difficult to perform.
pub trait ProtectedValue: Send + Sync {
    /// Returns the protected value as a [`SecretBytes`] instance.
    fn get_secret(&self) -> SecretBytes;
}

//=============================================================================
// DefaultProtectedValue
//-----------------------------------------------------------------------------
/// This struct implements the the default implementation of the
/// [`ProtectedValue`] trait. It uses a random mask to protect the value stored
/// in memory from simple memory scan attacks.
///
/// It is not the most sophisticated approach to this problem but is guaranteed
/// to work on all platforms.
pub struct DefaultProtectedValue {
    secret: SecretBytes,
    seed: u64,
}

impl DefaultProtectedValue {
    /// Creates a new DefaultProtectedValue with the given value.
    ///
    /// Arguments:
    /// - `value`: The value to be protected;
    pub fn new(value: &[u8]) -> Self {
        let mut secret = SecretBytes::with_value(value, true);
        let mut seed: u64 = 0;
        while seed == 0 {
            seed = random();
        }
        Self::apply_mask(seed, &mut secret);
        Self { secret, seed }
    }

    fn apply_mask(seed: u64, value: &mut [u8]) {
        let mut g = ByteMaskGenerator::new(seed);
        for v in value {
            *v ^= g.next();
        }
    }
}

impl ProtectedValue for DefaultProtectedValue {
    fn get_secret(&self) -> SecretBytes {
        let mut ret = self.secret.clone();
        Self::apply_mask(self.seed, &mut ret);
        ret
    }
}

/// Creates a protected value repository. It always uses the best
/// protection method available to the underlying platform.
///
/// It always returns a [`std::sync::Arc`] of the value because the
/// protection mechanism may be too expensive to create and/or maintain.
/// Furthermore, it is better to keep this kind of secret as isolated as
/// possible inside the memory.
///
/// Returns the protected value.
#[cfg(not(target_os = "windows"))]
pub fn create_protected_value(value: &[u8]) -> Arc<dyn ProtectedValue> {
    Arc::new(DefaultProtectedValue::new(value))
}

/// Creates a protected value repository. It always uses the best
/// protection method available to the underlying platform.
///
/// On Windows platforms, it uses an opaque implementation that relies on
/// `CryptProtectMemory()` and `CryptUnprotectMemory()` to protect the value
/// in memory.
///
/// It always returns a [`std::sync::Arc`] of the value because the
/// protection mechanism may be too expensive to create and/or maintain.
/// Furthermore, it is better to keep this kind of secret as isolated as
/// possible inside the memory.
///
/// Returns the protected value.
#[cfg(target_os = "windows")]
pub fn create_protected_value(value: &[u8]) -> Arc<dyn ProtectedValue> {
    Arc::new(impl_win32::Win32ProtectedValue::new(value))
}
