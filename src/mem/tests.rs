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

#[cfg(target_os = "linux")]
#[test]
fn test_lock_unlock_mem_core() {
    let mut vec = Vec::<u8>::with_capacity(16);
    vec.resize(16, 0);

    assert!(lock_mem_core(vec.as_ptr() as *const c_void, vec.len()));
    assert!(unlock_mem_core(vec.as_ptr() as *const c_void, vec.len()));
}

#[cfg(target_os = "linux")]
#[test]
fn test_lock_unlock_mem() {
    let mut vec = Vec::<u8>::with_capacity(16);
    vec.resize(16, 0);

    assert!(lock_mem(vec.as_ptr(), vec.len()));
    assert!(unlock_mem(vec.as_ptr(), vec.len()));

    assert!(!lock_mem(vec.as_ptr(), 0));
    assert!(!unlock_mem(vec.as_ptr(), 0));
}

#[cfg(target_os = "linux")]
#[test]
fn test_lock_supported() {
    assert!(lock_supported());
}

//=============================================================================
// SecretBytes
//-----------------------------------------------------------------------------
#[test]
fn test_secret_bytes_new() {
    // Locked
    let s = SecretBytes::new(16, true);
    let exp: [u8; 16] = [0; 16];
    assert_eq!(s.len(), s.len());
    assert_eq!(s.buffer_len(), s.len());
    assert_eq!(s.value(), &exp);
    if SecretBytes::lock_supported() {
        assert!(s.locked());
    } else {
        assert!(!s.locked());
    }

    // Unlocked
    let s = SecretBytes::new(16, false);
    let exp: [u8; 16] = [0; 16];
    assert_eq!(s.len(), s.len());
    assert_eq!(s.buffer_len(), s.len());
    assert_eq!(s.value(), &exp);
    assert!(!s.locked());
}

#[test]
fn test_secret_bytes_with_value() {
    let exp: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

    // Locked
    let s = SecretBytes::with_value(&exp, true);
    assert_eq!(s.len(), s.len());
    assert_eq!(s.buffer_len(), s.len());
    assert_eq!(s.value(), &exp);
    if SecretBytes::lock_supported() {
        assert!(s.locked());
    } else {
        assert!(!s.locked());
    }

    // Unlocked
    let s = SecretBytes::with_value(&exp, false);
    assert_eq!(s.len(), s.len());
    assert_eq!(s.buffer_len(), s.len());
    assert_eq!(s.value(), &exp);
    assert!(!s.locked());
}

#[test]
fn test_secret_bytes_len() {
    let exp: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut s = SecretBytes::with_value(&exp, false);

    assert_eq!(s.len(), exp.len());
    assert_eq!(s.buffer_len(), exp.len());
    assert_eq!(s.value(), &exp);
    assert_eq!(s.mut_value(), &exp);
    assert_eq!(s.buffer(), &exp);
    assert_eq!(s.mut_buffer(), &exp);
    assert_eq!(s.value().as_ptr(), s.buffer().as_ptr());

    s.set_len(4);
    assert_eq!(s.len(), 4);
    assert_eq!(s.buffer_len(), exp.len());
    assert_eq!(s.value(), &exp[..4]);
    assert_eq!(s.mut_value(), &exp[..4]);
    assert_eq!(s.mut_buffer(), &exp);
    assert_eq!(s.value().as_ptr(), s.buffer().as_ptr());

    s.set_len(9);
    assert_eq!(s.len(), exp.len());
    assert_eq!(s.buffer_len(), exp.len());
    assert_eq!(s.value(), &exp);
    assert_eq!(s.mut_value(), &exp);
    assert_eq!(s.mut_buffer(), &exp);
    assert_eq!(s.value().as_ptr(), s.buffer().as_ptr());
}

#[test]
fn test_secret_bytes_clone() {
    let exp: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    // Locked
    let mut src = SecretBytes::with_value(&exp, true);
    let s = src.clone();
    assert_eq!(s.len(), s.len());
    assert_eq!(s.buffer_len(), s.len());
    assert_eq!(s.value(), &exp);
    if SecretBytes::lock_supported() {
        assert!(s.locked());
    } else {
        assert!(!s.locked());
    }

    // Locked alt len
    src.set_len(6);
    let s = src.clone();
    assert_eq!(s.len(), 6);
    assert_eq!(s.buffer_len(), exp.len());
    assert_eq!(s.value(), &exp[..6]);
    if SecretBytes::lock_supported() {
        assert!(s.locked());
    } else {
        assert!(!s.locked());
    }

    // Unlocked
    let mut src = SecretBytes::with_value(&exp, false);
    let s = src.clone();
    assert_eq!(s.len(), s.len());
    assert_eq!(s.buffer_len(), s.len());
    assert_eq!(s.value(), &exp);
    assert!(!s.locked());

    // Unlocked alt len
    src.set_len(6);
    let s = src.clone();
    assert_eq!(s.len(), 6);
    assert_eq!(s.buffer_len(), exp.len());
    assert_eq!(s.value(), &exp[..6]);
    assert!(!s.locked());
}

//=============================================================================
// ByteMaskGenerator
//-----------------------------------------------------------------------------
#[test]
fn test_bytemaskgenerator_new() {
    let g = ByteMaskGenerator::new(1234);
    assert_eq!(g.state, 1234)
}

#[test]
fn test_bytemaskgenerator_next() {
    // Reference
    let mut g = ByteMaskGenerator::new(1234);
    assert_eq!(g.next(), 0x5b);
    assert_eq!(g.next(), 0x18);
    assert_eq!(g.next(), 0x2a);

    // Test stability
    let seed: u64 = random();
    let mut g1 = ByteMaskGenerator::new(seed);
    let mut g2 = ByteMaskGenerator::new(seed);
    for _ in 0..1000 {
        assert_eq!(g1.next(), g2.next());
    }
}

//=============================================================================
// DefaultProtectedValue
//-----------------------------------------------------------------------------
#[test]
fn test_defaultprotectedvalue_apply_mask() {
    let zero: [u8; 16] = [0; 16];
    let seed = 1234;
    let mut apply: [u8; 16] = [0; 16];

    DefaultProtectedValue::apply_mask(seed, &mut apply);
    assert_ne!(&zero, &apply);
    DefaultProtectedValue::apply_mask(seed, &mut apply);
    assert_eq!(&zero, &apply);
}

#[test]
fn test_defaultprotectedvalue() {
    let exp: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

    let p = DefaultProtectedValue::new(&exp);
    assert_ne!(p.secret.value(), &exp);
    assert_ne!(p.seed, 0);

    let v = p.get_secret();
    assert_eq!(v.value(), &exp);
}

//=============================================================================
// ProtectedValue
//-----------------------------------------------------------------------------
#[test]
fn test_create_protected_value() {
    let exp: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let x = create_protected_value(&exp);
    let y = x.clone();

    let s1 = x.get_secret();
    let s2 = y.get_secret();
    assert_eq!(s1.value(), &exp);
    assert_eq!(s1.value(), s2.value());
}
