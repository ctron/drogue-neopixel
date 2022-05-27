use crate::control::ControlEvent;
use crate::{pattern::ModeDiscriminants, Controller, MyNeoPixel};
use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
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

impl<const N: usize> Actor for Runner<N> {
    type Message<'m> = Msg;
    type OnMountFuture<'m, M> = impl Future<Output = ()> + 'm
    where M: 'm + Inbox<Self>;

    fn on_mount<'m, M>(
        &'m mut self,
        _: Address<Self>,
        inbox: &'m mut M,
    ) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            let mut controller = Controller::<N>::new();

            loop {
                let next = inbox.next();
                let delay = self.ticker.next();

                pin_mut!(next);
                pin_mut!(delay);

                match select(next, delay).await {
                    Either::Left((r, _)) => {
                        if let Some(mut m) = r {
                            match m.message() {
                                Msg::Next => {
                                    controller.next();
                                }
                                Msg::Prev => {
                                    controller.prev();
                                }
                                Msg::SetMode(mode) => {
                                    controller.mode(*mode);
                                }
                                Msg::StartSleep(duration) => {
                                    controller.start_sleep(*duration);
                                }
                                Msg::StopSleep => {
                                    controller.stop_sleep();
                                }
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
}

impl TryFrom<ControlEvent> for Msg {
    type Error = ();

    fn try_from(value: ControlEvent) -> Result<Self, Self::Error> {
        defmt::info!("Control button: {0}", defmt::Debug2Format(&value));
        match value {
            ControlEvent::Next => Ok(Msg::Next),
            ControlEvent::Prev => Ok(Msg::Prev),
        }
    }
}
