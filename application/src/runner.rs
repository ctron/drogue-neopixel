use crate::{
    control::{Action, ControlEvent, Event},
    pattern::ModeDiscriminants,
    Controller, MyNeoPixel,
};
use drogue_device::drivers::led::neopixel::rgb;
use ector::{Actor, Address, Inbox};
use embassy_time::{Duration, Ticker};
use futures::{
    future::{select, Either},
    pin_mut, StreamExt,
};

pub struct Runner<const N: usize> {
    pub neopixel: MyNeoPixel<N>,
    ticker: Ticker,
    controller: Controller<N>,
}

#[derive(Copy, Clone, Debug)]
pub enum Msg {
    Next,
    Prev,
    Faster,
    Slower,
    ResetSpeed,
    SetMode(ModeDiscriminants),
    StartSleep(Duration),
    StopSleep,
    SleepConfig(Event),
    Lighter,
    Darker,
    ResetBrightness,
}

pub enum State {
    Running,
    ConfigureSleep,
}

const TICKER_SPEED: Duration = Duration::from_millis(50);

#[ector::actor]
impl<const N: usize> Actor for Runner<N> {
    type Message<'m> = Msg;

    async fn on_mount<M>(&mut self, _: Address<Self::Message<'m>>, mut inbox: M)
    where
        M: Inbox<Self::Message<'m>>,
    {
        let mut state = State::Running;

        loop {
            match state {
                State::Running => {
                    state = self.running(&mut inbox).await;
                }
                State::ConfigureSleep => {
                    state = self.configure_sleep(&mut inbox).await;
                }
            }
        }
    }
}

impl<const N: usize> Runner<N> {
    pub fn new(neopixel: MyNeoPixel<N>) -> Self {
        let ticker = Ticker::every(TICKER_SPEED);
        let controller = Controller::<N>::new();
        Self {
            neopixel,
            ticker,
            controller,
        }
    }

    async fn configure_sleep<M: Inbox<Msg>>(&mut self, inbox: &mut M) -> State {
        defmt::info!("Begin sleep config");

        let current_ms = self
            .controller
            .remaining_sleep_ms()
            .unwrap_or(SleepConfig::DEFAULT_MS);

        let mut cfg = SleepConfig { current_ms };

        loop {
            cfg.render(&mut self.neopixel).await;

            let msg = inbox.next().await;
            defmt::info!("Event: {0}", defmt::Debug2Format(&msg));
            match msg {
                Msg::SleepConfig(Event::Stop) => {
                    defmt::info!("Stop sleep config");
                    if cfg.current_ms > 0.0 {
                        let duration = cfg.current_ms as u64;
                        defmt::info!("Start sleep: {}s", duration / 1000);
                        // start
                        self.controller.start_sleep(Duration::from_millis(duration));
                    } else {
                        // stop
                        defmt::info!("Stop sleep mode");
                        self.controller.stop_sleep();
                    }
                    return State::Running;
                }
                Msg::SleepConfig(Event::Increase) => {
                    cfg.increase();
                }
                Msg::SleepConfig(Event::Decrease) => {
                    cfg.decrease();
                }
                _ => {}
            }
        }
    }

