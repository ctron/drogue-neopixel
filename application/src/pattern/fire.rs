use crate::MyNeoPixel;
use drogue_device::drivers::led::neopixel::{filter::Filter, rgb::Rgb8};

pub struct Fire<const N: usize> {
    state: f64,
}

impl<const N: usize> Fire<N> {
    const MAX_GREEN: f64 = 100.0;

    pub fn new(_: &mut [Rgb8; N]) -> Self {
        Self { state: 0.0 }
    }

    pub async fn tick<F: Filter<Rgb8, 3>>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        f: &mut F,
    ) {
        self.state += 1.0;

        let mut s1 = self.state;
        let mut s2 = self.state;

        for i in 0..N {
            s1 += 1.0;
            s2 -= 1.0;

            let g = (libm::cos(s1 / 6.0) + 1.0) / 2.0 * Self::MAX_GREEN;
            let brightness = ((libm::sin(s2 / 3.0) + 1.0) / 2.0 * 0.75) + 0.25 /* 25%-100% */;
            let r = (255.0 * brightness).clamp(0.0, 255.0) as u8;
            let g = (g * brightness).clamp(0.0, 255.0) as u8;
            pixels[i] = Rgb8::new(r, g, 0);
        }

        neopixel.set_with_filter(&pixels, f).await.ok();
    }
}
