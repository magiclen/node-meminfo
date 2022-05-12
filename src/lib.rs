use std::str::from_utf8_unchecked;

use neon::prelude::*;

use scanner_rust::generic_array::typenum::U768;
use scanner_rust::ScannerAscii;

fn get(mut cx: FunctionContext) -> JsResult<JsObject> {
    let mut sc: ScannerAscii<_, U768> = match ScannerAscii::scan_path2("/proc/meminfo") {
        Ok(file) => file,
        Err(err) => return cx.throw_error(err.to_string()),
    };

    let obj = JsObject::new(&mut cx);

    loop {
        let label = match sc.next_raw() {
            Ok(v) => v,
            Err(err) => return cx.throw_error(err.to_string()),
        };

        match label {
            Some(mut label) => {
                unsafe {
                    label.set_len(label.len() - 1);
                }

                let value = match sc.next_usize() {
                    Ok(v) => {
                        match v {
                            Some(v) => v * 1024,
                            None => return cx.throw_error("unexpected EOF"),
                        }
                    }
                    Err(err) => return cx.throw_error(err.to_string()),
                };

                match sc.drop_next_line() {
                    Ok(v) => {
                        if v.is_none() {
                            return cx.throw_error("unexpected EOF");
                        }
                    }
                    Err(err) => return cx.throw_error(err.to_string()),
                }

                let value = JsNumber::new(&mut cx, value as f64);

                obj.set(&mut cx, unsafe { from_utf8_unchecked(&label) }, value)?;
            }
            None => break,
        }
    }

    Ok(obj)
}

fn free(mut cx: FunctionContext) -> JsResult<JsObject> {
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
        Err(err) => return cx.throw_error(err.to_string()),
    };

    let mut item_values = [0usize; USEFUL_ITEMS.len()];

    for (i, &item) in USEFUL_ITEMS.iter().enumerate() {
        loop {
            let label = match sc.next_raw() {
                Ok(v) => {
                    match v {
                        Some(v) => v,
                        None => return cx.throw_error("unexpected EOF"),
                    }
                }
                Err(err) => return cx.throw_error(err.to_string()),
            };

            if label.starts_with(item) {
                let value = match sc.next_usize() {
                    Ok(v) => {
                        match v {
                            Some(v) => v * 1024,
                            None => return cx.throw_error("unexpected EOF"),
                        }
                    }
                    Err(err) => return cx.throw_error(err.to_string()),
                };

                item_values[i] = value;

                match sc.drop_next() {
                    Ok(v) => {
                        if v.is_none() {
                            return cx.throw_error("unexpected EOF");
                        }
                    }
                    Err(err) => return cx.throw_error(err.to_string()),
                }

                break;
            } else {
                match sc.drop_next_line() {
                    Ok(v) => {
                        if v.is_none() {
                            return cx.throw_error("unexpected EOF");
                        }
                    }
                    Err(err) => return cx.throw_error(err.to_string()),
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

    let mem = JsObject::new(&mut cx);

    let v = JsNumber::new(&mut cx, total as f64);
    mem.set(&mut cx, "total", v)?;

    let v = JsNumber::new(&mut cx, (total - free - buffers - total_cached) as f64);
    mem.set(&mut cx, "used", v)?;

    let v = JsNumber::new(&mut cx, free as f64);
    mem.set(&mut cx, "free", v)?;

    let v = JsNumber::new(&mut cx, shmem as f64);
    mem.set(&mut cx, "shared", v)?;

    let v = JsNumber::new(&mut cx, buffers as f64);
    mem.set(&mut cx, "buffers", v)?;

    let v = JsNumber::new(&mut cx, total_cached as f64);
    mem.set(&mut cx, "cache", v)?;

    let v = JsNumber::new(&mut cx, available as f64);
    mem.set(&mut cx, "available", v)?;

    let swap = JsObject::new(&mut cx);

    let v = JsNumber::new(&mut cx, total_cached as f64);
    swap.set(&mut cx, "total", v)?;

    let v = JsNumber::new(&mut cx, (swap_total - swap_free - swap_cached) as f64);
    swap.set(&mut cx, "used", v)?;

    let v = JsNumber::new(&mut cx, swap_free as f64);
    swap.set(&mut cx, "free", v)?;

    let v = JsNumber::new(&mut cx, swap_cached as f64);
    swap.set(&mut cx, "cache", v)?;

    let obj = JsObject::new(&mut cx);

    obj.set(&mut cx, "mem", mem)?;
    obj.set(&mut cx, "swap", swap)?;

    Ok(obj)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("get", get)?;
    cx.export_function("free", free)?;

    Ok(())
}
