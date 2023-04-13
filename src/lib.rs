use std::str::from_utf8_unchecked;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use scanner_rust::{generic_array::typenum::U768, ScannerAscii};

#[napi(object)]
pub struct Swap {
    pub total: f64,
    pub used:  f64,
    pub free:  f64,
    pub cache: f64,
}

#[napi(object)]
pub struct Mem {
    pub total:     f64,
    pub used:      f64,
    pub free:      f64,
    pub shared:    f64,
    pub buffers:   f64,
    pub cache:     f64,
    pub available: f64,
}

#[napi(object)]
pub struct Free {
    pub mem:  Mem,
    pub swap: Swap,
}

/// Get data from `/proc/meminfo`.
#[napi(ts_return_type="Record<string, number>")]
pub fn meminfo(env: Env) -> Result<Object> {
    let mut sc: ScannerAscii<_, U768> = match ScannerAscii::scan_path2("/proc/meminfo") {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(err.to_string())),
    };

    let mut obj = env.create_object().unwrap();

    loop {
        let label = match sc.next_raw() {
            Ok(v) => v,
            Err(err) => return Err(Error::from_reason(err.to_string())),
        };

        match label {
            Some(mut label) => {
                unsafe {
                    label.set_len(label.len() - 1);
                }

                let value = match sc.next_usize() {
                    Ok(v) => match v {
                        Some(v) => v * 1024,
                        None => return Err(Error::from_reason("unexpected EOF")),
                    },
                    Err(err) => return Err(Error::from_reason(err.to_string())),
                };

                match sc.drop_next_line() {
                    Ok(v) => {
                        if v.is_none() {
                            return Err(Error::from_reason("unexpected EOF"));
                        }
                    },
                    Err(err) => return Err(Error::from_reason(err.to_string())),
                }

                let value = env.create_double(value as f64)?;

                obj.set(unsafe { from_utf8_unchecked(&label) }, value)?;
            },
            None => break,
        }
    }

    Ok(obj)
}

/// Get data from `/proc/meminfo`. The output format is like the `free` command.
#[napi]
pub fn free() -> Result<Free> {
    const USEFUL_ITEMS: [&[u8]; 11] = [
        b"MemTotal",
        b"MemFree",
        b"MemAvailable",
        b"Buffers",
        b"Cached",
        b"SwapCached",
        b"SwapTotal",
        b"SwapFree",
        b"Shmem",
        b"Slab",
        b"SUnreclaim",
    ];

    let mut sc: ScannerAscii<_, U768> = match ScannerAscii::scan_path2("/proc/meminfo") {
        Ok(file) => file,
        Err(err) => return Err(Error::from_reason(err.to_string())),
    };

    let mut item_values = [0usize; USEFUL_ITEMS.len()];

    for (i, &item) in USEFUL_ITEMS.iter().enumerate() {
        loop {
            let label = match sc.next_raw() {
                Ok(v) => match v {
                    Some(v) => v,
                    None => return Err(Error::from_reason("unexpected EOF")),
                },
                Err(err) => return Err(Error::from_reason(err.to_string())),
            };

            if label.starts_with(item) {
                let value = match sc.next_usize() {
                    Ok(v) => match v {
                        Some(v) => v * 1024,
                        None => return Err(Error::from_reason("unexpected EOF")),
                    },
                    Err(err) => return Err(Error::from_reason(err.to_string())),
                };

                item_values[i] = value;

                match sc.drop_next() {
                    Ok(v) => {
                        if v.is_none() {
                            return Err(Error::from_reason("unexpected EOF"));
                        }
                    },
                    Err(err) => return Err(Error::from_reason(err.to_string())),
                }

                break;
            } else {
                match sc.drop_next_line() {
                    Ok(v) => {
                        if v.is_none() {
                            return Err(Error::from_reason("unexpected EOF"));
                        }
                    },
                    Err(err) => return Err(Error::from_reason(err.to_string())),
                }
            }
        }
    }

    let total = item_values[0];
    let free = item_values[1];
    let available = item_values[2];
    let buffers = item_values[3];
    let cached = item_values[4];
    let swap_cached = item_values[5];
    let swap_total = item_values[6];
    let swap_free = item_values[7];
    let shmem = item_values[8];
    let slab = item_values[9];
    let s_unreclaim = item_values[10];

    let total_cached = cached + slab - s_unreclaim;

    let mem = Mem {
        total:     total as f64,
        used:      (total - free - buffers - total_cached) as f64,
        free:      free as f64,
        shared:    shmem as f64,
        buffers:   buffers as f64,
        cache:     total_cached as f64,
        available: available as f64,
    };

    let swap = Swap {
        total: total as f64,
        used:  (swap_total - swap_free - swap_cached) as f64,
        free:  free as f64,
        cache: cached as f64,
    };

    let obj = Free {
        mem,
        swap,
    };

    Ok(obj)
}
