use std::collections::VecDeque;

pub struct FilterIteratorButKeepContext<T: Copy, IteratorType, FilterFunction: Fn(T) -> bool>
where
    IteratorType: Iterator<Item = T>,
{
    iterator: IteratorType,
    filter_function: FilterFunction,
    placeholder_value: T,
    context: usize,
    queue: VecDeque<T>,
    consecutive_values: usize,
    next_valid_value: Option<T>,
    truncated: bool,
    has_ever_had_valid_value: bool,
}

impl<T: Copy, IteratorType, FilterFunction: Fn(T) -> bool>
    FilterIteratorButKeepContext<T, IteratorType, FilterFunction>
where
    IteratorType: Iterator<Item = T>,
{
    pub fn new(
        iterator: IteratorType,
        filter_function: FilterFunction,
        placeholder_value: T,
        context: usize,
    ) -> Self {
        let queue = VecDeque::<T>::with_capacity(context);

        Self {
            iterator,
            filter_function,
            placeholder_value,
            context,
            queue,
            consecutive_values: context + 1,
            next_valid_value: None,
            truncated: false,
            has_ever_had_valid_value: false,
        }
    }

    fn pop_from_queue_or_next_valid(&mut self) -> Option<T> {
        if let Some(next_valid_value) = self.next_valid_value {
            if self.truncated {
                self.truncated = false;
                return Some(self.placeholder_value);
            }
            if let Some(value) = self.queue.pop_back() {
                return Some(value);
            } else {
                self.next_valid_value = None;
                self.has_ever_had_valid_value = true;
                return Some(next_valid_value);
            }
        } else {
            return None;
        }
    }
}

impl<T: Copy, IteratorType, FilterFunction: Fn(T) -> bool> Iterator
    for FilterIteratorButKeepContext<T, IteratorType, FilterFunction>
where
    IteratorType: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.pop_from_queue_or_next_valid() {
            return Some(value);
        }

        if let Some(next_value) = self.iterator.next() {
            if (self.filter_function)(next_value) {
                self.consecutive_values = 0;
                return Some(next_value);
            } else {
                self.consecutive_values += 1;

                if self.consecutive_values <= self.context {
                    return Some(next_value);
                } else {
                    while let Some(value) = self.iterator.next() {
                        if (self.filter_function)(value) {
                            self.consecutive_values = 0;
                            self.next_valid_value = Some(value);

                            return self.pop_from_queue_or_next_valid();
                        } else {
                            self.queue.push_front(value);
                            if self.queue.len() > self.context {
                                self.truncated |= self.has_ever_had_valid_value;
                                self.queue.truncate(self.context);
                            }
                        }
                    }
                }

                return self.pop_from_queue_or_next_valid();
            }
        }
        None
    }
}
