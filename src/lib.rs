mod utils;

use cqt::naive_cqt_octaves;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn cqt_octaves() -> Vec<u16> {
    let mut circular_buffer = [0_i16; 22050];

    // saw wave combination of all 12 tones in octave
    println!("saw wave combination: (do, mi, le) ");
    let root_freq: f64 = 110.0;
    let rate: f64 = 44100.0;
    for i in 0..22050 {
        let mut addend: f64;
        for j in 0..3 {
            let freq = root_freq * (2.0_f64).powf((j as f64) / 3.0);
            addend = (32768.0 / 3.0)
                * (2.0 * ((i as f64 * freq / rate) - (0.5 + (i as f64 * freq / rate)).floor()));
            circular_buffer[i] += addend as i16;
        }
    }

    naive_cqt_octaves(220.0, 3, rate, 12, &circular_buffer, &cqt::window::hamming)
}
