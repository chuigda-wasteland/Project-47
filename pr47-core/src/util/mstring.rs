use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use smallvec::{SmallVec, smallvec};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StringHandle(u64);

pub struct StringPool {
    buckets: Box<[SmallVec<[Box<str>; 8]>]>,
    hash_max_items: u64
}

impl StringPool {
    pub fn new(bucket_count: usize) -> Self {
        assert_ne!(bucket_count, 0, "Zero buckets not allowed");
        assert!(bucket_count <= 1_000_000, "Too many buckets used");
        Self {
            buckets: vec![smallvec![]; bucket_count].into_boxed_slice(),
            hash_max_items: Self::hash_max_items(bucket_count as u64)
        }
    }

    pub fn create(&mut self, value: &str) -> StringHandle {
        let hash: u64 = Self::hash_string(value);
        let bucket_id: u64 = hash % (self.buckets.len() as u64);

        let bucket: &mut SmallVec<_> =  unsafe { self.buckets.get_unchecked_mut(bucket_id as usize) };
        for (n, str) in bucket.iter().enumerate() {
            if str.as_ref() == value {
                return StringHandle(bucket_id * self.hash_max_items + n as u64);
            }
        }
        bucket.push(String::from(value).into_boxed_str());
        return StringHandle(bucket_id * self.hash_max_items + bucket.len() as u64 - 1);
    }

    pub fn get(&self, handle: StringHandle) -> &str {
        let bucket_id: u64 = handle.0 / self.hash_max_items;
        let n: u64 = handle.0 % self.hash_max_items;

        unsafe {
            let bucket: &SmallVec<_> = self.buckets.get_unchecked(bucket_id as usize);
            debug_assert!((n as usize) < bucket.len());
            bucket.get_unchecked(n as usize)
        }
    }

    fn hash_string(string: &str) -> u64 {
        let mut hasher: DefaultHasher = DefaultHasher::new();
        string.hash(&mut hasher);
        hasher.finish()
    }

    const fn hash_max_items(bucket_count: u64) -> u64 {
        u64::MAX / bucket_count
    }
}

pub trait MToString {
    fn m_to_string(&self, pool: &StringPool) -> String;
}

pub trait MDisplay {
    fn m_fmt(&self, pool: &StringPool, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

#[cfg(test)]
mod test {
    use crate::util::mstring::{StringHandle, StringPool};

    #[test]
    #[ignore]
    fn test_mstring() {
        let mut strings = vec![];
        for i in 0..=255 {
            for j in 0..=255 {
                for k in 0..=9 {
                    strings.push(format!("{:02X}-{:02X}-{:02X}", i, j, k));
                }
            }
        }
        assert_eq!(strings.len(), 655360);

        let mut string_pool = StringPool::new(4096);
        let handles: Vec<StringHandle> = strings.iter().map(|s| string_pool.create(s)).collect();

        handles.iter().zip(strings.iter()).for_each(|(h, s)| {
            assert_eq!(string_pool.get(*h), s)
        })
    }

    #[test]
    #[ignore]
    fn test_mstring_repeated_insert() {
        let mut strings = vec![];
        for i in 0..=255 {
            for j in 0..=255 {
                for k in 0..=9 {
                    strings.push(format!("{:02X}-{:02X}-{:02X}", i, j, k));
                }
            }
        }
        assert_eq!(strings.len(), 655360);

        let mut string_pool = StringPool::new(4096);
        let handles: Vec<StringHandle> = strings.iter().map(|s| string_pool.create(s)).collect();

        strings.iter().zip(handles.iter()).for_each(|(s, h)| {
            assert_eq!(string_pool.create(s), *h);
        })
    }
}
