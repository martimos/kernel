use alloc::collections::VecDeque;
use core::fmt::Debug;
use core::ops::{Index, IndexMut};

#[derive(Debug, Eq, PartialEq)]
pub struct DeltaQueue<T> {
    data: VecDeque<Node<T>>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Node<T> {
    pub value: usize,
    pub elem: T,
}

impl<T> Node<T> {
    pub fn new(value: usize, elem: T) -> Self {
        Self { value, elem }
    }
}

impl<T> Default for DeltaQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DeltaQueue<T> {
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, value: usize, elem: T) {
        let mut total_value = 0;

        for i in 0..self.data.len() {
            let found = &self.data[i];

            if total_value + found.value >= value {
                // insert at this position
                let node_value = value - total_value;
                self.data.insert(i, Node::new(node_value, elem));

                if self.len() > i + 1 {
                    // if there is a next element, adapt its value
                    self.data[i + 1].value -= node_value;
                }
                return;
            }

            total_value += found.value;
        }
        self.data.push_back(Node::new(value - total_value, elem));
    }

    pub fn front(&self) -> Option<&Node<T>> {
        self.data.front()
    }

    pub fn front_mut(&mut self) -> Option<&mut Node<T>> {
        self.data.front_mut()
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.data.pop_front().map(|n| n.elem)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T> Index<usize> for DeltaQueue<T> {
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl<T> IndexMut<usize> for DeltaQueue<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;

    #[test_case]
    fn test_simple_insert() {
        let mut q = DeltaQueue::new();
        assert_eq!(0, q.len());

        q.insert(0, "hello");
        assert_eq!(1, q.len());
    }

    #[test_case]
    fn test_insert_in_order() {
        let mut q = DeltaQueue::new();

        q.insert(3, "three");
        q.insert(5, "five");
        q.insert(9, "nine");

        assert_eq!(
            vec![
                Node::new(3, "three"),
                Node::new(2, "five"),
                Node::new(4, "nine"),
            ],
            Vec::from(q.data)
        );
    }

    #[test_case]
    fn test_insert_reverse_order() {
        let mut q = DeltaQueue::new();

        q.insert(9, "nine");
        q.insert(5, "five");
        q.insert(3, "three");

        assert_eq!(
            vec![
                Node::new(3, "three"),
                Node::new(2, "five"),
                Node::new(4, "nine"),
            ],
            Vec::from(q.data)
        );
    }

    #[test_case]
    fn test_insert_no_order() {
        let mut q = DeltaQueue::new();

        q.insert(3, "three");
        q.insert(6, "six");
        q.insert(5, "five");
        q.insert(9, "nine");
        q.insert(4, "four");

        assert_eq!(
            vec![
                Node::new(3, "three"),
                Node::new(1, "four"),
                Node::new(1, "five"),
                Node::new(1, "six"),
                Node::new(3, "nine"),
            ],
            Vec::from(q.data)
        );
    }

    #[test_case]
    fn test_insert_duplicates() {
        let mut q = DeltaQueue::new();

        q.insert(3, "three");
        q.insert(5, "five");
        q.insert(5, "five");
        q.insert(5, "five");
        q.insert(6, "six");
        q.insert(9, "nine");

        assert_eq!(
            vec![
                Node::new(3, "three"),
                Node::new(2, "five"),
                Node::new(0, "five"),
                Node::new(0, "five"),
                Node::new(1, "six"),
                Node::new(3, "nine"),
            ],
            Vec::from(q.data)
        );
    }

    #[test_case]
    fn test_insert_duplicates_different_values() {
        let mut q = DeltaQueue::new();

        q.insert(5, "five1");
        q.insert(5, "five2");
        q.insert(5, "five3");

        assert_eq!(
            vec![
                Node::new(5, "five3"),
                Node::new(0, "five2"),
                Node::new(0, "five1"),
            ],
            Vec::from(q.data)
        );
    }

    #[test_case]
    fn test_insert_many() {
        let limit = 1000;
        let mut q = DeltaQueue::with_capacity(limit);

        for i in (0..limit).rev() {
            q.insert(i + 1, i);
            assert_eq!(limit - i, q.len());
        }

        for i in 0..limit {
            let front = &q.data[0];
            assert_eq!(1, front.value);
            assert_eq!(i, front.elem);
            assert_eq!(Some(i), q.pop_front());
        }
    }
}
