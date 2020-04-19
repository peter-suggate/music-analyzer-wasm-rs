//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use music_analyzer_wasm_rs::*;

// wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn adding_data() {
  let mut processor = AudioSamplesProcessor::new();

  const SAMPLE_RATE: usize = 44100;
  const FREQUENCY: f32 = 440.0;
  const DURATION: f32 = 0.2;
  const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;
  let sine_wave_samples = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);

  for i in 0..16 {
    processor.add_samples(
      sine_wave_samples[(i * 128)..((i + 1) * 128)]
        .iter()
        .cloned()
        .collect(),
    );
  }

  let mut detector = processor.create_pitch_detector().unwrap();
  processor.set_latest_samples_on(&mut detector);

  let pitch = detector.next_pitch(String::from("McLeod")).unwrap();

  assert_eq!(
    pitch,
    pitch_detector::Pitch {
      t: 0,
      frequency: 441.14816,
      clarity: 0.9018697
    }
  );

  let pitches = detector.pitches(String::from("McLeod"));

  // Generates four pitches (one for each sliding window).
  assert_eq!(pitches.length(), 4);
}
