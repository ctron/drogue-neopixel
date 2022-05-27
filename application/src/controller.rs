use crate::pattern::{Mode, ModeDiscriminants};
use crate::MyNeoPixel;
use drogue_device::drivers::led::neopixel::{Brightness, Rgb8, BLACK};
use embassy::time::{Duration, Instant};
use num::traits::Float;
use num::{cast, NumCast};

pub struct Controller<const N: usize> {
    pixels: [Rgb8; N],
    mode: Mode<N>,
    sleep: Option<Sleep<u8>>,
}

impl<const N: usize> Controller<N> {
    pub fn new() -> Self {
        let mut result = Self {
            mode: Mode::Off,
            pixels: [BLACK; N],
            sleep: None,
        };
        result.next();
        result
    }

    pub fn mode(&mut self, mode: ModeDiscriminants) {
        self.mode = mode.new(&mut self.pixels);
    }

    pub fn next(&mut self) {
        self.mode(ModeDiscriminants::from(&self.mode).next());
    }

    pub fn prev(&mut self) {
        self.mode(ModeDiscriminants::from(&self.mode).prev());
    }

    pub async fn tick(&mut self, neopixel: &mut MyNeoPixel<N>) {
        let mut f = if let Some(sleep) = &self.sleep {
            Brightness(sleep.remaining_now())
        } else {
            Brightness(16)
        };

        self.mode.tick(&mut self.pixels, neopixel, &mut f).await;
    }

    pub fn start_sleep(&mut self, duration: Duration) {
        self.sleep = Some(Sleep::now(0, 16, duration))
    }

    pub fn stop_sleep(&mut self) {
        self.sleep = None;
    }
}

pub struct Sleep<T>
where
    T: Copy,
{
    start: Instant,
    duration: Duration,
    min: T,
    max: T,
}

impl<T> Sleep<T>
where
    T: Copy + NumCast,
{
    pub fn now(min: T, max: T, duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            min,
            max,
        }
    }

    pub fn remaining(&self, now: Instant) -> T {
        let end = self.start + self.duration;
        if now >= end {
            return self.min;
        }

        let rem = (end - now).as_millis();
        if rem > self.duration.as_millis() {
            return self.max;
        }
        if rem <= 0 {
            return self.min;
        }

        let p = rem as f64 / self.duration.as_millis() as f64;

        cast((p * self.max.to_f64().unwrap_or_default()).round()).unwrap_or(self.min)
    }

    pub fn remaining_now(&self) -> T {
        self.remaining(Instant::now())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    //#[test]
    fn test() {
        let sleep = Sleep {
            start: Instant::from_secs(0),
            duration: Duration::from_secs(300),
            max: 16,
        };

        assert_eq!(sleep.remaining(Instant::from_secs(0)), 16);
        assert_eq!(sleep.remaining(Instant::from_secs(150)), 8);
        assert_eq!(sleep.remaining(Instant::from_secs(300)), 0);
        assert_eq!(sleep.remaining(Instant::from_secs(350)), 0);
    }
}
