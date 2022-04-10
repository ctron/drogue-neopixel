use crate::{Controller, ModeDiscriminants, MyNeoPixel};
use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy::time::Ticker;
use futures::future::{select, Either};
use futures::{pin_mut, StreamExt};

pub struct Runner<const N: usize> {
    pub ticker: Ticker,
    pub neopixel: MyNeoPixel<N>,
}

#[derive(Copy, Clone)]
pub enum Msg {
    Toggle,
    SetMode(ModeDiscriminants),
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
                                Msg::Toggle => {
                                    controller.next();
                                }
                                Msg::SetMode(mode) => {
                                    controller.mode(*mode);
                                }
                            }
                        }
                    }
                    Either::Right((_, d)) => {
                        controller.tick(&mut self.neopixel).await;
                    }
                }
            }
        }
    }
}
