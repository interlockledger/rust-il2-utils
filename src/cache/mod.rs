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
//! This module implements a very simple associative cache that stores read-only
//! entries associated to a key.
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

#[cfg(test)]
mod tests;

//=============================================================================
// ValueCache
//-----------------------------------------------------------------------------
/// This trait is implemented by all value caches on this module. A value cache
/// must be able to associate shared read-only values to a given key value.
///
/// It is up to the implementator of this trait to define how old values are
/// prunned from the cache.
///
/// All methods of this trait are required to be thread safe.
pub trait ValueCache<K: Eq + Hash + Copy + Sync, V: Send + Sync>: Send {
    /// Gets the value from the cache if it exists.
    ///
    /// Arguments:
    /// - `key`: The key to be found;
    ///
    /// Returns:
    /// - `Some(v)`: The cached value. `v` is a new [`Arc`] that points to it.
    /// - `None`: IF the entry is not in the cache;
    fn get(&self, key: &K) -> Option<Arc<V>>;

    /// Inserts the value into the cache. Reinserting a new value with the
    /// same key will replace the existing value.
    ///
    /// Arguments:
    /// - `key`: The key;
    /// - `value`: A reference to an [`Arc`] that points to the value;
    fn insert(&self, key: K, value: &Arc<V>);

    /// Removes all entries from the cache.
    fn clear(&self);

    /// Returns the number of entries in the cache.
    fn len(&self) -> usize;

    /// Returns true if the cache is empty or false otherwise.
    fn is_empty(&self) -> bool;
}

//=============================================================================
// CacheEntry
//-----------------------------------------------------------------------------
/// This struct implements a SimpleCache entry. The value is shared by an
/// [`Arc`] reference.
struct SimpleCacheEntry<V: Send + Sync> {
    value: Arc<V>,
    counter: u64,
}

impl<V: Send + Sync> SimpleCacheEntry<V> {
    /// Creates a new [`SimpleCacheEntry`].
    ///
    /// Arguments:
    /// - `value`: The value;
    /// - `counter`: The current counter;
    ///
    pub fn new(value: &Arc<V>, counter: u64) -> Self {
        Self {
            value: Arc::clone(value),
            counter,
        }
    }

    /// Returns a new [`Arc`] that points to the value.
    pub fn get_value(&self) -> Arc<V> {
        Arc::clone(&self.value)
    }

    /// Returns the current counter. This value can be used
    /// to determine what entry is the oldest in this cache.
    pub fn counter(&self) -> u64 {
        self.counter
    }

    /// Sets the counter.
    ///
    /// Arguments:
    /// - `counter`: The new counter;
    pub fn set_counter(&mut self, counter: u64) {
        self.counter = counter
    }
}

//=============================================================================
// CacheEngine
//-----------------------------------------------------------------------------
/// This trait is implemented by all value caches on this module. A value cache
/// must be able to associate shared read-only values to a given key value.
///
/// It is up to the implementator of this trait to define how old values are
/// prunned from the cache.
///
/// All methods of this trait are required to be thread safe.
pub trait CacheEngine<K: Eq + Hash + Copy + Sync, V: Send + Sync>: Sync {
    /// Gets the value from the cache if it exists.
    ///
    /// Arguments:
    /// - `key`: The key to be found;
    ///
    /// Returns:
    /// - `Some(v)`: The cached value. `v` is a new [`Arc`] that points to it.
    /// - `None`: IF the entry is not in the cache;
    fn get(&mut self, key: &K) -> Option<Arc<V>>;

    /// Inserts the value into the cache.
    ///
    /// Arguments:
    /// - `key`: The key;
    /// - `value`: A reference to an [`Arc`] that points to the value;
    fn insert(&mut self, key: K, value: &Arc<V>);

    /// Removes all entries from the cache.
    fn clear(&mut self);

    /// Returns the number of entries in the cache.
    fn len(&self) -> usize;

    /// Returns true if the cache is empty or false otherwise.
    fn is_empty(&self) -> bool;
}

