//use crate::softdevice::SoftdeviceApp;
use ector::{Actor, Address, Inbox};
use embassy::time::{Duration, Instant};
use embassy_nrf::gpio::{AnyPin, Input};
use futures::future::{select, Either};
use futures::pin_mut;

#[derive(Clone, Copy, Debug)]
pub enum Action {
    A,
    B,
    C,
    D,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Start,
    Stop,
    Increase,
    Decrease,
}

#[derive(Clone, Copy, Debug)]
pub struct ActionEvent {
    pub action: Action,
    pub event: Event,
}

#[derive(Clone, Copy, Debug)]
pub enum ControlEvent {
    Next,
    Prev,
}

pub struct ControlButtons<H>
where
    H: TryFrom<ControlEvent> + 'static,
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
    H: TryFrom<ControlEvent> + 'static,
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

#[ector::actor]
impl<H> Actor for ControlButtons<H>
where
    H: TryFrom<ControlEvent> + 'static,
{
    type Message<'m> = ();

    async fn on_mount<M>(&mut self, _: Address<Self::Message<'m>>, _inbox: M)
    where
        M: Inbox<Self::Message<'m>>,
    {
        loop {
            let f1 = pushed(&mut self.buttons.0);
            let f2 = pushed(&mut self.buttons.1);

            pin_mut!(f1);
            pin_mut!(f2);

            match select(f1, f2).await {
                Either::Left(_) => {
                    defmt::info!("Button 1");
                    if let Ok(event) = H::try_from(ControlEvent::Next) {
                        let _ = self.handler.notify(event);
                    }
                }
                Either::Right(_) => {
                    defmt::info!("Button 2");
                    if let Ok(event) = H::try_from(ControlEvent::Prev) {
                        let _ = self.handler.notify(event);
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
