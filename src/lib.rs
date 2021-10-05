use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;
use std::slice;
use std::vec::IntoIter;

pub trait SimpleHasher<K>
where
    K: Hash,
{
    fn hash(&self, t: &K) -> u64;
}

pub struct DefaultSimpleHasher;
impl DefaultSimpleHasher {
    fn new() -> Self {
        DefaultSimpleHasher
    }
}
impl<K: Hash> SimpleHasher<K> for DefaultSimpleHasher {
    fn hash(&self, t: &K) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

pub struct HashTable<K, V, H = DefaultSimpleHasher>
where
    H: SimpleHasher<K>,
    K: Hash,
{
    buckets: Vec<Vec<(K, V)>>,
    total_entries: usize,
    hasher: H,
}

impl<K, V> Default for HashTable<K, V, DefaultSimpleHasher>
where
    K: Hash,
{
    fn default() -> Self {
        let default_number_of_starting_buckets = 10;
        let mut buckets = vec![];
        for _ in 0..default_number_of_starting_buckets {
            buckets.push(vec![]);
        }
        let hasher = DefaultSimpleHasher::new();

        HashTable {
            buckets,
            total_entries: 0,
            hasher,
        }
    }
}

impl<K, V> HashTable<K, V, DefaultSimpleHasher>
where
    K: Hash + PartialEq,
{
    pub fn new() -> HashTable<K, V, DefaultSimpleHasher> {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> HashTable<K, V, DefaultSimpleHasher> {
        let mut buckets = vec![];
        for _ in 0..capacity {
            buckets.push(vec![]);
        }

        HashTable {
            buckets,
            total_entries: 0,
            hasher: DefaultSimpleHasher::new(),
        }
    }
}

impl<K, V, H> HashTable<K, V, H>
where
    K: Hash + PartialEq,
    H: SimpleHasher<K>,
{
    pub fn with_hasher(hasher: H) -> HashTable<K, V, H> {
        let mut buckets: Vec<Vec<(K, V)>> = vec![];
        for _ in 0..10 {
            buckets.push(vec![]);
        }

        HashTable {
            buckets,
            total_entries: 0,
            hasher,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        // check if this key is being used
        let hash = self.hasher.hash(&k);
        let bucket_index = hash as usize % self.buckets.len();
        let mut to_remove = None;
        for (pos, (ek, _)) in self.buckets[bucket_index].iter().enumerate() {
            if ek == &k {
                // we are using a value for this key that needs to be replaced
                to_remove = Some(pos);
                break;
            }
        }
        match to_remove {
            Some(index) => {
                let (_, ov) = mem::replace(&mut self.buckets[bucket_index][index], (k, v));
                Some(ov)
            }
            None => {
                self._insert(k, v, hash);
                None
            }
        }
    }

    fn _insert(&mut self, k: K, v: V, hash: u64) -> &mut V {
        // first check if we need to prepare for capacity changes
        let new_load_factor = (self.total_entries + 1) as f64 / self.buckets.len() as f64;
        if new_load_factor > 0.75 {
            let mut new_buckets = vec![];
            let extended_number_of_buckets = self.buckets.len() * 2;
            for _ in 0..extended_number_of_buckets {
                new_buckets.push(vec![]);
            }

            for mut bucket in self.buckets.drain(..) {
                for (ek, ev) in bucket.drain(..) {
                    let hash = self.hasher.hash(&ek);
                    let new_bucket_index = hash as usize % new_buckets.len();
                    new_buckets[new_bucket_index].push((ek, ev));
                }
            }

            self.buckets = new_buckets;
        }

        // then add the new item (give up ownership of input v late so we can easily access the value for returning)
        let bucket_index = hash as usize % self.buckets.len();
        self.buckets[bucket_index].push((k, v));
        self.total_entries += 1;
        let len = self.buckets[bucket_index].len();
        let (_, v) = &mut self.buckets[bucket_index][len - 1];
        v
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let hash = self.hasher.hash(k);
        let bucket_index = hash as usize % self.buckets.len();
        for (ek, v) in &self.buckets[bucket_index] {
            if ek == k {
                return Some(v);
            }
        }
        None
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        let hash = self.hasher.hash(k);
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

    pub fn entry(&mut self, k: K) -> Entry<'_, K, V, H> {
        if self.get(&k).is_some() {
            Entry::Occupied { ht: self, k }
        } else {
            Entry::Vacant { ht: self, k }
        }
    }

    pub fn into_keys(self) -> Keys<K> {
        let mut keys = vec![];
        for b in self.buckets {
            for (k, _) in b {
                keys.push(k);
            }
        }
        Keys { inner: keys }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hasher.hash(key);
        let bucket_index = hash as usize % self.buckets.len();
        let bucket_iter = self.buckets[bucket_index].iter_mut().enumerate();
        let mut index = None;
        for (i, (ek, _)) in bucket_iter {
            if ek == key {
                index = Some(i);
            }
        }

        index.map(|i| {
            let (_, rv) = self.buckets[bucket_index].swap_remove(i);
            rv
        })
    }
}

pub struct Keys<K> {
    inner: Vec<K>,
}

impl<K> IntoIterator for Keys<K> {
    type Item = K;

    type IntoIter = IntoIter<K>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, K> IntoIterator for &'a Keys<K> {
    type Item = &'a K;

    type IntoIter = slice::Iter<'a, K>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

pub enum Entry<'a, K, V, H>
where
    K: Hash,
    H: SimpleHasher<K>,
{
    Occupied {
        ht: &'a mut HashTable<K, V, H>,
        k: K,
    },
    Vacant {
        ht: &'a mut HashTable<K, V, H>,
        k: K,
    },
}

impl<'a, K, V, H> Entry<'a, K, V, H>
where
    K: PartialEq + Hash,
    H: SimpleHasher<K>,
{
    pub fn or_insert(self, v: V) -> &'a mut V {
        match self {
            Entry::Occupied { k, ht } => {
                let e = ht.get_mut(&k);
                e.unwrap()
            }
            Entry::Vacant { k, ht } => {
                let hash = ht.hasher.hash(&k);
                ht._insert(k, v, hash)
            }
        }
    }
}

pub struct HashTableIterator<'a, K, V> {
    elements_iterator: slice::Iter<'a, (K, V)>,
    buckets_iterator: slice::Iter<'a, Vec<(K, V)>>,
}

impl<'a, K: Hash, V, H: SimpleHasher<K>> IntoIterator for &'a HashTable<K, V, H> {
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
            elements_iterator,
            buckets_iterator,
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
                self.elements_iterator = elements_iterator;
                self.next()
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hash;

    use crate::{HashTable, SimpleHasher};

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
        struct SillyHasher;
        impl<K> SimpleHasher<K> for SillyHasher
        where
            K: Hash,
        {
            fn hash(&self, _: &K) -> u64 {
                0
            }
        }

        // The SillyHasher hashes keys to a constant value of 0. We store all entries in the zeroth bucket.
        // This test uses the SillyHasher to force collisions to occur so we can assert that all key-value pairs
        // are addressable individually even when all key hashes collide.
        let mut hash_table = HashTable::with_hasher(SillyHasher {});
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
        ];

        for user in users {
            hash_table.insert(user.name.to_string(), user);
            assert_eq!(hash_table.capacity(), 9);
        }

        hash_table.insert(
            "caine".into(),
            User {
                name: "caine".to_string(),
                age: 22,
            },
        );

        assert_ne!(hash_table.capacity(), 9);
        assert!(hash_table.capacity() > 9);
        assert_eq!(hash_table.capacity(), 18);

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
        let mut hash_table = HashTable::new();

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

    #[test]
    fn test_insert_with_same_key() {
        let mut hash_table = HashTable::new();

        let g = User {
            name: "not gedalia!".to_string(),
            age: 27,
        };

        let user = hash_table.insert("gedalia", g);
        assert_eq!(user, None); // first time inersting this key so None gets returned since no old value to return

        let old_g = User {
            name: "gedalia".to_string(),
            age: 127,
        };

        let old_user = hash_table.insert("gedalia", old_g).unwrap(); // second time using key so we should overwrite entry's value and return old value
        assert_eq!(old_user.name, "not gedalia!");
        assert_eq!(old_user.age, 27);
    }

    #[test]
    fn test_into_keys() {
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

        let keys = hash_table.into_keys();

        for k in &keys {
            let found = users.binary_search_by(|u| u.name.cmp(&k));
            assert!(found.is_ok());
        }

        for k in keys {
            let found = users.binary_search_by(|u| u.name.cmp(&k));
            assert!(found.is_ok());
        }
    }

    #[test]
    fn test_remove() {
        let mut hash_table = HashTable::new();

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
        ];

        users.sort();

        for user in users {
            hash_table.insert(user.name.to_string(), user);
        }

        let ov = hash_table.remove(&"gedalia".into());

        assert_eq!(
            ov,
            Some(User {
                name: "gedalia".to_string(),
                age: 27,
            })
        );

        let ov = hash_table.remove(&"no_one".into());

        assert_eq!(ov, None)
    }
}
