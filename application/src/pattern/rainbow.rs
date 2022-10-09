use crate::pattern::Context;
use crate::MyNeoPixel;
use core::num::NonZeroUsize;
use drogue_device::drivers::led::neopixel::{filter::Filter, rgb::Rgb8};
use embassy::time::{Duration, Instant};
use palette::rgb::Rgb;
use palette::{Hsv, IntoColor};

pub trait IntoPixel {
    fn into_pixel(self) -> Rgb8;
}

impl IntoPixel for Rgb {
    fn into_pixel(self) -> Rgb8 {
        Rgb8::new(
            (255f32 * self.red).clamp(0f32, 255f32) as u8,
            (255f32 * self.green).clamp(0f32, 255f32) as u8,
            (255f32 * self.blue).clamp(0f32, 255f32) as u8,
        )
    }
}

impl IntoPixel for Hsv {
    fn into_pixel(self) -> Rgb8 {
        let color: Rgb = self.into_color();
        color.into_pixel()
    }
}

pub struct Rainbow<const N: usize> {
    last_shift: Instant,
}

impl<const N: usize> Rainbow<N> {
    pub fn new(pixels: &mut [Rgb8; N]) -> Self {
        for i in 0..N {
            let v = (360f32 / (N as f32)) * (i as f32);

            let color = Hsv::new(v, 1.0, 1.0);
            let color: Rgb = color.into_color();
            pixels[i] = color.into_pixel();
        }

        Self {
            last_shift: Instant::now(),
        }
    }

    pub async fn tick<F: Filter<Rgb8, 3>>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        ctx: Context,
        f: &mut F,
    ) {
        if let Some(num) = self.need_update(ctx) {
            // shift
            pixels.rotate_left(num.into());
            // and render
            neopixel.set_with_filter(&pixels, f).await.ok();
        }
    }

    fn need_update(&mut self, ctx: Context) -> Option<NonZeroUsize> {
        // expected length of ticks
        let tick_len_ms = Self::tick_len_ms(ctx.speed);
        // delta to last shift, in ms
        let delta = (Instant::now() - self.last_shift).as_millis();

        // number of ticks expected from last, rounded down
        let ticks = delta / tick_len_ms; // ignoring remainder

        // increment to last + number of ticks we process now
        self.last_shift += Duration::from_millis(ticks * tick_len_ms);

        // return number of ticks/shifts
        NonZeroUsize::new(ticks as usize)
    }

    fn tick_len_ms(speed: u8) -> u64 {
        if speed < 127 {
            (250f32 / 128f32 * (speed as f32)) as u64
        } else {
            250u64 + ((750f32) / 128f32 * (speed as f32)) as u64
        }
    }
}

pub struct RainbowPart<const N: usize, const MAX: usize>;

impl<const N: usize, const MAX: usize> RainbowPart<N, MAX> {
    pub fn new(_pixels: &mut [Rgb8; N]) -> Self {
        Self
    }

    pub async fn tick<F: Filter<Rgb8, 3>>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        ctx: Context,
        f: &mut F,
    ) {
        let now = (Instant::now().as_millis()) as f32;

        let add = (360f32 / 2.0) / (MAX as f32);
        let offset = now / ctx.speed as f32 / 10f32;

        for i in 0..N {
            let v = add * i as f32 + offset;

            let color = Hsv::new(v, 1.0, 1.0);
            let color: Rgb = color.into_color();
            pixels[i] = color.into_pixel();
        }

        neopixel.set_with_filter(&pixels, f).await.ok();
    }
}