    async fn running<M: Inbox<Msg>>(&mut self, inbox: &mut M) -> State {
        loop {
            let next = inbox.next();
            let delay = self.ticker.next();

            pin_mut!(next);
            pin_mut!(delay);

            match select(next, delay).await {
                Either::Left((m, _)) => {
                    defmt::info!("Message: {}", defmt::Debug2Format(&m));
                    match m {
                        Msg::Next => {
                            self.controller.next();
                        }
                        Msg::Prev => {
                            self.controller.prev();
                        }
                        Msg::SetMode(mode) => {
                            self.controller.mode(mode);
                        }
                        Msg::StartSleep(duration) => {
                            self.controller.start_sleep(duration);
                        }
                        Msg::StopSleep => {
                            self.controller.stop_sleep();
                        }
                        Msg::SleepConfig(Event::Reset) => {
                            self.controller.stop_sleep();
                        }
                        Msg::SleepConfig(Event::Start) => {
                            defmt::info!("Start sleep config");
                            return State::ConfigureSleep;
                        }
                        Msg::SleepConfig(_) => {
                            // ignore
                        }
                        Msg::Faster => {
                            self.controller.faster();
                        }
                        Msg::Slower => {
                            self.controller.slower();
                        }
                        Msg::ResetSpeed => {
                            self.controller.reset_speed();
                        }
                        Msg::Lighter => {
                            self.controller.lighter();
                        }
                        Msg::Darker => {
                            self.controller.darker();
                        }
                        Msg::ResetBrightness => {
                            self.controller.reset_brightness();
                        }
                    }
                }
                Either::Right((_, _d)) => {
                    self.controller.tick(&mut self.neopixel).await;
                }
            }
        }
    }
}

fn faster(current: u64) -> u64 {
    let dec = amount(current);
    if current > dec {
        current - dec
    } else {
        1
    }
}

fn slower(current: u64) -> u64 {
    let inc = amount(current);
    if current + inc < u32::MAX as u64 {
        current + inc
    } else {
        u32::MAX as u64
    }
}

fn amount(current: u64) -> u64 {
    use num::traits::Float;
    (current as f64 * 0.1).ceil() as u64
}

impl TryFrom<ControlEvent> for Msg {
    type Error = ();

    fn try_from(value: ControlEvent) -> Result<Self, Self::Error> {
        defmt::info!("Control button: {0}", defmt::Debug2Format(&value));
        match value {
            // A - pattern
            ControlEvent {
                action: Action::A,
                event: Event::Increase,
            } => Ok(Msg::Next),
            ControlEvent {
                action: Action::A,
                event: Event::Decrease,
            } => Ok(Msg::Prev),
            ControlEvent {
                action: Action::A,
                event: Event::Reset,
            } => Ok(Msg::SetMode(ModeDiscriminants::Off.next())),

            // B - speed
            ControlEvent {
                action: Action::B,
                event: Event::Increase,
            } => Ok(Msg::Faster),
            ControlEvent {
                action: Action::B,
                event: Event::Decrease,
            } => Ok(Msg::Slower),
            ControlEvent {
                action: Action::B,
                event: Event::Reset,
            } => Ok(Msg::ResetSpeed),

            // C - brightness
            ControlEvent {
                action: Action::C,
                event: Event::Increase,
            } => Ok(Msg::Lighter),
            ControlEvent {
                action: Action::C,
                event: Event::Decrease,
            } => Ok(Msg::Darker),
            ControlEvent {
                action: Action::C,
                event: Event::Reset,
            } => Ok(Msg::ResetBrightness),

            // D - sleep config
            ControlEvent {
                action: Action::D,
                event,
            } => Ok(Msg::SleepConfig(event)),

            // ignore
            _ => Err(()),
        }
    }
}

struct SleepConfig {
    pub current_ms: f64,
}

impl SleepConfig {
    const MAX_MS: f64 = 60.0 * 60.0 * 1000.0; /* 1h */
    const DEFAULT_MS: f64 = 15.0 * 60.0 * 1000.0; /* 15m */
    const STEP_MS: f64 = 5.0 * 60.0 * 1000.0; /* 5m */

    pub async fn render<const N: usize>(&self, pixels: &mut MyNeoPixel<N>) {
        let num = ((self.current_ms / Self::MAX_MS) * N as f64) as usize;

        let i = itertools::chain(
            itertools::repeat_n(&rgb::RED, num),
            itertools::repeat_n(&rgb::BLACK, N - num),
        );

        let _ = pixels.set_from_iter(i).await;
    }

    pub fn increase(&mut self) {
        self.current_ms += Self::STEP_MS;
        if self.current_ms > Self::MAX_MS {
            self.current_ms = Self::MAX_MS;
        }
    }

    pub fn decrease(&mut self) {
        if self.current_ms >= Self::STEP_MS {
            self.current_ms -= Self::STEP_MS;
        } else {
            self.current_ms = 0.0;
        }
    }
}
