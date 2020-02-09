use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub struct HashTable<V: Clone> {
    buckets: Vec<Option<V>>
}

impl<V: Clone> HashTable<V> {
    pub fn new() -> HashTable<V> {
        HashTable { buckets: vec![None; 10] }
    }

    fn get_bucket_index<K: std::hash::Hash>(&self, k: &K) -> usize {
        let hash = calculate_hash(k); 
        println!("hash computed: {}", hash);
        let bucket_index = hash as usize % self.buckets.capacity();
        println!("bucket index computed: {}", bucket_index);
        bucket_index
    }

    pub fn insert<K: std::hash::Hash>(&mut self, k: K, v: V) -> () {
        let bucket_index = self.get_bucket_index(&k);
        self.buckets[bucket_index] = Some(v);
    }



    pub fn get<K: std::hash::Hash>(&self, k: K) -> &Option<V> {
        let bucket_index = self.get_bucket_index(&k);
        &self.buckets[bucket_index]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn insert_key_value_pair() {
        use crate::HashTable;
        
        #[derive(Clone, Debug, PartialEq)]
        struct User { name: String, age: i32 }
        
        let mut hash_table = HashTable::new();
        hash_table.insert(
            "gedalia", 
            User { name: "gedalia".to_string(), age: 27 }
        );

        let result = hash_table.get("gedalia");
        let expected_result = &Some(User { name: "gedalia".to_string(), age: 27 });

        assert_eq!(result, expected_result);
    }
}
