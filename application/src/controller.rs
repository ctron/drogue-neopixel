use crate::pattern::{Mode, ModeDiscriminants};
use crate::MyNeoPixel;
use drogue_device::drivers::led::neopixel::{
    filter::Brightness,
    rgb::{Rgb8, BLACK},
};
use embassy::time::{Duration, Instant};
use num::{cast, traits::Float, NumCast};

pub struct Controller<const N: usize> {
    pixels: [Rgb8; N],
    mode: Mode<N>,
    sleep: Option<Sleep<u8>>,
    brightness: u8,
}

const INITIAL_BRIGHTNESS: u8 = 16;

impl<const N: usize> Controller<N> {
    pub fn new() -> Self {
        let mut result = Self {
            mode: Mode::Off,
            pixels: [BLACK; N],
            sleep: None,
            brightness: INITIAL_BRIGHTNESS,
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
            Brightness(self.brightness)
        };

        self.mode.tick(&mut self.pixels, neopixel, &mut f).await;
    }

    pub fn start_sleep(&mut self, duration: Duration) {
        self.sleep = Some(Sleep::now(0, 16, duration))
    }

    pub fn stop_sleep(&mut self) {
        self.sleep = None;
    }

    pub fn remaining_sleep_ms(&self) -> Option<f64> {
        self.sleep.as_ref().map(|s| s.remaining_ms())
    }

    pub fn lighter(&mut self) {
        if self.brightness < u8::MAX {
            self.brightness += 1;
        }
        defmt::info!("Brightness: {}", self.brightness);
    }

    pub fn darker(&mut self) {
        if self.brightness > u8::MIN {
            self.brightness -= 1;
        }
        defmt::info!("Brightness: {}", self.brightness);
    }

    pub fn reset_brightness(&mut self) {
        self.brightness = INITIAL_BRIGHTNESS;
        defmt::info!("Brightness: {}", self.brightness);
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

    /// Get the remaining time in ms
    pub fn remaining_ms(&self) -> f64 {
        let now = Instant::now();

        let end = self.start + self.duration;
        if now >= end {
            return 0.0;
        }

        let rem = (end - now).as_millis();

        rem as f64
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
            min: 0,
            max: 16,
        };

        assert_eq!(sleep.remaining(Instant::from_secs(0)), 16);
        assert_eq!(sleep.remaining(Instant::from_secs(150)), 8);
        assert_eq!(sleep.remaining(Instant::from_secs(300)), 0);
        assert_eq!(sleep.remaining(Instant::from_secs(350)), 0);
    }
}
