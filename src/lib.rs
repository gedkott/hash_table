use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub struct HashTable<K, V> {
    buckets: Vec<Vec<(K, V)>>,
}

impl<K: std::hash::Hash + std::fmt::Debug + Clone + PartialEq, V: Clone + std::fmt::Debug>
    HashTable<K, V>
{
    pub fn new() -> HashTable<K, V> {
        HashTable {
            buckets: vec![vec![]; 10],
        }
    }

    pub fn with_capacity(capacity: usize) -> HashTable<K, V> {
        HashTable {
            buckets: vec![vec![]; capacity],
        }
    }

    fn get_bucket_index(&self, k: &K) -> usize {
        let hash = calculate_hash(k);
        let bucket_index = hash as usize % self.buckets.capacity();
        bucket_index
    }

    pub fn insert(&mut self, k: K, v: V) -> () {
        let bucket_index = self.get_bucket_index(&k);
        self.buckets[bucket_index].push((k, v));
    }

    pub fn get(&self, k: K) -> Option<&V> {
        let bucket_index = self.get_bucket_index(&k);
        for (ek, v) in &self.buckets[bucket_index] {
            if ek == &k {
                return Some(v);
            }
        }
        return None;
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
        let gedalia = User {
            name: "gedalia".to_string(),
            age: 27,
        };
        let expected_result = Some(&gedalia);

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

        let gedalia = User {
            name: "gedalia".to_string(),
            age: 27,
        };
        let expected_gedalia_result = Some(&gedalia);
        let theo = User {
            name: "theo".to_string(),
            age: 0,
        };
        let expected_theo_result = Some(&theo);

        assert_eq!(gedalia_result, expected_gedalia_result);
        assert_eq!(theo_result, expected_theo_result);
    }
}
