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
    let mut target = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&target_file)
        .unwrap();
    {
        let mut rlock = SharedFileReadLockGuard {
            file: &mut target,
            _lock: lock.read().unwrap(),
        };
        // Cannot write
        assert!(lock2.try_write().is_err());
        // But can read
        drop(lock2.read().unwrap());

        // Check if it is pointing to the correct file by reading it
        let mut buff = Vec::<u8>::new();
        rlock.read_to_end(&mut buff).unwrap();
        let buff_len = buff.len() as u64;
        let contents = String::from_utf8(buff).unwrap();
        let exp = target_file.to_str().unwrap();
        assert_eq!(contents, exp);

        // Test Seek
        let pos = rlock.seek(SeekFrom::End(0)).unwrap();
        assert_eq!(pos, buff_len);

        // Test access to the inner File
        let size = rlock.file().metadata().unwrap().len() as u64;
        assert_eq!(size, buff_len);
    }
    let l = lock2.try_write().unwrap();
    drop(l);
}

//=============================================================================
// SharedFileWriteLockGuard
//-----------------------------------------------------------------------------
#[test]
fn test_sharedfilewritelockguard_impl() {
    let lock_file = get_test_file("target.lock");
    create_test_file(lock_file.as_os_str());
    let target_file = get_test_file("target");
    create_test_file(target_file.as_os_str());

    let mut lock = fd_lock::RwLock::new(File::open(&lock_file).unwrap());
    let mut lock2 = fd_lock::RwLock::new(File::open(&lock_file).unwrap());
    let mut target = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&target_file)
        .unwrap();
    {
        let mut rwlock = SharedFileWriteLockGuard {
            file: &mut target,
            _lock: lock.write().unwrap(),
        };
        // Cannot read nor write
        assert!(lock2.try_write().is_err());
        drop(lock2.try_read().is_err());

        // Check if it is pointing to the correct file by reading it
        let mut buff = Vec::<u8>::new();
        rwlock.read_to_end(&mut buff).unwrap();
        let buff_len = buff.len() as u64;
        let contents = String::from_utf8(buff).unwrap();
        let exp = target_file.to_str().unwrap();
        assert_eq!(contents, exp);

        // Test Seek
        let pos = rwlock.seek(SeekFrom::End(0)).unwrap();
        assert_eq!(pos, buff_len);

        // Test access to the inner File
        let size = rwlock.file().metadata().unwrap().len() as u64;
        assert_eq!(size, buff_len);

        let size = rwlock.mut_file().metadata().unwrap().len() as u64;
        assert_eq!(size, buff_len);

        // Test if it can write to it
        rwlock.mut_file().set_len(0).unwrap();
        let size = rwlock.file().metadata().unwrap().len() as u64;
        assert_eq!(size, 0);

        let sample: [u8; 4] = [1, 2, 3, 4];
        rwlock.seek(SeekFrom::Start(0)).unwrap();
        rwlock.write_all(&sample).unwrap();

        rwlock.seek(SeekFrom::Start(0)).unwrap();
        let mut buff = Vec::<u8>::new();
        rwlock.read_to_end(&mut buff).unwrap();
        assert_eq!(buff.as_slice(), &sample);
    }
    let l = lock2.try_write().unwrap();
    drop(l);
}

//=============================================================================
// SharedFile
//-----------------------------------------------------------------------------
#[test]
fn test_sharedfile_impl() {
    // This test ends up testing all constructors because
    // new() calls with_options(),
    // with_options() calls with_option_builder() and
    // with_option_builder() calls with_option_lock_file().
    //
    // Furthermore, it also tests read(), try_read(), write() and try_write() as
    // well in a concurrent scenario with at least to SharedFile instances
    // pointing to the same file.
    let dummy_content: &'static str = "123456";
    let test_file = get_test_file("protected");
    let test_file_path = Path::new(&test_file);
    let lock_file_builder = DefaultSharedFileLockNameBuilder;
    let test_file_lock = lock_file_builder
        .create_lock_file_path(test_file_path)
        .unwrap();
    let test_file_lock_path = Path::new(&test_file_lock);

    // Cleanup - No files should exist
    if test_file_lock_path.exists() {
        std::fs::remove_file(test_file_lock_path).unwrap();
    }
    if test_file_path.exists() {
        std::fs::remove_file(test_file_path).unwrap();
    }

    // Create the first from scratch
    let mut shared1 = SharedFile::new(test_file_path).unwrap();
    // Create the second just to confirm the operations
    let mut shared2 = SharedFile::new(test_file_path).unwrap();

    // Testing writing
    let mut write1 = shared1.write().unwrap();
    write1.write_all(dummy_content.as_bytes()).unwrap();
    assert!(shared2.try_read().is_err());
    assert!(shared2.try_write().is_err());

    // Test seek
    assert_eq!(write1.seek(SeekFrom::Start(0)).unwrap(), 0);
    write1.flush().unwrap();
    drop(write1);

    // Test reading from the other file
    let mut read2 = shared2.read().unwrap();
    let mut buff = Vec::<u8>::new();
    read2.read_to_end(&mut buff).unwrap();
    assert_eq!(dummy_content.as_bytes(), buff.as_slice());

    // Test concurrent read with try_read()
    assert!(shared1.try_write().is_err());
    let mut read1 = shared1.try_read().unwrap();
    let mut buff = Vec::<u8>::new();
    read1.read_to_end(&mut buff).unwrap();
    assert_eq!(dummy_content.as_bytes(), buff.as_slice());
    drop(read1);
    drop(read2);

    // Test write from 2 using try_write()
    let write2 = shared2.try_write().unwrap();
    assert!(shared1.try_write().is_err());
    assert!(shared1.try_read().is_err());
    drop(write2);
}

#[test]
fn test_sharedfile_default_options() {
    let options = SharedFile::default_options();
    let mut exp_options = OpenOptions::new();
    exp_options.read(true).write(true).create(true);
    // I'll compare the contents of both options using debug as it has
    // access to the internal fields. Furthermore, the debug strings should
    // be equal if the objects are instantiated in the same way. Furthermore,
    // I think it will
    assert_eq!(format!("{:?}", options), format!("{:?}", exp_options));
}
