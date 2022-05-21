#![no_std]
#![no_main]
#![macro_use]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use drogue_device::ActorContext;
use embassy::time::Duration;
use embassy::util::Forever;
use embassy_nrf::config::Config;
use embassy_nrf::interrupt::Priority;
use embassy_nrf::Peripherals;

const NUM_LEDS: usize = 60;

#[cfg(feature = "log")]
use embassy_nrf::{gpio::NoPin, interrupt, uarte};

mod fmt;

#[cfg(feature = "panic-probe")]
use panic_probe as _;

#[cfg(feature = "nrf-softdevice-defmt-rtt")]
use nrf_softdevice_defmt_rtt as _;

#[cfg(feature = "log")]
mod logger;

#[cfg(not(feature = "defmt"))]
use panic_reset as _;

mod app;
mod board;
mod control;
mod controller;
mod gatt;
mod led;
mod runner;
//mod softdevice;
mod pattern;
mod watchdog;

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

#[embassy::main(config = "config()")]
//#[embassy::main]
async fn main(s: embassy::executor::Spawner, p: Peripherals) {
    static APP: Forever<App> = Forever::new();
    let app = APP.put(App::enable(s, "Neopixel"));

    #[cfg(feature = "log")]
    {
        logger::init(uarte::Uarte::new(
            p.UARTE0,
            interrupt::take!(UARTE0_UART0),
            p.P0_24,
            p.P0_25,
            NoPin,
            NoPin,
            Default::default(),
        ));
    }

    // Setup burrboard peripherals
    static BOARD: BurrBoard = BurrBoard::new();

    let ap = BOARD.mount(s, /*app, */ p);

    // Launch the application
    app.mount(s, &ap);

    // Launch watchdog
    static WATCHDOG: ActorContext<Watchdog> = ActorContext::new();
    WATCHDOG.mount(s, Watchdog(Duration::from_secs(2)));

    info!("Application started");

    //let mut neopixel = defmt::unwrap!(NeoPixel::new(p.PWM0, p.P0_16));
    //let mut neopixel = defmt::unwrap!(NeoPixel::<'_, _, 1>::new(p.PWM0, p.P0_16));

    //let mut neopixel = ap.neopixel;

    //let dir = 1;

    //let mut pixels = [BLUE, BLUE, YELLOW, YELLOW, BLUE, BLUE, YELLOW, YELLOW];

    /*
    loop {
        if let Ok(f) = ap.user_led.request(LedMessage::Toggle) {
            f.await;
        }
    }*/
}

#[allow(unused)]
#[allow(unused_variables)]
pub fn log_stack(file: &'static str) {
    let _u: u32 = 1;
    let _uptr: *const u32 = &_u;
    info!("[{}] SP: 0x{:?}", file, &_uptr);
}
