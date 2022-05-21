//use crate::softdevice::SoftdeviceApp;
use core::future::Future;
use drogue_device::{Actor, Address, Inbox};
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
                self.buttons.3.wait_for_rising_edge().await;

                defmt::info!("Button");
                if let Ok(event) = H::Message::try_from(ControlEvent::Next) {
                    let _ = self.handler.notify(event);
                }
            }
        }
    }
}
