use crate::{
    control::{Action, ControlEvent, Event},
    pattern::ModeDiscriminants,
    Controller, MyNeoPixel,
};
use ector::{Actor, Address, Inbox};
use embassy::time::{Duration, Ticker};
use futures::{
    future::{select, Either},
    pin_mut, StreamExt,
};

pub struct Runner<const N: usize> {
    pub neopixel: MyNeoPixel<N>,
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
    Lighter,
    Darker,
    ResetBrightness,
}

const INITIAL_SPEED_MS: u64 = 250u64;

#[ector::actor]
impl<const N: usize> Actor for Runner<N> {
    type Message<'m> = Msg;

    async fn on_mount<M>(&mut self, _: Address<Self::Message<'m>>, mut inbox: M)
    where
        M: Inbox<Self::Message<'m>>,
    {
        let mut speed_ms = INITIAL_SPEED_MS;
        let mut ticker = Ticker::every(Duration::from_millis(speed_ms));
        let mut controller = Controller::<N>::new();

        loop {
            let next = inbox.next();
            let delay = ticker.next();

            pin_mut!(next);
            pin_mut!(delay);

            match select(next, delay).await {
                Either::Left((m, _)) => {
                    defmt::info!("Message: {}", defmt::Debug2Format(&m));
                    match m {
                        Msg::Next => {
                            controller.next();
                        }
                        Msg::Prev => {
                            controller.prev();
                        }
                        Msg::SetMode(mode) => {
                            controller.mode(mode);
                        }
                        Msg::StartSleep(duration) => {
                            controller.start_sleep(duration);
                        }
                        Msg::StopSleep => {
                            controller.stop_sleep();
                        }
                        Msg::Faster => {
                            speed_ms = faster(speed_ms);
                            defmt::info!("Speed: {} ms", speed_ms);
                            ticker = Ticker::every(Duration::from_millis(speed_ms));
                        }
                        Msg::Slower => {
                            speed_ms = slower(speed_ms);
                            defmt::info!("Speed: {} ms", speed_ms);
                            ticker = Ticker::every(Duration::from_millis(speed_ms));
                        }
                        Msg::ResetSpeed => {
                            speed_ms = INITIAL_SPEED_MS;
                            defmt::info!("Speed: {} ms", speed_ms);
                            ticker = Ticker::every(Duration::from_millis(speed_ms));
                        }
                        Msg::Lighter => {
                            controller.lighter();
                        }
                        Msg::Darker => {
                            controller.darker();
                        }
                        Msg::ResetBrightness => {
                            controller.reset_brightness();
                        }
                    }
                }
                Either::Right((_, _d)) => {
                    controller.tick(&mut self.neopixel).await;
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

            // C
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

            // ignore
            _ => Err(()),
        }
    }
}
