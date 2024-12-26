use std::collections::VecDeque;

#[derive(Debug)]
struct TickNode<T> {
    creation_tick: u32,
    t: T,
}

pub struct TickDeque<T> {
    max_ticks: u32,
    ticks_passed: u32,
    deque: VecDeque<TickNode<T>>,
}

impl<T> TickDeque<T> {
    pub fn new(max_ticks: u32) -> TickDeque<T> {
        assert!(max_ticks > 0);

        TickDeque {
            max_ticks,
            ticks_passed: 0,
            deque: VecDeque::new(),
        }
    }

    pub fn push_back(&mut self, t: T) {
        self.deque.push_back(TickNode { creation_tick: self.ticks_passed, t });
    }

    pub fn tick(&mut self) -> Vec<T> {
        let mut expired: Vec<T> = vec![];

        self.ticks_passed += 1;

        while let Some(node) = self.deque.front() {
            if node.creation_tick + self.max_ticks == self.ticks_passed {
                expired.push(self.deque.pop_front().unwrap().t);
            } else {
                break;
            }
        }

        expired
    }
}

#[cfg(test)]
mod tick_dequeue_tests {
    use crate::tick_deque::TickDeque;

    type TickDequeType = String;

    #[test]
    fn test_empty_deque() {
        let mut td: TickDeque<TickDequeType> = TickDeque::new(1);
        assert!(td.tick().is_empty());
    }

    #[test]
    fn test_1_max_ticks() {
        let mut td: TickDeque<TickDequeType> = TickDeque::new(1);

        td.push_back("first".to_string());
        td.push_back("second".to_string());

        let expired = td.tick();
        assert_eq!(expired[0], "first");
        assert_eq!(expired[1], "second");
    }

    #[test]
    fn test_2_max_ticks() {
        let mut td: TickDeque<TickDequeType> = TickDeque::new(2);

        td.push_back("first".to_string());
        td.push_back("second".to_string());

        assert!(td.tick().is_empty());

        let expired = td.tick();
        assert_eq!(expired[0], "first");
        assert_eq!(expired[1], "second");
    }
}
