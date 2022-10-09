mod countries;
mod fire;
mod rainbow;

use crate::{
    pattern::{
        // countries::{DE, UA},
        fire::Fire,
        rainbow::{Rainbow, RainbowPart},
    },
    MyNeoPixel,
};
use drogue_device::drivers::led::neopixel::{filter::Filter, rgb::Rgb8};
use embassy::time::Duration;
use strum::{EnumDiscriminants, EnumIter, IntoEnumIterator};

pub const YELLOW: Rgb8 = Rgb8::new(0xFF, 0xFF, 0x00);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    /// the speed configuration
    pub speed: u8,
    /// the time since the last run (could be zero)
    pub delta: Duration,
}

#[derive(EnumDiscriminants, strum::IntoStaticStr)]
#[strum_discriminants(derive(EnumIter))]
pub enum Mode<const N: usize> {
    Off,
    //UA(UA<N>),
    //DE(DE<N>),
    Fire(Fire<N>),
    Rainbow(Rainbow<N>),
    RainbowPart(RainbowPart<N, 200>),
}

impl ModeDiscriminants {
    pub fn next(&self) -> Self {
        let mut it = ModeDiscriminants::iter().skip(1);

        let mut next = None;
        while let Some(i) = it.next() {
            if i == *self {
                next = it.next();
                break;
            }
        }

        next.unwrap_or(Self::Fire)
    }

    pub fn prev(&self) -> Self {
        let mut it = ModeDiscriminants::iter().skip(1).rev();

        let mut next = None;
        while let Some(i) = it.next() {
            if i == *self {
                next = it.next();
                break;
            }
        }

        next.unwrap_or(Self::RainbowPart)
    }

    pub fn new<const N: usize>(&self, pixels: &mut [Rgb8; N]) -> Mode<N> {
        match self {
            Self::Off => Mode::Off,
            //Self::UA => Mode::UA(UA::new(pixels)),
            //Self::DE => Mode::DE(DE::new(pixels)),
            Self::Rainbow => Mode::Rainbow(Rainbow::new(pixels)),
            Self::RainbowPart => Mode::RainbowPart(RainbowPart::new(pixels)),
            Self::Fire => Mode::Fire(Fire::new(pixels)),
        }
    }
}

impl<const N: usize> Mode<N> {
    pub async fn tick<F: Filter<Rgb8, 3>>(
        &mut self,
        pixels: &mut [Rgb8; N],
        neopixel: &mut MyNeoPixel<N>,
        ctx: Context,
        f: &mut F,
    ) {
        match self {
            Self::Off => {}
            //Self::UA(pattern) => pattern.tick(pixels, neopixel, f).await,
            //Self::DE(pattern) => pattern.tick(pixels, neopixel, f).await,
            Self::Rainbow(pattern) => pattern.tick(pixels, neopixel, ctx, f).await,
            Self::RainbowPart(pattern) => pattern.tick(pixels, neopixel, ctx, f).await,
            Self::Fire(pattern) => pattern.tick(pixels, neopixel, ctx, f).await,
        }
    }
}
