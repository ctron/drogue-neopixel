use crate::control::ControlButtons;
use drogue_device::drivers::led::neopixel::rgb::NeoPixelRgb;
//use drogue_device::{actors::led::Led,};
use ector::{ActorContext, Address};
use embassy::{
    executor::Spawner,
    time::{Duration, Ticker},
};
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
    runner: ActorContext<MyRunner>,
    control: ActorContext<MyControlButtons>,
    //flash: ActorContext<SharedFlash<Flash>>,
    //dfu: ActorContext<FirmwareManager<SharedFlashHandle<Flash>>>,

    //control: ActorContext<ControlButton>,
}

pub struct BoardActors {
    //pub user_led: Address<UserLed>,

    //pub flash: Address<SharedFlash<Flash>>,

    //pub dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
    //pub button: Address<MyButton>,
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
            // flash: ActorContext::new(),
            // dfu: ActorContext::new(),
        }
    }

    pub fn mount(
        &'static self,
        s: Spawner,
        //app: &'static SoftdeviceApp,
        p: BoardPeripherals,
    ) -> BoardActors {
        // Actor for shared access to flash
        // let flash = self.flash.mount(s, SharedFlash::new(app.flash()));

        // Actor for DFU\
        /*
        let dfu = self.dfu.mount(
            s,
            FirmwareManager::new(flash.into(), embassy_boot_nrf::updater::new()),
        );

         */

        /*
        self.control.mount(
            s,
            ControlButton::new(app, Input::new(p.P1_02.degrade(), Pull::Up)),
        );
         */

        //p1.p1_08
        let runner = self.runner.mount(
            s,
            Runner {
                neopixel: p.neopixel,
                ticker: Ticker::every(Duration::from_millis(250)),
            },
        );

        /*
        let button = self.button.mount(
            s,
            Button::new(
                Input::new(p.P1_02.degrade(), Pull::Up),
                ButtonPressed(runner.clone(), runner::Msg::Next),
            ),
        )*/

        let control = self
            .control
            .mount(s, MyControlButtons::new(runner.clone(), p.buttons));

        BoardActors {
            //flash,
            //dfu,
            runner,
            //button,
            control,
        }
    }
}
