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

impl<K, V> Default for HashTable<K, V> {
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

    pub fn insert_mut(&mut self, k: K, v: V) -> &mut V {
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
            let len = self.buckets[bucket_index].len();
            let (_, v) = self.buckets[bucket_index].get_mut(len - 1).unwrap();
            v
        } else {
            let len = self.buckets[bucket_index].len();
            let (_, v) = self.buckets[bucket_index].get_mut(len - 1).unwrap();
            v
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

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        let hash = calculate_hash(k);
        let bucket_index = hash as usize % self.buckets.len();
        for (ek, v) in &mut self.buckets[bucket_index] {
            if ek == k {
                return Some(v);
            }
        }
        None
    }

    pub fn capacity(&self) -> usize {
        self.buckets.len()
    }

    pub fn entry(&mut self, k: K) -> Entry<'_, K, V> {
        if self.get_mut(&k).is_some() {
            Entry::Occupied { ht: self, k }
        } else {
            Entry::Vacant { ht: self, k }
        }
    }
}

pub enum Entry<'a, K, V> {
    Occupied { ht: &'a mut HashTable<K, V>, k: K },
    Vacant { ht: &'a mut HashTable<K, V>, k: K },
}

impl<'a, K: PartialEq + Hash, V> Entry<'a, K, V> {
    pub fn or_insert(self, v: V) -> &'a mut V {
        match self {
            Entry::Occupied { k, ht } => {
                let e = ht.get_mut(&k);
                e.unwrap()
            }
            Entry::Vacant { k, ht } => ht.insert_mut(k, v),
        }
    }
}

pub struct HashTableIterator<'a, K, V> {
    elements_iterator: Box<dyn Iterator<Item = &'a (K, V)> + 'a>,
    buckets_iterator: Box<dyn Iterator<Item = &'a Vec<(K, V)>> + 'a>,
}

impl<'a, K, V> IntoIterator for &'a HashTable<K, V> {
    type Item = &'a (K, V);

    type IntoIter = HashTableIterator<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let mut buckets_iterator = self.buckets.iter();
        // first elements iterator needs to be initialized
        let elements_iterator = buckets_iterator
            .next()
            .map(|bi| bi.iter())
            .unwrap_or_else(|| [].iter());
        HashTableIterator {
            elements_iterator: Box::new(elements_iterator),
            buckets_iterator: Box::new(buckets_iterator),
        }
    }
}

impl<'a, K, V> Iterator for HashTableIterator<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.elements_iterator.next().or_else(|| {
            // no element available in this bucket
            // iterating to next bucket and either
            // ending iteration or recursing
            self.buckets_iterator.next().and_then(|b| {
                // bucket is available so we are recursing
                let elements_iterator = b.iter();
                self.elements_iterator = Box::new(elements_iterator);
                self.next()
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::HashTable;

    #[derive(PartialEq, PartialOrd, Debug, Eq, Clone, Ord)]
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

        let mut users = vec![
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

        users.sort();

        for user in &users {
            hash_table.insert(user.name.to_string(), user);
        }

        for (k, v) in &hash_table {
            let found = &users.binary_search(v);
            assert!(found.is_ok());
            assert!(found.map(|i| &users[i].name == k).unwrap() == true);
        }

        let nowhere_man = User {
            name: String::from("nowhereman"),
            age: -1,
        };

        // should still be able to use hash table AKA this should compile
        hash_table.insert(String::from("nowhereman"), &nowhere_man);

        assert_eq!(
            hash_table.get(&String::from("nowhereman")),
            Some(&&User {
                name: String::from("nowhereman"),
                age: -1,
            })
        );
    }

    #[test]
    fn test_entry_interface() {
        let mut hash_table: HashTable<&str, User> = HashTable::new();

        let g_backup = User {
            name: "gedalia".to_string(),
            age: 27,
        };

        let user_entry = hash_table.entry("gedalia");
        let user = user_entry.or_insert(g_backup);
        // user_entry; // should not compile if uncommented since or_insert moves the entry (consumed)
        (*user).age += 100;

        let user = hash_table.get(&"gedalia");
        assert_eq!(user.unwrap().age, 127);
    }
}
