use core::future::Future;
use drogue_device::drivers::led::neopixel::*;
use drogue_device::{Actor, Address, Inbox};
use embassy::time::{Duration, Ticker, Timer};
use embassy_nrf::peripherals::PWM0;
use futures::future::{select, Either};
use futures::{pin_mut, StreamExt};

pub struct Runner<const N: usize> {
    pub ticker: Ticker,
    pub neopixel: NeoPixel<'static, PWM0, N>,
}

pub const YELLOW: Rgb8 = Rgb8::new(0xFF, 0xFF, 0x00);

#[derive(Clone)]
pub enum Msg {
    Toggle,
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
        let mut pixels1 = [BLACK; N];
        for i in 0..N {
            pixels1[i] = if (i >> 1) % 2 == 0 { BLUE } else { YELLOW };
        }
        let mut pixels2 = [BLACK; N];
        /*
        for i in 0..N {
            pixels2[i] = match (i >> 1) % 3 {
                0 => GREEN,
                1 => WHITE,
                2 => RED,
                _ => BLACK,
            };
        }*/
        for i in 0..N {
            pixels2[i] = match (i >> 1) % 3 {
                0 => BLACK,
                1 => RED,
                2 => YELLOW,
                _ => BLACK,
            };
        }

        async move {
            let mut direction = true;
            let mut pixels = &mut pixels1;

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
                                    direction = !direction;
                                    pixels = match direction {
                                        true => &mut pixels1,
                                        false => &mut pixels2,
                                    };
                                }
                            }
                        }
                    }
                    Either::Right((_, d)) => {
                        if direction {
                            pixels.rotate_right(1);
                        } else {
                            pixels.rotate_left(1);
                        }
                        self.neopixel
                            .set_with_filter(&pixels, &mut Brightness(16))
                            .await;
                    }
                }
            }
        }
    }
}
