//use crate::softdevice::SoftdeviceApp;
use ector::{Actor, Address, Inbox};
use embassy::time::{Duration, Timer};
use embassy::util::{select4, Either4};
use embassy_nrf::gpio::{AnyPin, Input};

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum Action {
    A,
    B,
    C,
    D,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum Event {
    Start,
    Stop,
    Increase,
    Decrease,
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub struct ControlEvent {
    pub action: Action,
    pub event: Event,
}

impl From<(Action, Event)> for ControlEvent {
    fn from((action, event): (Action, Event)) -> Self {
        Self { action, event }
    }
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
            let action = action_started([
                &mut self.buttons.0,
                &mut self.buttons.1,
                &mut self.buttons.2,
                &mut self.buttons.3,
            ])
            .await;

            defmt::debug!("Start {}", action);
            self.send((action, Event::Start));

            match action {
                Action::A => {
                    run_action(
                        &mut self.handler,
                        action,
                        &mut self.buttons.0,
                        &mut self.buttons.3,
                        [&mut self.buttons.1, &mut self.buttons.2],
                    )
                    .await;
                }
                Action::B => {
                    run_action(
                        &mut self.handler,
                        action,
                        &mut self.buttons.1,
                        &mut self.buttons.2,
                        [&mut self.buttons.3, &mut self.buttons.0],
                    )
                    .await;
                }
                Action::C => {
                    run_action(
                        &mut self.handler,
                        action,
                        &mut self.buttons.2,
                        &mut self.buttons.1,
                        [&mut self.buttons.3, &mut self.buttons.0],
                    )
                    .await;
                }
                Action::D => {
                    run_action(
                        &mut self.handler,
                        action,
                        &mut self.buttons.3,
                        &mut self.buttons.0,
                        [&mut self.buttons.1, &mut self.buttons.2],
                    )
                    .await;
                }
            }

            defmt::debug!("Stop {}", action);
            self.send((action, Event::Stop));
        }
    }
}

impl<H> ControlButtons<H>
where
    H: TryFrom<ControlEvent> + 'static,
{
    fn send<E>(&mut self, event: E)
    where
        E: Into<ControlEvent>,
    {
        if let Ok(event) = H::try_from(event.into()) {
            self.handler.try_notify(event).ok();
        }
    }
}

async fn run_action<H>(
    address: &mut Address<H>,
    action: Action,
    activator: &mut Input<'static, AnyPin>,
    increment: &mut Input<'static, AnyPin>,
    decrement: [&mut Input<'static, AnyPin>; 2],
) where
    H: TryFrom<ControlEvent> + 'static,
{
    let [d1, d2] = decrement;

    loop {
        match select4(
            stopped(activator),
            pushed(increment),
            pushed(d1),
            pushed(d2),
        )
        .await
        {
            Either4::First(_) => {
                // Stopped
                return;
            }
            Either4::Second(_) => {
                // Increment
                if let Ok(event) = H::try_from(ControlEvent::from((action, Event::Increase))) {
                    address.try_notify(event).ok();
                }
            }
            Either4::Third(_) | Either4::Fourth(_) => {
                // Decrement
                if let Ok(event) = H::try_from(ControlEvent::from((action, Event::Decrease))) {
                    address.try_notify(event).ok();
                }
            }
        }
    }
}

async fn action_started(input: [&mut Input<'static, AnyPin>; 4]) -> Action {
    let [a, b, c, d] = input;
    match select4(started(a), started(b), started(c), started(d)).await {
        Either4::First(_) => Action::A,
        Either4::Second(_) => Action::B,
        Either4::Third(_) => Action::C,
        Either4::Fourth(_) => Action::D,
    }
}

async fn started(input: &mut Input<'static, AnyPin>) {
    loop {
        input.wait_for_high().await;
        input.wait_for_low().await;
        Timer::after(DEBOUNCE_DELAY).await;
        if input.is_low() {
            // still low, ok let's go...
            return;
        }
    }
}

async fn stopped(input: &mut Input<'static, AnyPin>) {
    // FIXME: need to debounce too
    input.wait_for_high().await;
}

const DEBOUNCE_DELAY: Duration = Duration::from_millis(50);
const REPEAT_DELAY: Duration = Duration::from_millis(250);

async fn pushed(input: &mut Input<'static, AnyPin>) {
    if input.is_low() {
        // already pressed
        loop {
            Timer::after(REPEAT_DELAY).await;
            if input.is_low() {
                // still pressed, fire
                return;
            } else {
                // start waiting, but don't fire
                break;
            }
        }
    }

    // button not pressed or no longer pressed, we wait
    loop {
        input.wait_for_low().await;
        //let now = Instant::now();
        Timer::after(DEBOUNCE_DELAY).await;
        if input.is_low() {
            // still pressed
            return;
        }
    }
}

/*
 * Wait for any button:
 *   * As long as the button is held, watch other 3
 */
