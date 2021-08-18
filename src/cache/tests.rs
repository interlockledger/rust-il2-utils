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
// CacheEntry
//-----------------------------------------------------------------------------
#[test]
fn test_simplecacheentry_impl() {
    let v = Arc::new(10 as u64);

    let e = SimpleCacheEntry::new(&v, 10);
    assert_eq!(e.counter(), 10);

    let vr = e.get_value();
    assert_eq!(v, vr);
    assert!(!std::ptr::eq(&v, &vr));

    let mut e = SimpleCacheEntry::new(&v, 10);
    assert_eq!(e.counter(), 10);
    e.set_counter(1234);
    assert_eq!(e.counter(), 1234);
}

//=============================================================================
// SimpleCacheEngine
//-----------------------------------------------------------------------------
#[test]
fn test_simplecacheengine_impl_new() {
    let e: SimpleCacheEngine<u64, u64> = SimpleCacheEngine::new(10);
    assert_eq!(e.map.len(), 0);
    assert_eq!(e.max_size, 10);
    assert_eq!(e.counter, 0);
}

#[test]
fn test_simplecacheengine_impl_next_counter() {
    let mut e: SimpleCacheEngine<u64, u64> = SimpleCacheEngine::new(10);

    assert_eq!(e.counter, 0);
    assert_eq!(e.next_counter(), 0);
    assert_eq!(e.next_counter(), 1);
    assert_eq!(e.next_counter(), 2);
    assert_eq!(e.next_counter(), 3);
}

#[test]
fn test_simplecacheengine_impl_remove_oldest() {
    let mut e: SimpleCacheEngine<u64, u64> = SimpleCacheEngine::new(10);

    for key in 0..10 as u64 {
        let value = Arc::new(key);
        e.map.insert(key, SimpleCacheEntry::new(&value, key));
    }
    for key in 0..10 as u64 {
        assert_eq!(e.len(), (10 - key) as usize);
        e.remove_oldest();
        assert_eq!(e.len(), (10 - key - 1) as usize);
        assert!(e.get(&key).is_none());
    }
}

#[test]
fn test_simplecacheengine_simplecacheengine_get() {
    let mut e: SimpleCacheEngine<u64, u64> = SimpleCacheEngine::new(10);

    for key in 0..10 as u64 {
        let value = Arc::new(key + 100);
        let counter = e.next_counter();
        e.map.insert(key, SimpleCacheEntry::new(&value, counter));
    }

    // Test the recovery and the counter update at each
    for key in 0..10 as u64 {
        let old_counter = e.map.get(&key).unwrap().counter;
        let v = e.get(&key).unwrap();
        assert_eq!(*v, key + 100);
        let new_counter = e.map.get(&key).unwrap().counter;
        assert!(old_counter < new_counter);
    }

    // Ensure that the counter is always increased
    for key in 0..10 as u64 {
        let old_counter = e.map.get(&key).unwrap().counter;
        let v = e.get(&key).unwrap();
        assert_eq!(*v, key + 100);
        let new_counter = e.map.get(&key).unwrap().counter;
        assert!(old_counter < new_counter);
    }

    let key = 10 as u64;
    assert!(e.get(&key).is_none());
}

#[test]
fn test_simplecacheengine_simplecacheengine_insert() {
    let mut e: SimpleCacheEngine<u64, u64> = SimpleCacheEngine::new(10);

    // Adding 10 entries
    for key in 0..10 as u64 {
        assert_eq!(e.len(), key as usize);
        let value = Arc::new(key + 100);
        let curr_counter = e.counter;
        e.insert(key, &value);
        assert_eq!(e.len(), (key + 1) as usize);
        let entry = e.map.get(&key).unwrap();
        assert_eq!(entry.counter, curr_counter);
        assert_eq!(*entry.value, *value);
    }

    // Replacing entries
    for key in 0..10 as u64 {
        assert_eq!(e.len(), 10);
        let value = Arc::new(key + 1000);
        let curr_counter = e.counter;
        e.insert(key, &value);
        assert_eq!(e.len(), 10);
        let entry = e.map.get(&key).unwrap();
        assert_eq!(entry.counter, curr_counter);
        assert_eq!(*entry.value, *value);
    }

    // Adding 10 new entries
    for key in 10..20 as u64 {
        assert_eq!(e.len(), 10);
        let value = Arc::new(key + 1000);
        let curr_counter = e.counter;
        e.insert(key, &value);
        assert_eq!(e.len(), 10);
        let entry = e.map.get(&key).unwrap();
        assert_eq!(entry.counter, curr_counter);
        assert_eq!(*entry.value, *value);

        // The older key will always be the one with the smallest key
        let removed = key - 10;
        assert!(e.map.get(&removed).is_none());
    }
}

