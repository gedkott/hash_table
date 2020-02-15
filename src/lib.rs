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

impl<K: std::hash::Hash + PartialEq, V> Default for HashTable<K, V> {
    fn default() -> Self {
        let default_number_of_starting_buckets = 10;
        let mut buckets: Vec<Vec<(K, V)>> = vec![];
        for _ in 0..default_number_of_starting_buckets {
            buckets.push(vec![]);
        }

        HashTable {
            buckets,
            total_entries: 0,
        }
    }
}

impl<K: std::hash::Hash + PartialEq, V> HashTable<K, V> {
    pub fn new() -> HashTable<K, V> {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> HashTable<K, V> {
        let mut buckets: Vec<Vec<(K, V)>> = vec![];
        for _ in 0..capacity {
            buckets.push(vec![]);
        }

        HashTable {
            buckets,
            total_entries: 0,
        }
    }

    fn load_factor(&self) -> f64 {
        self.total_entries as f64 / self.buckets.len() as f64
    }

    pub fn insert(&mut self, k: K, v: V) {
        let hash = calculate_hash(&k);
        let bucket_index = hash as usize % self.buckets.len();
        self.buckets[bucket_index].push((k, v));
        self.total_entries += 1;
        let current_load_factor = self.load_factor();
        if current_load_factor > 0.75 {
            let mut new_buckets: Vec<Vec<(K, V)>> = vec![];
            let extended_number_of_buckets = self.buckets.len() * 2;
            for _ in 0..extended_number_of_buckets {
                new_buckets.push(vec![]);
            }

            for bucket in &mut self.buckets {
                for (ek, ev) in bucket.drain(..) {
                    let hash = calculate_hash(&ek);
                    let bucket_index = hash as usize % new_buckets.len();
                    new_buckets[bucket_index].push((ek, ev));
                }
            }

            self.buckets = new_buckets;
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let hash = calculate_hash(k);
        let bucket_index = hash as usize % self.buckets.len();
        for (ek, v) in &self.buckets[bucket_index] {
            if ek == k {
                return Some(v);
            }
        }
        None
    }

    pub fn capacity(&self) -> usize {
        self.buckets.len()
    }
}

pub struct HashTableIterator<'a, K, V> {
    current_bucket_index: usize,
    current_element_index: usize,
    table: &'a HashTable<K, V>,
}

impl<'a, K, V> Iterator for HashTableIterator<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.current_bucket_index;
        let curr_bucket = &self.table.buckets[i];
        let next_item = curr_bucket.get(self.current_element_index);
        match next_item {
            Some(e) => {
                // println!("still on current bucket index");
                // println!("bi {} ei {}, nei {}", i, self.current_element_index, self.current_element_index + 1);
                self.current_element_index += 1;
                Some(e)
            }
            None => {
                // println!("current bucket and index combo resulted in None");
                // println!("curr bucket has {} elements", curr_bucket.len());
                if self.current_element_index + 1 < curr_bucket.len() {
                    // println!("still on current bucket index");
                    // println!("bi {} ei {}, nei {}", i, self.current_element_index, self.current_element_index + 1);
                    self.current_element_index += 1;
                } else {
                    // println!("bucket fully checked");
                    // println!("bi {} ei {}, nbi {} nei {}", i, self.current_element_index, i + 1, 0);
                    if self.current_bucket_index + 1 >= self.table.buckets.len() {
                        return None;
                    } else {
                        self.current_bucket_index += 1;
                        self.current_element_index = 0;
                        return self.next();
                    }
                }
                self.next()
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashTable<K, V> {
    type Item = &'a (K, V);

    type IntoIter = HashTableIterator<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        HashTableIterator {
            current_element_index: 0,
            current_bucket_index: 0,
            table: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::HashTable;

    #[derive(PartialEq, Debug, Clone)]
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

        let result = hash_table.get(&"gedalia");
        let gedalia = User {
            name: "gedalia".to_string(),
            age: 27,
        };
        let expected_result = Some(&gedalia);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_get_key_parameter_is_not_moved() {
        let mut hash_table = HashTable::new();

        let gedalia_string = String::from("gedalia");

        let gedalia = User {
            name: gedalia_string.clone(),
            age: 27,
        };

        hash_table.insert(gedalia_string.clone(), gedalia.clone());

        let result = hash_table.get(&gedalia_string);
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

        let gedalia_result = hash_table.get(&"gedalia");
        let theo_result = hash_table.get(&"theo");

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
    fn test_dynamic_resizing() {
        let mut hash_table = HashTable::with_capacity(9);

        assert_eq!(hash_table.capacity(), 9);

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

        for user in users {
            hash_table.insert(user.name.to_string(), user);
        }

        let gedalia_result = hash_table.get(&String::from("gedalia"));
        let caine_result = hash_table.get(&String::from("caine"));

        let gedalia = &User {
            name: "gedalia".to_string(),
            age: 27,
        };
        let expected_gedalia_result = Some(gedalia);
        let caine = &User {
            name: "caine".to_string(),
            age: 22,
        };
        let expected_caine_result = Some(caine);

        assert_eq!(gedalia_result, expected_gedalia_result);
        assert_eq!(caine_result, expected_caine_result);

        assert!(hash_table.capacity() > 9);
        assert_eq!(hash_table.capacity(), 18);
        assert_ne!(hash_table.capacity(), 9);
    }

    #[test]
    fn test_iteration_over_hash_table() {
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

        for user in users {
            hash_table.insert(user.name.to_string(), user);
        }

        for (k, v) in &hash_table {
            println!("{} {:?}", k, v);
        }

        // should still be able to use hash table AKA this should compile
        hash_table.insert(
            String::from("nowhereman"),
            User {
                name: String::from("nowhereman"),
                age: -1,
            },
        );

        hash_table.get(&String::from("nowhereman"));
    }
}
