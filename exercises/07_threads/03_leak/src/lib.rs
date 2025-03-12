// TODO: Given a vector of integers, leak its heap allocation.
//  Then split the resulting static slice into two halves and
//  sum each half in a separate thread.
//  Hint: check out `Vec::leak`.

use std::thread::{self, JoinHandle};

pub fn sum(v: Vec<i32>) -> i32 {
    let v = v.leak();
    let mid = v.len() / 2;
    let left = &v[..mid];
    let right = &v[mid..];
    let left_handle: JoinHandle<i32> = thread::spawn(|| left.iter().sum());
    let right_handle: JoinHandle<i32> = thread::spawn(|| right.iter().sum());
    left_handle.join().unwrap() + right_handle.join().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(sum(vec![]), 0);
    }

    #[test]
    fn one() {
        assert_eq!(sum(vec![1]), 1);
    }

    #[test]
    fn five() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn nine() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]), 45);
    }

    #[test]
    fn ten() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]), 55);
    }
}
