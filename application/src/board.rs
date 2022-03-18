use drogue_device::{
    actors::button::ButtonPressed, actors::dfu::*, actors::flash::*, actors::led::Led,
    ActorContext, Address,
};
use embassy::executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Level, OptionalPin, Output, OutputDrive, Pin, Pull};
use embassy_nrf::peripherals::TIMER1;
use embassy_nrf::{interrupt, Peripherals};

pub type UserLed = Led<Output<'static, AnyPin>>;
pub type MyNeoPixel = NeoPixel<TIMER1>;

pub struct BurrBoard {
    user_led: ActorContext<UserLed>,
    //flash: ActorContext<SharedFlash<Flash>>,
    //dfu: ActorContext<FirmwareManager<SharedFlashHandle<Flash>>>,

    //control: ActorContext<ControlButton>,
}

pub struct BoardPeripherals {
    pub user_led: Address<UserLed>,

    //pub flash: Address<SharedFlash<Flash>>,

    //pub dfu: Address<FirmwareManager<SharedFlashHandle<Flash>>>,
    pub neopixel: MyNeoPixel,
}

impl BurrBoard {
    pub const fn new() -> Self {
        Self {
            user_led: ActorContext::new(),
            // flash: ActorContext::new(),
            // dfu: ActorContext::new(),
            //control: ActorContext::new(),
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

        let timer = embassy_nrf::timer::Timer::new_awaitable(p.TIMER1, interrupt::take!(TIMER1));

        let neopixel_pin = p.P0_07.degrade();
        let neopixel = NeoPixel::new(
            timer,
            Output::new(neopixel_pin, Level::Low, OutputDrive::Standard),
        );

        BoardPeripherals {
            user_led,
            //flash,
            //dfu,
            neopixel,
        }
    }
}
