#![no_main]
use bencode::{from_bytes, Value};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = from_bytes::<Value>(data);
});
