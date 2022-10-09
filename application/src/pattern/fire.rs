use crate::pattern::Context;
use crate::{MyNeoPixel, DEFAULT_SPEED};
use drogue_device::drivers::led::neopixel::{filter::Filter, rgb::Rgb8};
use embassy_time::Instant;
use num::pow;

pub struct Fire<const N: usize>;

impl<const N: usize> Fire<N> {
    const MAX_GREEN: f64 = 100.0;
    const MIN_GREEN: f64 = 10.0;

    pub fn new(_: &mut [Rgb8; N]) -> Self {
        Self
    }

    pub async fn tick<F: Filter<Rgb8, 3>>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        ctx: Context,
        f: &mut F,
    ) {
        let speed = 2f64
            / (pow(
                2,
                (DEFAULT_SPEED - ctx.speed.clamp(0, DEFAULT_SPEED)) as usize,
            ) as f64);

        let now = (Instant::now().as_millis() / ctx.speed as u64) as f64 * speed;

        let mut s1 = now;
        let mut s2 = now;

        for i in 0..N {
            s1 += 1.0;
            s2 -= 1.0;

            let g = ((libm::cos(s1 / 6.0) + 1.0) / 2.0 * Self::MAX_GREEN) - Self::MIN_GREEN;
            let brightness = ((libm::sin(s2 / 3.0) + 1.0) / 2.0 * 0.75) + 0.25 /* 25%-100% */;

            let r = (255.0 * brightness).clamp(0.0, 255.0) as u8;
            let g = (g * brightness + Self::MIN_GREEN).clamp(0.0, 255.0) as u8;
            pixels[i] = Rgb8::new(r, g, 0);
        }

        neopixel.set_with_filter(&pixels, f).await.ok();
    }
}
