#[macro_use]
pub mod macros;
pub mod audio_samples_processor;
pub mod pitch_detector;
pub mod test_utils;
pub mod timeline;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Integration tests

#[cfg(test)]
mod tests {
  // use super::*;

  // #[test]
  // fn populating_timeline_with_events() {
  //   let mut processor = audio_samples_processor::AudioSamplesProcessor::new();

  //   // Play C Major scale, crotchet = 60 BPM.
  //   const SAMPLE_RATE: usize = 48000;

  //   let c_major_scale_pitches: Vec<f32> = vec![
  //     261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25,
  //   ];
  //   const DURATION: f32 = 1.0;
  //   const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;

  //   let mut total_samples = vec![];
  //   for hz in c_major_scale_pitches {
  //     total_samples.extend(test_utils::sin_signal(hz, SAMPLE_SIZE, SAMPLE_RATE));
  //   }

  //   let mut detector_mcleod = processor
  //     .create_pitch_detector(String::from("McLeod"), 2048)
  //     .unwrap();
  //   // let mut detector_autocorrelation = processor
  //   //   .create_pitch_detector(String::from("Autocorrelation"))
  //   //   .unwrap();

  //   let total_chunks =
  //     (total_samples.len() as f32 / (processor.chunk_size as f32)).floor() as usize;

  //   // Assume the audio samples arrive in chunks of 128 samples (as they do from a web browser's)
  //   // AudioWorklet.
  //   for i in 0..total_chunks {
  //     processor.add_samples_chunk(
  //       total_samples[(i * 128)..((i + 1) * 128)]
  //         .iter()
  //         .cloned()
  //         .collect(),
  //     );

  //     // Extract any new detected events to the timeline.
  //     // detector_autocorrelation.
  //   }

  //   // Pass the latest samples to the detectors.
  //   // processor.set_latest_samples_on(&mut detector_autocorrelation);
  //   processor.set_latest_samples_on(&mut detector_mcleod);

  //   let pitches = detector_mcleod.pitches();
  //   // Generates four pitches (one for each sliding window).
  //   assert_eq!(pitches.length(), 4);
  // }
}
