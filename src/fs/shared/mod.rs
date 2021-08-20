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
//! This module implements a shared file with access controlled by an
//! "advisory lock". It allows multiple read access while restricts write
//! access to a single actor at any given time.
//!
//! This lock can be used to coordinate access to the file but should not be
//! used as a mean to guarantee security restrictions to t

#[cfg(test)]
mod tests;

use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::path::Path;

//=============================================================================
// SharedFileLockNameBuilder
//-----------------------------------------------------------------------------
pub trait SharedFileLockNameBuilder {
    /// Creates the lock file name from the target file.
    ///
    /// Arguments:
    /// - `file`: The path to the target file;
    ///
    /// Returns:
    /// - `Ok(x)`: The lock file path;
    /// - `Err(e)`: If the lock file name cannot be created from the specified path;
    fn create_lock_file_path(&self, file: &Path) -> Result<OsString> {
        let file_name = match file.file_name() {
            Some(name) => name,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Unable to extract the file name.",
                ))
            }
        };
        let lock_file_name = self.create_lock_file_name(&file_name);
        match self.get_lock_directory(file) {
            Some(v) => Ok(v.join(lock_file_name).into_os_string()),
            None => Ok(lock_file_name),
        }
    }

    /// Returns the lock directory. It is used by [`Self::create_lock_file_path()`] to
    /// compose the lock file name.
    ///
    /// By default it is the same directory of the protected file.
    ///
    /// Arguments:
    /// - `file`: The path to the protected file.
    ///
    /// Returns:
    /// - `Some(x)`: The parent path;
    /// - `None`: If the parent path is not present or is not available;
    fn get_lock_directory<'a>(&self, file: &'a Path) -> Option<&'a Path> {
        file.parent()
    }

    /// Creates the lock file name based on the original file name. It is used by
    /// [`Self::create_lock_file_path()`] to compose the lock file name.
    ///
    /// Arguments:
    /// - `file_name`: The name of the file that will be protected;
    ///
    /// Returns the name of the lock file.
    fn create_lock_file_name(&self, file_name: &OsStr) -> OsString;
}

//=============================================================================
// DefaultSharedFileLockNameBuilder
//-----------------------------------------------------------------------------
/// This is the default implementation of the [`SharedFileLockNameBuilder`].
///
/// The lock file will be in the same directory of the target file
/// and will have the same name of the target file with the prefix
/// "." and the suffix ".lock~".
pub struct DefaultSharedFileLockNameBuilder;

impl DefaultSharedFileLockNameBuilder {
    /// Prefix of the lock file.
    pub const LOCK_FILE_PREFIX: &'static str = ".";

    /// Suffic of the lock file.
    pub const LOCK_FILE_SUFFIX: &'static str = ".lock~";
}

impl SharedFileLockNameBuilder for DefaultSharedFileLockNameBuilder {
    fn create_lock_file_name(&self, file_name: &OsStr) -> OsString {
        let mut lock_file_name = OsString::from(Self::LOCK_FILE_PREFIX);
        lock_file_name.push(file_name);
        lock_file_name.push(Self::LOCK_FILE_SUFFIX);
        lock_file_name
    }
}

//=============================================================================
// SharedFileReadLockGuard
//-----------------------------------------------------------------------------
/// An RAII implementation of an “advisory lock” of a shared read to the
/// protected file. When this structure is dropped (falls out of scope), the
/// shared read lock is released.
///
/// It exposes the the traits [`Read`] and [`Seek`] over the shared file in
/// order to restrict the access to read operations.
///
/// See [`SharedFile`] for further details about how it works.
pub struct SharedFileReadLockGuard<'a> {
    file: &'a mut File,
    _lock: fd_lock::RwLockReadGuard<'a, File>,
}

impl<'a> SharedFileReadLockGuard<'a> {
    /// Returns a reference to the protected file.
    pub fn file(&self) -> &File {
        self.file
    }
}

impl<'a> Read for SharedFileReadLockGuard<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf)
    }
}

impl<'a> Seek for SharedFileReadLockGuard<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.file.seek(pos)
    }
}

//=============================================================================
// SharedFileWriteLockGuard
//-----------------------------------------------------------------------------
/// An RAII implementation of an “advisory lock” of a exclusive read and write
/// to the protected file. When this structure is dropped (falls out of scope),
/// the shared read lock is released.
///
/// It exposes the the traits [`Read`], [`Write`] and [`Seek`] over the shared
/// file. Since it grants exclusive access to the file, this struct also
/// grants a mutable access to the protecte file instance.
///
/// See [`SharedFile`] for further details about how it works.
pub struct SharedFileWriteLockGuard<'a> {
    file: &'a mut File,
    _lock: fd_lock::RwLockWriteGuard<'a, File>,
}

impl<'a> SharedFileWriteLockGuard<'a> {
    /// Returns a reference to the inner file.
    pub fn file(&self) -> &File {
        self.file
    }

    /// Returns a mutable reference to the inner file.
    pub fn mut_file(&mut self) -> &mut File {
        self.file
    }
}

impl<'a> Read for SharedFileWriteLockGuard<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf)
    }
}

impl<'a> Write for SharedFileWriteLockGuard<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.file.write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
}

