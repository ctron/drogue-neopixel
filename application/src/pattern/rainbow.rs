use crate::MyNeoPixel;
use drogue_device::drivers::led::neopixel::{Filter, Rgb8};
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

pub struct Rainbow<const N: usize> {}

impl<const N: usize> Rainbow<N> {
    pub fn new(pixels: &mut [Rgb8; N]) -> Self {
        for i in 0..N {
            let v = (360f32 / (N as f32)) * (i as f32);

            let color = Hsv::new(v, 1.0, 1.0);
            let color: Rgb = color.into_color();
            pixels[i] = color.into_pixel();
        }

        Self {}
    }

    pub async fn tick<F: Filter>(
        &self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        f: &mut F,
    ) {
        pixels.rotate_left(1);
        neopixel.set_with_filter(&pixels, f).await.ok();
    }
}

pub struct RainbowPart<const N: usize, const MAX: usize> {
    state: usize,
}

impl<const N: usize, const MAX: usize> RainbowPart<N, MAX> {
    pub fn new(_pixels: &mut [Rgb8; N]) -> Self {
        Self { state: 0 }
    }

    pub async fn tick<F: Filter>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        f: &mut F,
    ) {
        for i in 0..N {
            let v = (360f32 / (MAX as f32)) * ((i + self.state) as f32);

            let color = Hsv::new(v, 1.0, 1.0);
            let color: Rgb = color.into_color();
            pixels[i] = color.into_pixel();
        }

        neopixel.set_with_filter(&pixels, f).await.ok();

        self.state += 1;
        if self.state > MAX {
            self.state = 0;
        }
    }
}