//=============================================================================
// SimpleCacheEngine
//-----------------------------------------------------------------------------
/// This struct implements the SimpleCacheEngine. It is the core of the
/// [`SimpleCache`] implementation.
///
/// When it reaches its maximum capacity it will drop the oldest unused entries.
///
/// This struct is not thread safe and must have its concurrency protected by
/// an external [`RwLock`] or other synchronization primitive.
struct SimpleCacheEngine<K: Eq + Hash + Copy + Send + Sync, V: Send + Sync> {
    map: HashMap<K, SimpleCacheEntry<V>>,
    max_size: usize,
    counter: u64,
}

impl<K: Eq + Hash + Copy + Send + Sync, V: Send + Sync> SimpleCacheEngine<K, V> {
    /// Creates a new `SimpleCacheEngine` with a given capacity.
    ///
    /// Arguments:
    /// - `max_size`: Maximum number of items in the cache;
    pub fn new(max_size: usize) -> Self {
        Self {
            map: HashMap::new(),
            max_size,
            counter: 0,
        }
    }

    /// Returns the next value of the internal counter.
    fn next_counter(&mut self) -> u64 {
        let ret = self.counter;
        self.counter += 1;
        ret
    }

    /// This method removes the entry with the smallest counter.
    fn remove_oldest(&mut self) {
        let mut key: Option<K> = None;
        let mut oldest = u64::MAX;
        for (k, v) in self.map.iter() {
            if v.counter() < oldest {
                key = Some(*k);
                oldest = v.counter()
            }
        }
        match key {
            Some(k) => {
                self.map.remove(&k);
            }
            None => (),
        };
    }
}

impl<K: Eq + Hash + Copy + Send + Sync, V: Send + Sync> CacheEngine<K, V>
    for SimpleCacheEngine<K, V>
{
    fn get(&mut self, key: &K) -> Option<Arc<V>> {
        let counter = self.next_counter();
        let entry = match self.map.get_mut(key) {
            Some(entry) => entry,
            None => return None,
        };
        entry.set_counter(counter);
        Some(entry.get_value())
    }

    fn insert(&mut self, key: K, value: &Arc<V>) {
        let counter = self.next_counter();
        self.map.insert(key, SimpleCacheEntry::new(value, counter));
        if self.map.len() > self.max_size {
            self.remove_oldest();
        }
    }

    fn clear(&mut self) {
        self.map.clear()
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

//=============================================================================
// SimpleCache
//-----------------------------------------------------------------------------
/// This struct implements a simple value cache that holds up to a certain
/// number of entries at a time.
///
/// When it reaches its maximum capacity it will drop the oldest unused entries.
///
/// All methods of this struct are thread-safe.
pub struct SimpleCache<K: Eq + Hash + Copy + Send + Sync, V: Send + Sync> {
    engine: RwLock<SimpleCacheEngine<K, V>>,
}

impl<K: Eq + Hash + Copy + Send + Sync, V: Send + Sync> SimpleCache<K, V> {
    /// Creates a new SimpleCache with a given capacity.
    ///
    /// Arguments:
    /// - `max_size`: Maximum number of items in the cache;
    pub fn new(max_size: usize) -> Self {
        Self {
            engine: RwLock::new(SimpleCacheEngine::new(max_size)),
        }
    }
}

impl<K: Eq + Hash + Copy + Send + Sync, V: Send + Sync> ValueCache<K, V> for SimpleCache<K, V> {
    fn get(&self, key: &K) -> Option<Arc<V>> {
        let mut s = self.engine.write().unwrap();
        s.get(key)
    }

    fn insert(&self, key: K, value: &Arc<V>) {
        let mut s = self.engine.write().unwrap();
        s.insert(key, value)
    }

    fn clear(&self) {
        let mut s = self.engine.write().unwrap();
        s.clear()
    }

    fn len(&self) -> usize {
        let s = self.engine.read().unwrap();
        s.len()
    }

    fn is_empty(&self) -> bool {
        let s = self.engine.read().unwrap();
        s.is_empty()
    }
}
