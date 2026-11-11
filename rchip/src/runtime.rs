use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::emulator::Emulator;

const MAX_ELAPSED: Duration = Duration::from_millis(50);
const PAUSE_SLEEP: Duration = Duration::from_millis(5);
const RUN_SLEEP: Duration = Duration::from_millis(1);

fn limit_elapsed(elapsed: Duration) -> Duration {
    if elapsed > MAX_ELAPSED {
        Duration::ZERO
    } else {
        elapsed
    }
}

pub fn run<S, P>(
    emulator: &mut Emulator,
    mut should_stop: S,
    mut is_paused: P,
) -> Result<(), String>
where
    S: FnMut() -> bool,
    P: FnMut() -> bool,
{
    let mut last_iter = Instant::now();

    loop {
        if should_stop() {
            return Ok(());
        }

        if is_paused() {
            sleep(PAUSE_SLEEP);
            last_iter = Instant::now();
            continue;
        }

        let now = Instant::now();
        let elapsed = limit_elapsed(now.duration_since(last_iter));

        emulator.advance(elapsed)?;

        last_iter = now;
        sleep(RUN_SLEEP);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_immediately() {
        let mut emulator = Emulator::new(60);

        let result = run(&mut emulator, || true, || false);

        assert!(result.is_ok());
    }

    #[test]
    fn stops_while_paused() {
        let mut emulator = Emulator::new(60);
        let mut stop_checks = 0;

        let result = run(
            &mut emulator,
            || {
                stop_checks += 1;
                stop_checks >= 3
            },
            || true,
        );

        assert!(result.is_ok());
        assert_eq!(stop_checks, 3);
    }

    #[test]
    fn stop_condition_is_checked_before_execution() {
        let mut emulator = Emulator::new(60);
        let mut stop_checks = 0;
        let mut pause_checks = 0;

        emulator
            .load_program(&[0x6A, 0x42, 0x7A, 0x05, 0xAA, 0x00])
            .unwrap();

        let result = run(
            &mut emulator,
            || {
                stop_checks += 1;
                stop_checks >= 2
            },
            || {
                pause_checks += 1;
                false
            },
        );

        assert_eq!(result, Ok(()));
        assert_eq!(stop_checks, 2);
        assert_eq!(pause_checks, 1);
    }

    #[test]
    fn elapsed_below_limit_is_preserved() {
        let elapsed = Duration::from_millis(25);
        assert_eq!(limit_elapsed(elapsed), elapsed);
    }

    #[test]
    fn elapsed_at_limit_is_preserved() {
        assert_eq!(limit_elapsed(MAX_ELAPSED), MAX_ELAPSED);
    }

    #[test]
    fn elapsed_above_limit_is_discarded() {
        assert_eq!(limit_elapsed(Duration::from_millis(100)), Duration::ZERO);
    }
}
