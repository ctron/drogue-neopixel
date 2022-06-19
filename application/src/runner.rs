use crate::control::{Action, ControlEvent, Event};
use crate::{pattern::ModeDiscriminants, Controller, MyNeoPixel};
use ector::{Actor, Address, Inbox};
use embassy::time::{Duration, Ticker};
use futures::{
    future::{select, Either},
    pin_mut, StreamExt,
};

pub struct Runner<const N: usize> {
    pub ticker: Ticker,
    pub neopixel: MyNeoPixel<N>,
}

#[derive(Copy, Clone)]
pub enum Msg {
    Next,
    Prev,
    SetMode(ModeDiscriminants),
    StartSleep(Duration),
    StopSleep,
}

#[ector::actor]
impl<const N: usize> Actor for Runner<N> {
    type Message<'m> = Msg;

    async fn on_mount<M>(&mut self, _: Address<Self::Message<'m>>, mut inbox: M)
    where
        M: Inbox<Self::Message<'m>>,
    {
        let mut controller = Controller::<N>::new();

        loop {
            let next = inbox.next();
            let delay = self.ticker.next();

            pin_mut!(next);
            pin_mut!(delay);

            match select(next, delay).await {
                Either::Left((m, _)) => match m {
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
                },
                Either::Right((_, _d)) => {
                    controller.tick(&mut self.neopixel).await;
                }
            }
        }
    }
}

impl TryFrom<ControlEvent> for Msg {
    type Error = ();

    fn try_from(value: ControlEvent) -> Result<Self, Self::Error> {
        defmt::info!("Control button: {0}", defmt::Debug2Format(&value));
        match value {
            ControlEvent {
                action: Action::A,
                event: Event::Increase,
            } => Ok(Msg::Next),
            ControlEvent {
                action: Action::A,
                event: Event::Decrease,
            } => Ok(Msg::Prev),
            _ => Err(()),
        }
    }
}
