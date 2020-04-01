//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use music_analyzer_wasm_rs;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn adding_data() {
  let mut processor = music_analyzer_wasm_rs::AudioSamplesProcessor::new();

  processor.add_samples(vec![]);
}

// #[wasm_bindgen_test]
// fn pass() {
//   let result = music_analyzer_wasm_rs::cqt_octaves();

//   assert_eq!(
//     result,
//     vec![
//       944, 279, 311, 365, 984, 249, 216, 322, 1114, 154, 209, 548, 1063, 222, 258, 363, 514, 119,
//       125, 669, 681, 287, 453, 379, 636, 209, 375, 191, 326, 173, 446, 100, 254, 103, 302, 146
//     ]
//   );
// }
