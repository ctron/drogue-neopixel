use crate::control::ControlButtons;
use drogue_device::actors::button::{Button, ButtonPressed};
use drogue_device::drivers::led::neopixel::NeoPixel;
use drogue_device::{actors::led::Led, ActorContext, Address};
use embassy::executor::Spawner;
use embassy::time::{Duration, Ticker};
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::PWM0;
use embassy_nrf::Peripherals;

use crate::{runner, Runner, NUM_LEDS};

pub type UserLed = Led<Output<'static, AnyPin>>;
pub type MyNeoPixel<const N: usize> = NeoPixel<'static, PWM0, N>;
pub type MyRunner = Runner<NUM_LEDS>;
pub type MyButton = Button<Input<'static, AnyPin>, ButtonPressed<MyRunner>>;
pub type MyControlButtons = ControlButtons<MyRunner>;

pub struct BurrBoard {
    user_led: ActorContext<UserLed>,
    runner: ActorContext<MyRunner>,
    button: ActorContext<MyButton>,
    control: ActorContext<MyControlButtons>,
    //flash: ActorContext<SharedFlash<Flash>>,
    //dfu: ActorContext<FirmwareManager<SharedFlashHandle<Flash>>>,

    //control: ActorContext<ControlButton>,
}

pub struct BoardPeripherals {
    pub user_led: Address<UserLed>,

    //pub flash: Address<SharedFlash<Flash>>,

    //pub dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
    pub button: Address<MyButton>,
    pub runner: Address<MyRunner>,
    pub control: Address<MyControlButtons>,
}

impl BurrBoard {
    pub const fn new() -> Self {
        Self {
            user_led: ActorContext::new(),
            button: ActorContext::new(),
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
        p: Peripherals,
    ) -> BoardPeripherals {
        let user_led_pin = p.P1_10.degrade();

        // LED
        let user_led = self.user_led.mount(
            s,
            UserLed::new(Output::new(user_led_pin, Level::Low, OutputDrive::Standard)),
        );

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
        let neopixel = defmt::unwrap!(NeoPixel::<'_, _, NUM_LEDS>::new(p.PWM0, p.P1_08));
        let runner = self.runner.mount(
            s,
            Runner {
                neopixel,
                ticker: Ticker::every(Duration::from_millis(250)),
            },
        );

        let button = self.button.mount(
            s,
            Button::new(
                Input::new(p.P1_02.degrade(), Pull::Up),
                ButtonPressed(runner.clone(), runner::Msg::Toggle),
            ),
        );

        let control = self.control.mount(
            s,
            MyControlButtons::new(
                runner,
                (
                    Input::new(p.P0_26.degrade(), Pull::Up),
                    Input::new(p.P0_06.degrade(), Pull::Up),
                    Input::new(p.P0_08.degrade(), Pull::Up),
                    Input::new(p.P0_27.degrade(), Pull::Up),
                ),
            ),
        );

        BoardPeripherals {
            user_led,
            //flash,
            //dfu,
            runner,
            button,
            control,
        }
    }
}
