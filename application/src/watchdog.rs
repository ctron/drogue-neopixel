use ector::{Actor, Address, Inbox};
use embassy_time::{Duration, Ticker};
use futures::StreamExt;

pub struct Watchdog(pub Duration);

#[ector::actor]
impl Actor for Watchdog {
    type Message<'m> = ();

    async fn on_mount<M>(&mut self, _: Address<Self::Message<'m>>, _inbox: M)
    where
        M: Inbox<Self::Message<'m>>,
    {
        let mut ticker = Ticker::every(self.0);
        let mut handle = unsafe { embassy_nrf::wdt::WatchdogHandle::steal(0) };
        loop {
            handle.pet();
            ticker.next().await;
        }
    }
}
