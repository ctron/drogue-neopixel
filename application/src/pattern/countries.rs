use crate::pattern::YELLOW;
use crate::MyNeoPixel;
use drogue_device::drivers::led::neopixel::{
    filter::Filter,
    rgb::{Rgb8, BLACK, BLUE, RED},
};

pub struct UA<const N: usize>;

impl<const N: usize> UA<N> {
    pub fn new(pixels: &mut [Rgb8; N]) -> Self {
        for i in 0..N {
            pixels[i] = if (i >> 1) % 2 == 0 { BLUE } else { YELLOW };
        }
        Self
    }

    pub async fn tick<F: Filter<Rgb8, 3>>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        f: &mut F,
    ) {
        pixels.rotate_right(1);
        neopixel.set_with_filter(&pixels, f).await.ok();
    }
}

pub struct DE<const N: usize>;

impl<const N: usize> DE<N> {
    pub fn new(pixels: &mut [Rgb8; N]) -> Self {
        for i in 0..N {
            pixels[i] = match (i >> 1) % 3 {
                0 => BLACK,
                1 => RED,
                2 => YELLOW,
                _ => BLACK,
            };
        }
        Self
    }

    pub async fn tick<F: Filter<Rgb8, 3>>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        f: &mut F,
    ) {
        pixels.rotate_left(1);
        neopixel.set_with_filter(&pixels, f).await.ok();
    }
}
