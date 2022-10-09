#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use ector::ActorContext;
use embassy_nrf::config::Config;
use embassy_nrf::interrupt::Priority;
use embassy_time::{Duration, Timer};

const NUM_LEDS: usize = 60;

use drogue_device::drivers::led::neopixel::rgb::NeoPixelRgb;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use futures::future::{select, Either};
use futures::pin_mut;

//mod fmt;

#[cfg(feature = "panic-probe")]
use panic_probe as _;

#[cfg(feature = "defmt-rtt")]
use defmt_rtt as _;

#[cfg(feature = "panic-reset")]
use panic_reset as _;

#[cfg(feature = "log")]
mod logger;

#[cfg(feature = "ble")]
mod app;
#[cfg(feature = "ble")]
mod gatt;

mod board;
mod control;
mod controller;
//mod led;
mod runner;
//mod softdevice;
mod pattern;
mod watchdog;

#[cfg(feature = "ble")]
use app::*;

use board::*;
use runner::*;
//use softdevice::*;
use controller::*;
use watchdog::*;

// Application must run at a lower priority than softdevice
fn config() -> Config {
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy_executor::main]
//#[embassy::main]
async fn main(s: embassy_executor::Spawner) {
    let p = embassy_nrf::init(Default::default());

    // Setup burrboard peripherals
    static BOARD: BurrBoard = BurrBoard::new();

    let mut buttons = (
        Input::new(p.P0_27.degrade(), Pull::Up),
        Input::new(p.P0_06.degrade(), Pull::Up),
        Input::new(p.P0_26.degrade(), Pull::Up),
        Input::new(p.P0_08.degrade(), Pull::Up),
    );

    let mut user_led = Output::new(p.P1_10.degrade(), Level::Low, OutputDrive::Standard);
    #[cfg(feature = "ble")]
    let enable_ble = enable_ble(&mut buttons.0, &mut user_led).await;

    let ap = BOARD.mount(
        s,
        BoardPeripherals {
            buttons,
            //neopixel: defmt::unwrap!(NeoPixelRgb::<'_, _, NUM_LEDS>::new(p.PWM0, p.P1_08)),
            neopixel: NeoPixelRgb::<'_, _, NUM_LEDS>::new(p.PWM0, p.P1_08).unwrap(),
        },
    );

    // Launch the softdevice
    #[cfg(feature = "ble")]
    if enable_ble {
        info!("Enable BLE");
        user_led.set_high();

        static LED: Forever<Output<'static, AnyPin>> = Forever::new();
        LED.put(user_led);

        static APP: Forever<App> = Forever::new();
        let app = APP.put(App::enable(s, "Neopixel"));
        app.mount(s, &ap);
    }

    // Launch watchdog
    static WATCHDOG: ActorContext<Watchdog> = ActorContext::new();
    WATCHDOG.mount(s, Watchdog(Duration::from_secs(2)));

    defmt::info!("Application started");
}

#[allow(unused)]
#[allow(unused_variables)]
pub fn log_stack(file: &'static str) {
    let _u: u32 = 1;
    let _uptr: *const u32 = &_u;
    defmt::info!("[{}] SP: 0x{:?}", file, &_uptr);
}

async fn enable_ble(
    button: &mut Input<'static, AnyPin>,
    _led: &mut Output<'static, AnyPin>,
) -> bool {
    // startup led

    defmt::info!(
        "Button 1 - high: {}, low: {}",
        button.is_high(),
        button.is_low()
    );

    if button.is_low() {
        defmt::info!("Button 1 is pressed, waiting ...");

        let time = Timer::after(Duration::from_secs(1));
        let button = button.wait_for_high();

        pin_mut!(button);

        match select(time, button).await {
            Either::Left(_) => {
                // timeout
                true
            }
            Either::Right(_) => {
                // button release
                false
            }
        }
    } else {
        false
    }
}
