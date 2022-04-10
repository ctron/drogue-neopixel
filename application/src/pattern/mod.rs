mod countries;
mod rainbow;

use crate::{
    pattern::{
        countries::{DE, UA},
        rainbow::Rainbow,
    },
    MyNeoPixel,
};
use drogue_device::drivers::led::neopixel::Rgb8;
use strum::EnumDiscriminants;

pub const YELLOW: Rgb8 = Rgb8::new(0xFF, 0xFF, 0x00);

#[derive(EnumDiscriminants)]
pub enum Mode<const N: usize> {
    Off,
    UA(UA<N>),
    DE(DE<N>),
    Rainbow(Rainbow<N>),
}

impl ModeDiscriminants {
    pub fn next(&self) -> Self {
        match self {
            Self::Off => Self::UA,
            Self::UA => Self::DE,
            Self::DE => Self::Rainbow,
            Self::Rainbow => Self::UA,
        }
    }

    pub fn new<const N: usize>(&self, pixels: &mut [Rgb8; N]) -> Mode<N> {
        match self {
            Self::Off => Mode::Off,
            Self::UA => Mode::UA(UA::new(pixels)),
            Self::DE => Mode::DE(DE::new(pixels)),
            Self::Rainbow => Mode::Rainbow(Rainbow::new(pixels)),
        }
    }
}

impl<const N: usize> Mode<N> {
    pub async fn tick(&mut self, pixels: &mut [Rgb8; N], neopixel: &mut MyNeoPixel<N>) {
        match self {
            Self::Off => {}
            Self::UA(pattern) => pattern.tick(pixels, neopixel).await,
            Self::DE(pattern) => pattern.tick(pixels, neopixel).await,
            Self::Rainbow(pattern) => pattern.tick(pixels, neopixel).await,
        }
    }
}
