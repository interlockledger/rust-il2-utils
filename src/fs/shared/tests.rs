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
use std::ffi::{OsStr, OsString};
use std::fs::{write, DirBuilder, File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::path::Path;

const TEST_DIR: &'static str = "tmp";

/// Get the path of a test file. This file will always be
/// inside the test directory.
///
/// This function panics if the test directory does not exist, cannot be
/// created or is not a directory.
fn get_test_file(name: &str) -> OsString {
    let path = Path::new(TEST_DIR);
    if !path.is_dir() {
        let builder = DirBuilder::new();
        builder.create(&path).unwrap();
    }
    path.join(name).into_os_string()
}

fn create_test_file(file_path: &OsStr) {
    let p = Path::new(&file_path);
    write(p, file_path.to_str().unwrap().as_bytes()).unwrap()
}

#[test]
fn test_get_test_file() {
    let f = get_test_file("test");
    println!("{:?}", f);
}

//=============================================================================
// SharedFileLockNameBuilder
//-----------------------------------------------------------------------------
struct DummySharedFileLockNameBuilder;

impl SharedFileLockNameBuilder for DummySharedFileLockNameBuilder {
    fn create_lock_file_name(&self, file_name: &OsStr) -> OsString {
        let mut lock_file_name = OsString::from("prefix.");
        lock_file_name.push(file_name);
        lock_file_name.push(".suffix");
        lock_file_name
    }
}

#[test]
fn test_sharedfilelocknamebuilder_get_lock_directory() {
    let f = DummySharedFileLockNameBuilder;

    let file = Path::new("name");
    assert_eq!(f.get_lock_directory(&file), file.parent());

    let file = Path::new("/test/name");
    assert_eq!(f.get_lock_directory(&file), file.parent());

    let file = Path::new("/");
    assert_eq!(f.get_lock_directory(&file), None);
}

#[test]
fn test_sharedfilelocknamebuilder_create_lock_file_path() {
    let f = DummySharedFileLockNameBuilder;

    let file = Path::new("name");
    assert_eq!(
        f.create_lock_file_path(&file).unwrap(),
        OsString::from("prefix.name.suffix")
    );

    let file = Path::new("/test/name");
    assert_eq!(
        f.create_lock_file_path(&file).unwrap(),
        OsString::from("/test/prefix.name.suffix")
    );

    let file = Path::new("/name");
    assert_eq!(
        f.create_lock_file_path(&file).unwrap(),
        OsString::from("/prefix.name.suffix")
    );

    let file = Path::new("/");
    assert!(f.create_lock_file_path(&file).is_err());

    let file = Path::new("/test/..");
    assert!(f.create_lock_file_path(&file).is_err());
}

//=============================================================================
// DefaultSharedFileLockNameBuilder
//-----------------------------------------------------------------------------
#[test]
fn test_defaultsharedfilelocknamebuilder_impl() {
    assert_eq!(DefaultSharedFileLockNameBuilder::LOCK_FILE_PREFIX, ".");
    assert_eq!(DefaultSharedFileLockNameBuilder::LOCK_FILE_SUFFIX, ".lock~");
}

#[test]
fn test_defaultsharedfilelocknamebuilder_namebuilder_create_lock_file_name() {
    let b = DefaultSharedFileLockNameBuilder;

    let name = OsStr::new("file");
    assert_eq!(b.create_lock_file_name(name), OsStr::new(".file.lock~"));

    let name = OsStr::new("z");
    assert_eq!(b.create_lock_file_name(name), OsStr::new(".z.lock~"));
}

//=============================================================================
// SharedFileReadLockGuard
//-----------------------------------------------------------------------------
#[test]
fn test_sharedfilereadlockguard_impl() {
    let lock_file = get_test_file("target.lock");
    create_test_file(lock_file.as_os_str());
    let target_file = get_test_file("target");
    create_test_file(target_file.as_os_str());

    let lock = fd_lock::RwLock::new(File::open(&lock_file).unwrap());
    let mut lock2 = fd_lock::RwLock::new(File::open(&lock_file).unwrap());
    let mut target = File::open(&target_file).unwrap();
    {
        let mut rlock = SharedFileReadLockGuard {
            file: &mut target,
            _lock: lock.read().unwrap(),
        };
        assert!(lock2.try_write().is_err());

        // Check if it is pointing to the correct file
        let mut buff = Vec::<u8>::new();
        rlock.read_to_end(&mut buff).unwrap();
        let buff_len = buff.len() as u64;
        let contents = String::from_utf8(buff).unwrap();
        let exp = target_file.to_str().unwrap();
        assert_eq!(contents, exp);

        let pos = rlock.seek(SeekFrom::End(0)).unwrap();
        assert_eq!(pos, buff_len);

        let size = rlock.file().metadata().unwrap().len() as u64;
        assert_eq!(size, buff_len);
    }
    let l = lock2.try_write().unwrap();
    drop(l);
}
