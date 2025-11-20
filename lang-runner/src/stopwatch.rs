use std::{
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

use common::{TimerType, Timers};
use futures_util::FutureExt;

pub fn start_stopwatch<T, Fut: Future<Output = T> + Unpin>(
    timers: Timers,
    receiver: tokio::sync::mpsc::Receiver<TimerType>,
    future: Fut,
) -> impl Future<Output = (Option<T>, Timers)> {
    Stopwatch {
        future,
        channel: receiver,
        timer: Box::pin(tokio::time::sleep(timers.judge)),
        elapsed_timers: Timers {
            run: Duration::ZERO,
            compile: Duration::ZERO,
            judge: Duration::ZERO,
        },
        remaining_timers: timers,
        current_timer: TimerType::Judge,
        start_time: Instant::now(),
    }
}

struct Stopwatch<T, Fut>
where
    Fut: Future<Output = T> + Unpin,
{
    future: Fut,
    channel: tokio::sync::mpsc::Receiver<TimerType>,
    timer: Pin<Box<tokio::time::Sleep>>,

    elapsed_timers: Timers,
    remaining_timers: Timers,

    current_timer: TimerType,
    start_time: Instant,
}

impl<T, Fut> Stopwatch<T, Fut>
where
    Fut: Future<Output = T> + Unpin,
{
    fn subtract_time(&mut self, poll_start_time: Instant) {
        let elapsed_time = poll_start_time - self.start_time;
        let current_timer = self.current_timer;
        *self.elapsed_timers.get_mut_type(current_timer) += elapsed_time;
        self.start_time = poll_start_time;
    }
}

impl<T, Fut> Future for Stopwatch<T, Fut>
where
    Fut: Future<Output = T> + Unpin,
{
    type Output = (Option<T>, Timers);

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let poll_start_time = Instant::now();

        match self.future.poll_unpin(cx) {
            Poll::Ready(r) => {
                self.subtract_time(poll_start_time);
                eprintln!(
                    "Stopwatch: Process finished normally (time since poll start: {:?})",
                    (Instant::now() - poll_start_time)
                );
                return Poll::Ready((Some(r), self.elapsed_timers));
            }
            Poll::Pending => (),
        }

        match self.timer.as_mut().poll(cx) {
            Poll::Pending => (),
            Poll::Ready(_) => {
                self.subtract_time(poll_start_time);
                eprintln!(
                    "Stopwatch: Process finished with timeout (time since poll: {:?})",
                    (Instant::now() - poll_start_time)
                );

                return Poll::Ready((None, self.elapsed_timers));
            }
        }

        match self.channel.poll_recv(cx) {
            Poll::Ready(Some(new_timer_type)) => {
                let elapsed_time = poll_start_time - self.start_time;
                self.subtract_time(poll_start_time);

                let current_timer = self.current_timer;
                let remaining_time = self.remaining_timers.get_mut_type(current_timer);
                if let Some(new_remaining_time) = remaining_time.checked_sub(elapsed_time) {
                    *remaining_time = new_remaining_time
                } else {
                    *remaining_time = Duration::ZERO;
                    return Poll::Ready((None, self.elapsed_timers));
                }

                self.current_timer = new_timer_type;
                self.timer = Box::pin(tokio::time::sleep(
                    *self.remaining_timers.get_type(new_timer_type),
                ));
            }
            Poll::Ready(None) => {
                // It shouldn't be possible to actually reach this branch I think
                self.subtract_time(poll_start_time);
                return Poll::Ready((None, self.elapsed_timers));
            }
            Poll::Pending => (),
        }

        Poll::Pending
    }
}
