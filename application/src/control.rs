//use crate::softdevice::SoftdeviceApp;
use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
use embassy::time::{Duration, Instant};
use embassy_nrf::gpio::{AnyPin, Input};
use futures::future::{select, Either};
use futures::pin_mut;

#[derive(Clone, Copy, Debug)]
pub enum ControlEvent {
    Next,
}

pub struct ControlButtons<H>
where
    H: Actor + 'static,
{
    handler: Address<H>,
    buttons: (
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
        Input<'static, AnyPin>,
    ),
}

impl<H> ControlButtons<H>
where
    H: Actor + 'static,
{
    pub fn new(
        handler: Address<H>,
        buttons: (
            Input<'static, AnyPin>,
            Input<'static, AnyPin>,
            Input<'static, AnyPin>,
            Input<'static, AnyPin>,
        ),
    ) -> Self {
        Self { handler, buttons }
    }
}

impl<H> Actor for ControlButtons<H>
where
    H: Actor + 'static,
    H::Message<'static>: TryFrom<ControlEvent>,
{
    type OnMountFuture<'m, M> = impl Future<Output = ()> + 'm
    where M: 'm + Inbox<Self>;

    fn on_mount<'m, M>(&'m mut self, _: Address<Self>, _: &'m mut M) -> Self::OnMountFuture<'m, M>
    where
        M: Inbox<Self> + 'm,
        Self: 'm,
    {
        async move {
            loop {
                let f1 = pushed(&mut self.buttons.0);
                let f2 = pushed(&mut self.buttons.1);

                pin_mut!(f1);
                pin_mut!(f2);

                match select(f1, f2).await {
                    Either::Left(_) => {
                        defmt::info!("Button 1");
                        if let Ok(event) = H::Message::try_from(ControlEvent::Next) {
                            let _ = self.handler.notify(event);
                        }
                    }
                    Either::Right(_) => {
                        defmt::info!("Button 2");
                    }
                }
            }
        }
    }
}

async fn pushed(input: &mut Input<'static, AnyPin>) {
    loop {
        input.wait_for_high().await;
        input.wait_for_low().await;
        let now = Instant::now();
        input.wait_for_high().await;
        if Instant::now() - now > Duration::from_millis(100) {
            return;
        }
    }
}
