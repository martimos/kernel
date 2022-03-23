use alloc::collections::VecDeque;

pub struct LruCache<V> {
    max_size: usize,
    data: VecDeque<V>,
}

impl<V> LruCache<V> {
    pub fn new(size: usize) -> Self {
        Self {
            max_size: size,
            data: VecDeque::with_capacity(size),
        }
    }

    pub fn find<P>(&mut self, predicate: P) -> Option<&V>
    where
        P: FnMut(&V) -> bool,
    {
        if let Some(position) = self.data.iter().position(predicate) {
            let item = self.data.remove(position).unwrap();
            self.data.push_front(item);
            return Some(&self.data[0]);
        }
        None
    }

    pub fn insert(&mut self, item: V) {
        if self.data.len() >= self.max_size {
            let _ = self.data.pop_back(); // evict the last element
        }
        self.data.push_front(item);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::VecDeque;

    use crate::collection::lru::LruCache;

    #[test_case]
    fn test_lru_new_is_empty() {
        let lru = LruCache::<u8>::new(10);
        assert_eq!(VecDeque::from([]), lru.data);
    }

    #[test_case]
    fn test_lru_insert() {
        let mut lru = LruCache::<u8>::new(10);
        lru.insert(0);
        lru.insert(1);
        lru.insert(2);
        lru.insert(2);
        lru.insert(3);
        assert_eq!(VecDeque::from([3, 2, 2, 1, 0]), lru.data);
    }

    #[test_case]
    fn test_lru_insert_with_evict() {
        let mut lru = LruCache::<u8>::new(10);
        for i in 0_u8..10 {
            lru.insert(i);
        }
        assert_eq!(VecDeque::from([9, 8, 7, 6, 5, 4, 3, 2, 1, 0]), lru.data);
        let &item = lru.find(|&v| v == 4).unwrap();
        assert_eq!(4, item);
        assert_eq!(VecDeque::from([4, 9, 8, 7, 6, 5, 3, 2, 1, 0]), lru.data);
    }
}
