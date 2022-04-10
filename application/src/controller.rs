use crate::pattern::{Mode, ModeDiscriminants};
use crate::MyNeoPixel;
use drogue_device::drivers::led::neopixel::{Rgb8, BLACK};

pub struct Controller<const N: usize> {
    pixels: [Rgb8; N],
    mode: Mode<N>,
}

impl<const N: usize> Controller<N> {
    pub fn new() -> Self {
        let mut result = Self {
            mode: Mode::Off,
            pixels: [BLACK; N],
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

    pub async fn tick(&mut self, neopixel: &mut MyNeoPixel<N>) {
        self.mode.tick(&mut self.pixels, neopixel).await;
    }
}
