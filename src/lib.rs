use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub struct HashTable<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    total_entries: usize,
}

impl<K: std::hash::Hash + std::fmt::Debug + Clone + PartialEq, V: Clone + std::fmt::Debug>
    HashTable<K, V>
{
    pub fn new() -> HashTable<K, V> {
        let default_number_of_starting_buckets = 10;
        HashTable {
            buckets: vec![vec![]; default_number_of_starting_buckets],
            total_entries: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> HashTable<K, V> {
        HashTable {
            buckets: vec![vec![]; capacity],
            total_entries: 0,
        }
    }

    fn load_factor(&self) -> f64 {
        self.total_entries as f64 / self.buckets.len() as f64
    }

    pub fn insert(&mut self, k: K, v: V) -> () {
        let hash = calculate_hash(&k);
        let bucket_index = hash as usize % self.buckets.capacity();
        self.buckets[bucket_index].push((k, v));
        self.total_entries += 1;
        let current_load_factor = self.load_factor();
        if current_load_factor > 0.75 {
            let mut new_buckets: Vec<Vec<(K, V)>> = vec![vec![]; self.buckets.capacity() * 2];

            for bucket in &mut self.buckets {
                for (ek, ev) in bucket.drain(..) {
                    let hash = calculate_hash(&ek);
                    let bucket_index = hash as usize % new_buckets.capacity();
                    new_buckets[bucket_index].push((ek, ev));
                }
            }

            self.buckets = new_buckets;
        }
    }

    pub fn get(&self, k: K) -> Option<&V> {
        let hash = calculate_hash(&k);
        let bucket_index = hash as usize % self.buckets.capacity();
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

    #[test]
    fn test_load_factor_threshold_calculation() {
        let mut hash_table = HashTable::with_capacity(9);

        let users = vec![
            User {
                name: "gedalia".to_string(),
                age: 27,
            },
            User {
                name: "theo".to_string(),
                age: 0,
            },
            User {
                name: "aviva".to_string(),
                age: 26,
            },
            User {
                name: "chani".to_string(),
                age: 25,
            },
            User {
                name: "nachmi".to_string(),
                age: 24,
            },
            User {
                name: "avery".to_string(),
                age: 23,
            },
            User {
                name: "caine".to_string(),
                age: 22,
            },
        ];

        for user in &users {
            hash_table.insert(user.name.to_string(), user);
        }

        let gedalia_result = hash_table.get("gedalia".to_string());
        let caine_result = hash_table.get("caine".to_string());

        let gedalia = &User {
            name: "gedalia".to_string(),
            age: 27,
        };
        let expected_gedalia_result = Some(&gedalia);
        let caine = &User {
            name: "caine".to_string(),
            age: 22,
        };
        let expected_caine_result = Some(&caine);

        assert_eq!(gedalia_result, expected_gedalia_result);
        assert_eq!(caine_result, expected_caine_result);
    }
}
