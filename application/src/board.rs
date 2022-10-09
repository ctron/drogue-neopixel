use crate::control::ControlButtons;
use drogue_device::drivers::led::neopixel::rgb::NeoPixelRgb;
use ector::{ActorContext, Address};
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{AnyPin, Input},
    peripherals::PWM0,
};

use crate::{runner, Runner, NUM_LEDS};

//pub type UserLed = Led<Output<'static, AnyPin>>;
pub type MyNeoPixel<const N: usize> = NeoPixelRgb<'static, PWM0, N>;
pub type MyRunner = Runner<NUM_LEDS>;
pub type MyControlButtons = ControlButtons<runner::Msg>;

pub struct BurrBoard {
    runner: ActorContext<MyRunner, 5>,
    control: ActorContext<MyControlButtons>,
}

pub struct BoardActors {
    pub runner: Address<runner::Msg>,
    pub control: Address<()>,
}

pub struct BoardPeripherals {
    pub buttons: (
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
    ),

    pub neopixel: MyNeoPixel<NUM_LEDS>,
}

impl BurrBoard {
    pub const fn new() -> Self {
        Self {
            runner: ActorContext::new(),
            control: ActorContext::new(),
        }
    }

    pub fn mount(&'static self, s: Spawner, p: BoardPeripherals) -> BoardActors {
        let runner = self.runner.mount(s, Runner::new(p.neopixel));

        let control = self
            .control
            .mount(s, MyControlButtons::new(runner.clone(), p.buttons));

        BoardActors { runner, control }
    }
}
