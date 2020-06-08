//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

use music_analyzer_wasm_rs::*;

extern crate web_sys;
// wasm_bindgen_test_configure!(run_in_browser);

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! console_log {
  ( $( $t:tt )* ) => {
      web_sys::console::log_1(&format!( $( $t )* ).into());
  }
}

// macro_rules! console_log {
//   // Note that this is using the `log` function imported above during
//   // `bare_bones`
//   ($($t:tt)*) => (web_sys::console::log(&format_args!($($t)*).to_string()))
// }

fn print_detector_state(detector: &pitch_detector::PitchDetector) {
  console_log!(
    "time_of_first_sample {} time_of_next_unprocessed_sample {} index_of_next_unprocessed_sample {} num_audio_samples {}",
    detector.time_of_first_sample,
    detector.time_of_next_unprocessed_sample,
    detector.index_of_next_unprocessed_sample(),
    detector.num_audio_samples()
  );
  // let array = js_sys::Array::new();
  // array.push(
  //   &format!(
  //   )
  //   .into(),
  // );

  // web_sys::console::log(&array);
}

#[wasm_bindgen_test]
fn adding_data() {
  let mut processor = audio_samples_processor::AudioSamplesProcessor::new();

  const SAMPLE_RATE: usize = 44100;
  const FREQUENCY: f32 = 440.0;
  const DURATION: f32 = 0.2;
  const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;
  let sine_wave_samples = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);

  for i in 0..16 {
    processor.add_samples_chunk(
      sine_wave_samples[(i * 128)..((i + 1) * 128)]
        .iter()
        .cloned()
        .collect(),
    );
  }

  let mut detector = processor
    .create_pitch_detector(String::from("McLeod"))
    .unwrap();
  print_detector_state(&detector);

  processor.set_latest_samples_on(&mut detector);

  let mut pitches = detector.pitches();

  // Generates four pitches (one for each sliding window).
  assert_eq!(pitches.length(), 4);

  // Calling again returns nothing.
  pitches = detector.pitches();
  assert_eq!(pitches.length(), 0);
  print_detector_state(&detector);

  // Add more samples
  for i in 0..2 {
    processor.add_samples_chunk(
      sine_wave_samples[(i * 128)..((i + 1) * 128)]
        .iter()
        .cloned()
        .collect(),
    );
  }

  processor.set_latest_samples_on(&mut detector);
  print_detector_state(&detector);

  pitches = detector.pitches();
  assert_eq!(pitches.length(), 1);
  print_detector_state(&detector);

  // Try getting more pitches whenm we've exhausted the available samples.
  pitches = detector.pitches();
  assert_eq!(pitches.length(), 0);
  print_detector_state(&detector);

  // Add lots more samples (more than the internal buffer size)
  for i in 0..48 {
    processor.add_samples_chunk(
      sine_wave_samples[(i * 128)..((i + 1) * 128)]
        .iter()
        .cloned()
        .collect(),
    );
  }

  processor.set_latest_samples_on(&mut detector);
  print_detector_state(&detector);
  pitches = detector.pitches();
  assert_eq!(pitches.length(), 4);

  print_detector_state(&detector);

  // assert_eq!(
  //   pitches,
  //   pitch_detector::Pitch {
  //     t: 0,
  //     frequency: 441.14816,
  //     clarity: 0.9018697
  //   }
  // );
}
