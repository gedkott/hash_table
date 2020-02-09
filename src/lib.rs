use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub struct HashTable<V: Clone + std::fmt::Debug> {
    buckets: Vec<Option<V>>,
}

impl<V: Clone + std::fmt::Debug> HashTable<V> {
    pub fn new() -> HashTable<V> {
        HashTable {
            buckets: vec![None; 10],
        }
    }

    pub fn with_capacity(capacity: usize) -> HashTable<V> {
        HashTable {
            buckets: vec![None; capacity],
        }
    }

    fn get_bucket_index<K: std::hash::Hash>(&self, k: &K) -> usize {
        let hash = calculate_hash(k);
        println!("hash computed: {}", hash);
        let bucket_index = hash as usize % self.buckets.capacity();
        println!("bucket index computed: {}", bucket_index);
        bucket_index
    }

    pub fn insert<K: std::hash::Hash + std::fmt::Debug>(&mut self, k: K, v: V) -> () {
        let bucket_index = self.get_bucket_index(&k);
        // implement separate chaining collision resolution
        if let Some(existing_v) = &self.buckets[bucket_index] {
            // we have a collision
            println!(
                "we have a collision on insert for {:?} against existing value {:?}",
                v, existing_v
            );
        }
        self.buckets[bucket_index] = Some(v);
    }

    pub fn get<K: std::hash::Hash>(&self, k: K) -> &Option<V> {
        let bucket_index = self.get_bucket_index(&k);
        &self.buckets[bucket_index]
    }
}

#[cfg(test)]
mod tests {
    use crate::HashTable;

    #[derive(Clone, PartialEq, Debug)]
    struct User {
        name: String,
        age: i32,
    }

    #[test]
    fn insert_and_get_key_value_pair() {
        let mut hash_table = HashTable::new();
        hash_table.insert(
            "gedalia",
            User {
                name: "gedalia".to_string(),
                age: 27,
            },
        );

        let result = hash_table.get("gedalia");
        let expected_result = &Some(User {
            name: "gedalia".to_string(),
            age: 27,
        });

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_collisions() {
        let mut hash_table = HashTable::with_capacity(1);
        hash_table.insert(
            "gedalia",
            User {
                name: "gedalia".to_string(),
                age: 27,
            },
        );

        hash_table.insert(
            "theo",
            User {
                name: "theo".to_string(),
                age: 0,
            },
        );

        let gedalia_result = hash_table.get("gedalia");
        let theo_result = hash_table.get("theo");

        let expected_gedalia_result = &Some(User {
            name: "gedalia".to_string(),
            age: 27,
        });
        let expected_theo_result = &Some(User {
            name: "theo".to_string(),
            age: 0,
        });

        assert_eq!(gedalia_result, expected_gedalia_result);
        assert_eq!(theo_result, expected_theo_result);
    }
}