#[test]
fn test_simplecacheengine_simplecacheengine_clear() {
    let mut e: SimpleCacheEngine<u64, u64> = SimpleCacheEngine::new(10);

    for key in 0..10 as u64 {
        let value = Arc::new(key + 100);
        e.insert(key, &value);
        assert_eq!(e.len(), (key + 1) as usize);
    }
    assert!(!e.is_empty());
    e.clear();
    assert!(e.is_empty());
}

//=============================================================================
// SimpleCache
//-----------------------------------------------------------------------------
#[test]
fn test_simplecache_impl() {
    let c: SimpleCache<u64, u64> = SimpleCache::new(10);
    {
        let r = c.engine.read().unwrap();
        assert_eq!(r.len(), 0);
    }
    {
        let mut w = c.engine.write().unwrap();
        w.clear();
    }
}

#[test]
fn test_simplecache_valuecache_concurrent_insert() {
    let c: Arc<SimpleCache<u64, u64>> = Arc::new(SimpleCache::new(10));

    let t1c = Arc::clone(&c);
    let t1 = std::thread::spawn(move || {
        for key in 0..8 as u64 {
            let value = Arc::new(key + 1000);
            t1c.insert(key, &value);
        }
    });
    let t2c = Arc::clone(&c);
    let t2 = std::thread::spawn(move || {
        for key in 2..10 as u64 {
            let value = Arc::new(key + 10000);
            t2c.insert(key, &value);
        }
    });
    t1.join().unwrap();
    t2.join().unwrap();

    // Test the result of the inserts
    for key in 0..10 as u64 {
        c.get(&key).unwrap();
    }
}

#[test]
fn test_simplecache_valuecache_concurrent_get() {
    let c: Arc<SimpleCache<u64, u64>> = Arc::new(SimpleCache::new(10));

    for key in 0..10 as u64 {
        let value = Arc::new(key + 1000);
        c.insert(key, &value);
    }

    let t1c = Arc::clone(&c);
    let t1 = std::thread::spawn(move || {
        for key in 0..8 as u64 {
            t1c.get(&key).unwrap();
        }
    });
    let t2c = Arc::clone(&c);
    let t2 = std::thread::spawn(move || {
        for key in 2..10 as u64 {
            t2c.get(&key).unwrap();
        }
    });
    t1.join().unwrap();
    t2.join().unwrap();

    // Test the counters
    for key in 0..10 as u64 {
        let counter = c.engine.read().unwrap().map.get(&key).unwrap().counter;
        print!("{:?} ", counter);
        assert!(counter >= 10);
    }
}

#[test]
fn test_simplecache_valuecache_concurrent_clear() {
    let c: Arc<SimpleCache<u64, u64>> = Arc::new(SimpleCache::new(10));

    for key in 0..10 as u64 {
        let value = Arc::new(key + 1000);
        c.insert(key, &value);
    }

    let t1c = Arc::clone(&c);
    let t1 = std::thread::spawn(move || {
        t1c.clear();
        assert_eq!(t1c.len(), 0);
        assert!(t1c.is_empty());
    });
    let t2c = Arc::clone(&c);
    let t2 = std::thread::spawn(move || {
        t2c.clear();
        assert_eq!(t2c.len(), 0);
        assert!(t2c.is_empty());
    });
    t1.join().unwrap();
    t2.join().unwrap();

    assert_eq!(c.len(), 0);
    assert!(c.is_empty());
}
