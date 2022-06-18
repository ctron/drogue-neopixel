use drogue_device::actors::led::LedMessage;
use ector::Address;

pub struct StatefulLed {
    led: Address<LedMessage>,
    state: bool,
}

impl StatefulLed {
    pub fn new(led: Address<LedMessage>, state: bool) -> Self {
        Self { led, state }
    }

    pub async fn on(&mut self) {
        self.led.notify(LedMessage::On).await;
        self.state = true;
    }

    pub async fn off(&mut self) {
        self.led.notify(LedMessage::Off).await;
        self.state = false;
    }

    pub fn is_on(&mut self) -> bool {
        self.state
    }
}

//FIXME: impl<L> Copy for StatefulLed<L> where L: Actor<Message<'static> = LedMessage> + 'static {}

impl Clone for StatefulLed {
    fn clone(&self) -> Self {
        Self {
            led: self.led.clone(),
            state: self.state,
        }
    }
}
