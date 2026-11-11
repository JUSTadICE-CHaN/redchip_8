use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduledEvent {
    CpuInstruction,
    TimerTick,
}

pub struct Scheduler {
    cpu_period: Duration,
    timer_period: Duration,
    cpu_remaining: Duration,
    timer_remaining: Duration,
}

impl Scheduler {
    pub fn new(freq: u16) -> Self {
        assert!(freq > 0, "CPU frequency must be greater than zero");

        let cpu_period = Duration::from_secs_f64(1.0 / freq as f64);
        let timer_period = Duration::from_secs_f64(1.0 / 60.0);

        Self {
            cpu_period,
            timer_period,
            cpu_remaining: Duration::ZERO,
            timer_remaining: timer_period,
        }
    }

    pub fn advance(&mut self, mut elapsed: Duration) -> Vec<ScheduledEvent> {
        let mut events = Vec::new();

        loop {
            let until_next_event = self.cpu_remaining.min(self.timer_remaining);

            if until_next_event > elapsed {
                self.cpu_remaining -= elapsed;
                self.timer_remaining -= elapsed;
                break;
            }

            elapsed -= until_next_event;
            self.cpu_remaining -= until_next_event;
            self.timer_remaining -= until_next_event;

            if self.cpu_remaining.is_zero() {
                events.push(ScheduledEvent::CpuInstruction);
                self.cpu_remaining += self.cpu_period;
            }

            if self.timer_remaining.is_zero() {
                events.push(ScheduledEvent::TimerTick);
                self.timer_remaining += self.timer_period;
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_freq_test() {
        let mut scheduler = Scheduler::new(60);

        let result = scheduler.advance(Duration::from_secs_f64(1.0 / 60.0));

        assert_eq!(
            result,
            vec![
                ScheduledEvent::CpuInstruction,
                ScheduledEvent::CpuInstruction,
                ScheduledEvent::TimerTick
            ]
        );
    }

    #[test]
    fn zero_elapsed_time() {
        let mut scheduler = Scheduler::new(60);

        let result = scheduler.advance(Duration::ZERO);

        assert_eq!(result, vec![ScheduledEvent::CpuInstruction]);
    }

    #[test]
    fn partial_elapsed_time() {
        let mut scheduler = Scheduler::new(60);

        let first = scheduler.advance(Duration::from_millis(10));
        let second = scheduler.advance(Duration::from_millis(7));

        assert_eq!(first, vec![ScheduledEvent::CpuInstruction]);

        assert_eq!(
            second,
            vec![ScheduledEvent::CpuInstruction, ScheduledEvent::TimerTick]
        );
    }

    #[test]
    fn less_than_one_period() {
        let mut scheduler = Scheduler::new(60);

        let result = scheduler.advance(Duration::from_secs_f64(1.0 / 70.0));

        assert_eq!(result, vec![ScheduledEvent::CpuInstruction]);
    }

    #[test]
    fn multiple_periods() {
        let mut scheduler = Scheduler::new(30);

        let result = scheduler.advance(Duration::from_secs_f64(6.0 / 60.0));

        assert_eq!(
            result,
            vec![
                ScheduledEvent::CpuInstruction,
                ScheduledEvent::TimerTick,
                ScheduledEvent::CpuInstruction,
                ScheduledEvent::TimerTick,
                ScheduledEvent::TimerTick,
                ScheduledEvent::CpuInstruction,
                ScheduledEvent::TimerTick,
                ScheduledEvent::TimerTick,
                ScheduledEvent::CpuInstruction,
            ]
        );
    }
}
