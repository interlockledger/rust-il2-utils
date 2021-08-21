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
//! This module contains common test functions used by this library.
use std::ffi::OsString;
use std::fs::{read_dir, remove_dir_all, remove_file, write, DirBuilder};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

//=============================================================================
// File test utilities
//-----------------------------------------------------------------------------
/// This struct implements a set of utilities that helps with the management of
/// test files used in the unit tests.
pub struct TestDirUtils {
    test_dir: OsString,
}

impl TestDirUtils {
    /// Directory to be used by the unit tests.
    pub const DEFAULT_TEST_DIR: &'static str = "test_dir.tmp";

    /// Creates a new `TestDirUtils` with the default name.
    /// It will automatically create the test directory if it does not exist.
    ///
    /// Returns the new instance of an error if the test directory is invalid
    /// or cannot be created.
    pub fn new() -> Result<Self> {
        Self::with_path(Path::new(Self::DEFAULT_TEST_DIR))
    }

    /// Creates a new `TestDirUtils`. It will automatically create
    /// the test directory if it does not exist.
    ///
    /// Arguments:
    /// - `test_dir`: The path to the test directory;
    ///
    /// Returns the new instance of an error if the test directory is invalid
    /// or cannot be created.
    pub fn with_path(test_dir: &Path) -> Result<Self> {
        if !test_dir.exists() {
            DirBuilder::new().recursive(true).create(test_dir)?;
        } else if !test_dir.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("{:?} is not a directory!", test_dir),
            ));
        }
        Ok(Self {
            test_dir: OsString::from(test_dir),
        })
    }

    /// Returns the path to the test directory.
    pub fn test_dir(&self) -> &Path {
        Path::new(&self.test_dir)
    }

    /// Deletes all of the contents of the test directory.
    pub fn reset(&self) -> Result<()> {
        for entry in read_dir(self.test_dir())? {
            match entry {
                Ok(e) => {
                    let file_type = e.file_type()?;
                    if file_type.is_file() || file_type.is_symlink() {
                        remove_file(e.path())?;
                    } else if file_type.is_dir() {
                        remove_dir_all(e.path())?;
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Get the path of a file inside the test directory.
    pub fn get_test_file_path(&self, name: &str) -> OsString {
        let path = Path::new(&self.test_dir);
        path.join(name).into_os_string()
    }

    /// Create a test file inside the specified path. This file
    /// will have as its contents, the UTF-8 bytes that represents
    /// the name of the file.
    ///
    /// Arguments:
    /// - `name`: The name of the file to be created;
    ///
    /// Returns the path to the newly created file.
    pub fn create_test_file(&self, name: &str) -> Result<OsString> {
        let full_path = self.get_test_file_path(name);
        let p = Path::new(&full_path);
        write(p, full_path.to_str().unwrap().as_bytes())?;
        Ok(full_path)
    }

    /// Create a test file inside the specified path. This file
    /// will have as its contents, the UTF-8 bytes that represents
    /// the name of the file.
    ///
    /// Arguments:
    /// - `name`: The name of the file to be removed;
    pub fn delete_test_file(&self, name: &str) -> Result<()> {
        let full_path = self.get_test_file_path(name);
        let p = Path::new(&full_path);
        remove_file(p)
    }
}

//#[test]
fn test_testdirutils_new() {
    let test_dir_path = Path::new(TestDirUtils::DEFAULT_TEST_DIR);

    if test_dir_path.exists() {
        remove_dir_all(test_dir_path).unwrap();
    }
}
