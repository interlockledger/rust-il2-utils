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
//! This module implements utilities to handle [`std::vec::Vec`]. Most of those
//! utilities are optimized for fast memory manipulations whenever possible.
//!
//! It is important to notice that some methods and functions inside this
//! package rely on unsafe code to achieve maximum performance.
#[cfg(test)]
mod tests;

use zeroize::Zeroize;

//=============================================================================
// VecExtensions
//-----------------------------------------------------------------------------
/// This trait adds some extension methods to [`std::vec::Vec`] for primitive
/// types like integers, floating points and booleans.
///
/// Most of those extensions are designed to be either fast implementations of
/// existing methods or secure versions of them.
///
/// ## Safety
///
/// Some operations performed by this extension relies heavily on pointer
/// operations and bitwise copies (see [`std::ptr`] for further details).
/// Because of that, it is not safe to implement this trait for non primitive
/// types because it may lead to memory safety violations and potential double
/// free situations.
///
/// ## Secure variants
///
/// This extension also include secure variants of some of the vector methods.
/// Those variants always zeroes the memory segments before releasing them back
/// to the memory pool.
///
/// Logically, they do perform the same operations but they are way more
/// expensive than their regular versions. We recommend the use of those
/// versions if and only if you need to avoid potential confidential data
/// leak to the system.
///
/// It is possible that, when the **allocator_api**
/// [#32838](https://github.com/rust-lang/rust/issues/32838) become fully
/// integrated into the standard API, those methods will no longer be
/// necessary as the proper memory cleanup will be done by an
/// [`std::alloc::Allocator`] instead of the hacks used by those methods.
pub trait VecExtensions<T: Copy + Sized>: Zeroize {
    /// Creates a new vector already initialized with the specified value.
    ///
    /// Since it is the first allocation, there is no need to have a secure version
    /// of this constructor.
    ///
    /// Arguments:
    /// - `value`: The initial value of the new Vec instance, the elements of this
    /// slice are copied into the new vector;
    fn with_value(value: &[T]) -> Vec<T>;

    /// This method sets the capacity of the given Vec<u8> to hold at least the
    /// specified amount of entries. It is similar to [`Vec<u8>::reserve()`] but it
    /// takes the target capacity insted of an additional capacity.
    ///
    /// If the current capacity is equal or larger than the required capacity,
    /// this method does nothing.
    ///
    /// Arguments:
    /// - `capacity`: The new capacity;
    fn set_capacity_to(&mut self, capacity: usize);

    /// This method is the secure variant of [`Self::set_capacity_to()`].
    ///
    /// Arguments:
    /// - `capacity`: The new capacity;
    fn set_capacity_to_secure(&mut self, capacity: usize);

    /// Replaces the contents of this vector with the contents of a given
    /// slice. It will expand the size of this vector as needed but will
    /// never shrink it.
    ///
    /// Arguments:
    /// - `other`: The new capacity;
    fn set_contents_from_slice(&mut self, other: &[T]);

    /// This method is the secure variant of [`Self::set_contents_from_slice()`].
    ///
    /// Arguments:
    /// - `other`: The new capacity;
    fn set_contents_from_slice_secure(&mut self, other: &[T]);

    /// This method is the secure version of [`std::vec::Vec::shrink_to_fit()`].
    fn shrink_to_fit_secure(&mut self);

    /// This method is the secure version of [`std::vec::Vec::reserve()`].
    fn reserve_secure(&mut self, additional: usize);

    /// This method is the secure version of [`std::vec::Vec::extend_from_slice()`].
    fn extend_from_slice_secure(&mut self, other: &[T]);
}

macro_rules! vecextention_base_impl {
    ($type: ty) => {
        impl VecExtensions<$type> for Vec<$type> {
            fn with_value(value: &[$type]) -> Vec<$type> {
                let mut obj = Vec::with_capacity(value.len());
                obj.set_contents_from_slice(value);
                obj
            }

            fn set_capacity_to(&mut self, capacity: usize) {
                let curr_capacity = self.capacity();
                if curr_capacity < capacity {
                    self.reserve(capacity - self.len());
                }
            }

            fn set_capacity_to_secure(&mut self, capacity: usize) {
                let curr_capacity = self.capacity();
                if curr_capacity < capacity {
                    if self.is_empty() {
                        // No data to move, just adjust the capacity
                        self.zeroize();
                        self.set_capacity_to(capacity);
                    } else if curr_capacity < capacity {
                        // Copy the values into a temporary buffer before resizing
                        // because it is not possible to ensure that the original
                        // buffer will not be replaced by a larger one. If this happens,
                        // the original data will be released to the memory pool with its
                        // contents intact and this is exactly what we are trying to avoid.
                        let mut tmp: Vec<$type> = Vec::with_capacity(self.len());
                        tmp.set_contents_from_slice(self.as_slice());
                        // Zeroize the original vector before resizing, also set its
                        // size to zero to avoid unecessary copy operation while resizing.
                        self.zeroize();
                        self.truncate(0);
                        // Sets the new capacity
                        self.set_capacity_to(capacity);
                        // Copy the values back into the original vector
                        assert!(self.capacity() >= tmp.len());
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                tmp.as_ptr(),
                                self.as_mut_ptr(),
                                tmp.len(),
                            );
                            self.set_len(tmp.len());
                        }
                        // Clear the temporay copy...
                        tmp.zeroize();
                    }
                }
            }

            fn set_contents_from_slice(&mut self, other: &[$type]) {
                self.set_capacity_to(other.len());
                unsafe {
                    self.set_len(other.len());
                    std::ptr::copy_nonoverlapping(other.as_ptr(), self.as_mut_ptr(), other.len());
                }
            }

            fn set_contents_from_slice_secure(&mut self, other: &[$type]) {
                self.zeroize();
                self.reserve(other.len());
                unsafe {
                    self.set_len(other.len());
                    std::ptr::copy_nonoverlapping(other.as_ptr(), self.as_mut_ptr(), other.len());
                }
            }

            fn shrink_to_fit_secure(&mut self) {
                // Copy to a temporary value
                let mut tmp: Vec<$type> = Vec::with_capacity(self.len());
                tmp.set_contents_from_slice(self.as_slice());
                // Clear the old data and shrink
                self.zeroize();
                self.shrink_to_fit();
                // Copy the contents back into the array.
                self.set_contents_from_slice(tmp.as_slice());
                // Clear the temporary buffer
                tmp.zeroize();
            }

            fn reserve_secure(&mut self, additional: usize) {
                self.set_capacity_to_secure(self.len() + additional);
            }

            fn extend_from_slice_secure(&mut self, other: &[$type]) {
                self.reserve_secure(other.len());
                assert!(self.capacity() >= self.len() + other.len());
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        other.as_ptr(),
                        self.as_mut_ptr().add(self.len()),
                        other.len(),
                    );
                    self.set_len(self.len() + other.len());
                }
            }
        }
    };
}

macro_rules! multi_vecextention_base_impl {
    ($type: ty) => {
        vecextention_base_impl!($type);
    };
    ($type: ty, $($type2: ty), +) => {
        vecextention_base_impl! ($type);
        multi_vecextention_base_impl!($($type2), +);
    };
}

multi_vecextention_base_impl!(bool);
multi_vecextention_base_impl!(u8, u16, u32, u64, u128);
multi_vecextention_base_impl!(i8, i16, i32, i64, i128);
multi_vecextention_base_impl!(f32, f64);