impl<'a> Seek for SharedFileWriteLockGuard<'a> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.file.seek(pos)
    }
}

//=============================================================================
// SharedFile
//-----------------------------------------------------------------------------
/// This struct implements an “advisory lock” of a file using an auxiliary
/// lock file to control the shared read access to the file as well as an
/// exclusive read and write access to it.
///
/// Internally, it uses the crate `fd-lock` to control the access to the lock
/// file while protecting the access tot the actual file.
///
/// The protected file is always opened with shared read and write and create
/// options.
///
/// ## Locking the same file in multiple threads
///
/// It is very important to notice that this struct is not thread safe and must
/// be protected by a Mutex whenever necessary. It happens because the file
/// offset pointer cannot be safely shared among multiple threads even for
/// read operations. Unfortunately, the use of the mutex or other sync
/// mechanisms will lead to the serialization of both read and write locks
/// inside the same application.
///
/// Because of that, it is recommended to create multiple instances of this
/// struct pointing to the same file. The access control will be guaranteed
/// by the use of the lock file instead of the traditional thread sync
/// mechanisms.
pub struct SharedFile {
    lock: fd_lock::RwLock<File>,
    file: File,
}

impl SharedFile {
    /// Creates a new `SharedFile`. The name of the lock file will be determine
    /// automatically based on the name of the original file.
    ///
    /// The shared file is opened with the [`Self::default_options()`].
    ///
    /// Arguments:
    /// - `file`: The file to be protected;
    ///
    /// Returns the new instance of an IO error to indicate what went wrong.
    pub fn new(file: &Path) -> Result<Self> {
        let options = Self::default_options();
        Self::with_options(file, &options)
    }

    /// Creates a new `SharedFile`. The name of the lock file will be determine
    /// automatically based on the name of the original file.
    ///
    /// Arguments:
    /// - `file`: The file to be protected;    
    /// - `options`: [`OpenOptions`] used to open the file;
    ///
    /// Returns the new instance of an IO error to indicate what went wrong.
    pub fn with_options(file: &Path, options: &OpenOptions) -> Result<Self> {
        let lock_file_builder = DefaultSharedFileLockNameBuilder;
        Self::with_option_builder(file, options, &lock_file_builder)
    }

    /// Creates a new `SharedFile`. The name of the lock file will be determine
    /// by the specified [`SharedFileLockNameBuilder`].
    ///
    /// Arguments:
    /// - `file`: The file to be protected;
    /// - `options`: [`OpenOptions`] used to open the file;
    /// - `lock_file_builder`: The lock file builder to use;
    ///
    /// Returns the new instance of an IO error to indicate what went wrong.
    pub fn with_option_builder(
        file: &Path,
        options: &OpenOptions,
        lock_file_builder: &dyn SharedFileLockNameBuilder,
    ) -> Result<Self> {
        let lock_file = lock_file_builder.create_lock_file_path(file)?;
        Self::with_option_lock_file(file, options, Path::new(lock_file.as_os_str()))
    }

    /// Creates a new `SharedFile`.
    ///
    /// Arguments:
    /// - `file`: The file to be protected;
    /// - `options`: [`OpenOptions`] used to open the file;
    /// - `lock_file`: The lock file to use;
    ///
    /// Returns the new instance of an IO error to indicate what went wrong.
    pub fn with_option_lock_file(
        file: &Path,
        options: &OpenOptions,
        lock_file: &Path,
    ) -> Result<Self> {
        Ok(Self {
            lock: fd_lock::RwLock::new(File::create(lock_file)?),
            file: options.open(file)?,
        })
    }

    /// Returns the default open options used to open the target file. It sets
    /// read write and create to true.
    pub fn default_options() -> OpenOptions {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        options
    }

    /// Locks the file for shared read.
    ///
    /// Returns read lock that grants access to the file.
    pub fn read(&mut self) -> Result<SharedFileReadLockGuard<'_>> {
        Ok(SharedFileReadLockGuard {
            _lock: self.lock.read()?,
            file: &mut self.file,
        })
    }

    /// Locks the file for exclusive write and read
    ///
    /// Returns read/write lock that grants access to the file.
    pub fn write(&mut self) -> Result<SharedFileWriteLockGuard<'_>> {
        Ok(SharedFileWriteLockGuard {
            _lock: self.lock.write()?,
            file: &mut self.file,
        })
    }

    /// Attempts to locks the file for shared read. It fails without waiting if
    /// the lock cannot be acquired.
    ///
    /// Returns read lock that grants access to the file.
    pub fn try_read(&mut self) -> Result<SharedFileReadLockGuard<'_>> {
        Ok(SharedFileReadLockGuard {
            _lock: self.lock.try_read()?,
            file: &mut self.file,
        })
    }

    /// Attempts to acquire the file lock for exclusive write and read. It fails
    /// without waiting if the lock cannot be acquired.
    ///
    /// Returns read/write lock that grants access to the file.
    pub fn try_write(&mut self) -> Result<SharedFileWriteLockGuard<'_>> {
        Ok(SharedFileWriteLockGuard {
            _lock: self.lock.try_write()?,
            file: &mut self.file,
        })
    }
}
