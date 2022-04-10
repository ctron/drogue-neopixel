use crate::MyNeoPixel;
use drogue_device::drivers::led::neopixel::{Brightness, Rgb8, BLACK, BLUE, RED};
use strum::EnumDiscriminants;

pub const YELLOW: Rgb8 = Rgb8::new(0xFF, 0xFF, 0x00);

#[derive(EnumDiscriminants)]
pub enum Mode<const N: usize> {
    Off,
    UA(UA<N>),
    DE(DE<N>),
}

impl ModeDiscriminants {
    pub fn next(&self) -> Self {
        match self {
            Self::Off => Self::UA,
            Self::UA => Self::DE,
            Self::DE => Self::UA,
        }
    }

    pub fn new<const N: usize>(&self, pixels: &mut [Rgb8; N]) -> Mode<N> {
        match self {
            Self::Off => Mode::Off,
            Self::UA => Mode::UA(UA::new(pixels)),
            Self::DE => Mode::DE(DE::new(pixels)),
        }
    }
}

impl<const N: usize> Mode<N> {
    async fn tick(&mut self, pixels: &mut [Rgb8; N], neopixel: &mut MyNeoPixel<N>) {
        match self {
            Self::Off => {}
            Self::UA(pattern) => pattern.tick(pixels, neopixel).await,
            Self::DE(pattern) => pattern.tick(pixels, neopixel).await,
        }
    }
}

pub struct UA<const N: usize>;

impl<const N: usize> UA<N> {
    pub fn new(pixels: &mut [Rgb8; N]) -> Self {
        for i in 0..N {
            pixels[i] = if (i >> 1) % 2 == 0 { BLUE } else { YELLOW };
        }
        Self
    }

    async fn tick(&mut self, pixels: &mut [Rgb8; N], neopixel: &mut MyNeoPixel<N>) {
        pixels.rotate_right(1);
        neopixel
            .set_with_filter(&pixels, &mut Brightness(16))
            .await
            .ok();
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

    async fn tick(&mut self, pixels: &mut [Rgb8; N], neopixel: &mut MyNeoPixel<N>) {
        pixels.rotate_left(1);
        neopixel
            .set_with_filter(&pixels, &mut Brightness(16))
            .await
            .ok();
    }
}

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
