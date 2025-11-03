use std::collections::VecDeque;

pub struct FilterIteratorButKeepContext<
    T,
    IteratorType,
    FilterFunction: Fn(&T) -> bool,
    PlaceholderFunction: Fn(usize) -> T,
> where
    IteratorType: Iterator<Item = T>,
{
    iterator: IteratorType,
    filter_function: FilterFunction,
    placeholder_function: PlaceholderFunction,
    context: usize,
    queue: VecDeque<T>,
    consecutive_boring_values: usize,
    next_valid_value: Option<T>,
    number_truncated: usize,
}

impl<T, IteratorType, FilterFunction: Fn(&T) -> bool, PlaceholderFunction: Fn(usize) -> T>
    FilterIteratorButKeepContext<T, IteratorType, FilterFunction, PlaceholderFunction>
where
    IteratorType: Iterator<Item = T>,
{
    pub fn new(
        iterator: IteratorType,
        filter_function: FilterFunction,
        placeholder_function: PlaceholderFunction,
        context: usize,
    ) -> Self {
        let queue = VecDeque::<T>::with_capacity(context);

        Self {
            iterator,
            filter_function,
            placeholder_function,
            context,
            queue,
            consecutive_boring_values: context + 1,
            next_valid_value: None,
            number_truncated: 0,
        }
    }

    fn pop_from_queue_or_next_valid(&mut self) -> Option<T> {
        if let Some(next_valid_value) = self.next_valid_value.take() {
            if self.number_truncated > 0 {
                let old_truncated = self.number_truncated;
                self.number_truncated = 0;
                self.next_valid_value = Some(next_valid_value);
                return Some((self.placeholder_function)(old_truncated));
            }
            if let Some(value) = self.queue.pop_back() {
                self.next_valid_value = Some(next_valid_value);
                Some(value)
            } else {
                Some(next_valid_value)
            }
        } else {
            None
        }
    }

    fn push_to_queue(&mut self, value: T) {
        self.queue.push_front(value);
        if self.queue.len() > self.context {
            self.number_truncated += 1;
            self.queue.truncate(self.context);
        }
    }
}

impl<T, IteratorType, FilterFunction: Fn(&T) -> bool, PlaceholderFunction: Fn(usize) -> T> Iterator
    for FilterIteratorButKeepContext<T, IteratorType, FilterFunction, PlaceholderFunction>
where
    IteratorType: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.pop_from_queue_or_next_valid() {
            return Some(value);
        }

        if let Some(next_value) = self.iterator.next() {
            if (self.filter_function)(&next_value) {
                self.consecutive_boring_values = 0;
                return Some(next_value);
            } else {
                self.consecutive_boring_values += 1;

                if self.consecutive_boring_values <= self.context {
                    return Some(next_value);
                } else {
                    self.push_to_queue(next_value);

                    while let Some(value) = self.iterator.next() {
                        if (self.filter_function)(&value) {
                            self.consecutive_boring_values = 0;
                            self.next_valid_value = Some(value);

                            return self.pop_from_queue_or_next_valid();
                        } else {
                            self.push_to_queue(value);
                        }
                    }
                    // We've reached the end of the iterator
                    self.queue.clear();
                    return None;
                }
            }
        }
        None
    }
}
