use embassy::executor::Spawner;
use nrf_softdevice::{raw, Flash, Softdevice};

use crate::board::*;

pub struct SoftdeviceApp {
    sd: &'static Softdevice,
}

impl SoftdeviceApp {
    pub fn enable(s: Spawner, device_name: &'static str) -> SoftdeviceApp {
        let config = nrf_softdevice::Config {
            clock: Some(raw::nrf_clock_lf_cfg_t {
                source: raw::NRF_CLOCK_LF_SRC_RC as u8,
                rc_ctiv: 4,
                rc_temp_ctiv: 2,
                accuracy: 7,
            }),
            conn_gap: Some(raw::ble_gap_conn_cfg_t {
                conn_count: 6,
                event_length: 24,
            }),
            conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
            gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
                attr_tab_size: 32768,
            }),
            gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
                adv_set_count: 1,
                periph_role_count: 3,
                central_role_count: 1,
                central_sec_count: 1,
                _bitfield_1: Default::default(),
            }),
            gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
                p_value: device_name.as_ptr() as *const u8 as _,
                current_len: device_name.len() as u16,
                max_len: device_name.len() as u16,
                write_perm: unsafe { core::mem::zeroed() },
                _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                    raw::BLE_GATTS_VLOC_STACK as u8,
                ),
            }),
            ..Default::default()
        };
        let sd = Softdevice::enable(&config);
        //s.spawn(softdevice_task(sd)).unwrap();

        Self { sd }
    }

    pub fn flash(&self) -> Flash {
        Flash::take(self.sd)
    }

    pub fn mount(&'static self, s: Spawner, p: &BoardPeripherals) {}
}

/*
#[embassy::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

 */
