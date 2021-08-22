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

//=============================================================================
// VecExtensions
//-----------------------------------------------------------------------------
#[test]
fn test_vecextensions_with_value() {
    for i in 0..32 {
        let sample: [u8; 32] = [i as u8; 32];
        let v: Vec<u8> = Vec::with_value(&sample);
        assert_eq!(v.as_slice(), &sample);
    }
}

#[test]
fn test_vecextensions_set_capacity_to() {
    let mut v = Vec::<u8>::new();

    v.set_capacity_to(10);
    assert_eq!(v.len(), 0);
    assert!(v.capacity() >= 10);

    v.set_capacity_to(100);
    assert_eq!(v.len(), 0);
    assert!(v.capacity() >= 100);

    let sample: [u8; 4] = [1, 2, 3, 4];
    let mut v = Vec::<u8>::new();
    v.extend_from_slice(&sample);

    v.set_capacity_to(10);
    assert_eq!(v.as_slice(), &sample);
    assert!(v.capacity() >= 10);

    v.set_capacity_to(100);
    assert_eq!(v.as_slice(), &sample);
    assert!(v.capacity() >= 100);
}

#[test]
fn test_vecextensions_set_capacity_to_secure() {
    let mut v = Vec::<u8>::new();

    v.set_capacity_to_secure(10);
    assert_eq!(v.len(), 0);
    assert!(v.capacity() >= 10);

    v.set_capacity_to_secure(100);
    assert_eq!(v.len(), 0);
    assert!(v.capacity() >= 100);

    let sample: [u8; 4] = [1, 2, 3, 4];
    let mut v = Vec::<u8>::new();
    v.extend_from_slice(&sample);

    v.set_capacity_to_secure(10);
    assert_eq!(v.as_slice(), &sample);
    assert!(v.capacity() >= 10);

    v.set_capacity_to_secure(100);
    assert_eq!(v.as_slice(), &sample);
    assert!(v.capacity() >= 100);
}

#[test]
fn test_vecextensions_set_contents_from_slice() {
    let sample: [u8; 32] = [0xFA; 32];

    let mut v = Vec::<u8>::new();
    let old_capacity = v.capacity();
    v.set_contents_from_slice(&sample[0..0]);
    assert!(v.is_empty());
    assert_eq!(v.capacity(), old_capacity);

    let mut v = Vec::<u8>::new();
    let old_capacity = v.capacity();
    v.set_contents_from_slice(&sample);
    assert_eq!(v.as_slice(), &sample);
    assert!(old_capacity < v.capacity());

    let old_capacity = v.capacity();
    let sample: [u8; 16] = [0xBA; 16];
    v.set_contents_from_slice(&sample);
    assert_eq!(v.as_slice(), &sample);
    assert_eq!(v.capacity(), old_capacity);
}

#[test]
fn test_vecextensions_set_contents_from_slice_secure() {
    let sample: [u8; 32] = [0xFA; 32];

    let mut v = Vec::<u8>::new();
    let old_capacity = v.capacity();
    v.set_contents_from_slice_secure(&sample[0..0]);
    assert!(v.is_empty());
    assert_eq!(v.capacity(), old_capacity);

    let mut v = Vec::<u8>::new();
    let old_capacity = v.capacity();
    v.set_contents_from_slice_secure(&sample);
    assert_eq!(v.as_slice(), &sample);
    assert!(old_capacity < v.capacity());

    let old_capacity = v.capacity();
    let sample: [u8; 16] = [0xBA; 16];
    v.set_contents_from_slice_secure(&sample);
    assert_eq!(v.as_slice(), &sample);
    assert_eq!(v.capacity(), old_capacity);
}

#[test]
fn test_vecextensions_shrink_to_fit_secure() {
    let sample: [u8; 32] = [0xFA; 32];

    let mut v = Vec::<u8>::with_capacity(128);
    let old_capacity = v.capacity();
    v.shrink_to_fit_secure();
    assert!(v.capacity() < old_capacity);

    let mut v = Vec::<u8>::with_capacity(128);
    let old_capacity = v.capacity();
    v.set_contents_from_slice(&sample);
    v.shrink_to_fit_secure();
    assert!(v.capacity() < old_capacity);
    assert_eq!(v.as_slice(), &sample);
}

#[test]
fn test_vecextensions_reserve_secure() {
    let sample: [u8; 32] = [0xFA; 32];

    let mut v = Vec::<u8>::new();
    let old_capacity = v.capacity();
    v.reserve_secure(10);
    assert!(v.capacity() > old_capacity);

    let mut v = Vec::<u8>::new();
    v.set_contents_from_slice(&sample);
    let old_capacity = v.capacity();
    v.reserve_secure(128);
    assert!(v.capacity() > old_capacity);
    assert_eq!(v.as_slice(), &sample);
}

#[test]
fn test_vecextensions_extend_from_slice_secure() {
    let mut v = Vec::<u8>::new();
    let mut exp = Vec::<u8>::new();

    for i in 0..32 {
        let sample: [u8; 32] = [i as u8; 32];
        v.extend_from_slice_secure(&sample);
        exp.extend_from_slice(&sample);
        assert_eq!(v.as_slice(), exp.as_slice());
    }
}
