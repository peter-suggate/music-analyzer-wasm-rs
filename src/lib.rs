pub mod test_utils;
mod utils;

use circular_queue::CircularQueue;
use std::option::*;
use wasm_bindgen::prelude::*;

pub mod pitch_detector;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const AUDIO_SAMPLES_PER_CHUNK: usize = 128;
const MIN_CHUNKS_FOR_ANALYSIS: usize = 16;

#[wasm_bindgen]
pub struct AudioSamplesProcessor {
  chunk_size: usize,
  max_stored_chunks: usize,

  recent_audio_sample_f32s: CircularQueue<Vec<f32>>,
}

#[wasm_bindgen]
impl AudioSamplesProcessor {
  pub fn new() -> AudioSamplesProcessor {
    AudioSamplesProcessor {
      // Matching the web audio worklet chunk size
      chunk_size: AUDIO_SAMPLES_PER_CHUNK,

      max_stored_chunks: MIN_CHUNKS_FOR_ANALYSIS,

      recent_audio_sample_f32s: CircularQueue::with_capacity(MIN_CHUNKS_FOR_ANALYSIS),
    }
  }

  pub fn add_samples(&mut self, sample_f32s: Vec<f32>) {
    if sample_f32s.len() != self.chunk_size {
      panic!(format!(
        "add_samples() requires {} samples, instead got {}",
        self.chunk_size,
        sample_f32s.len()
      ));
    }

    self.recent_audio_sample_f32s.push(sample_f32s);
  }

  pub fn has_sufficient_samples(&self) -> bool {
    self.recent_audio_sample_f32s.len() >= self.max_stored_chunks
  }

  pub fn create_pitch_detector(&self) -> Option<pitch_detector::PitchDetector> {
    const WINDOW: usize = 1024;
    Some(pitch_detector::PitchDetector::new(
      pitch_detector::make_params(WINDOW),
    ))
  }

  pub fn set_latest_samples_on(&self, detector: &mut pitch_detector::PitchDetector) {
    detector.set_audio_samples(self.get_latest_samples())
  }

  pub fn get_latest_samples(&self) -> Vec<f32> {
    self
      .recent_audio_sample_f32s
      .iter()
      .rev()
      .flat_map(|vec| vec.iter())
      .cloned()
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod adding_samples {
    use super::*;

    #[test]
    #[should_panic(expected = "add_samples() requires 128 samples, instead got 0")]
    fn panics_on_empty_samples_vec() {
      AudioSamplesProcessor::new().add_samples(vec![]);
    }

    #[test]
    fn adds_samples_if_of_correct_count() {
      AudioSamplesProcessor::new().add_samples(vec![0.0; 128]);
    }

    #[test]
    fn returns_added_chunks_in_correct_order() {
      let mut processor = AudioSamplesProcessor::new();

      let mut samples = vec![0.0; 256];
      for i in 0..2 {
        let chunk = vec![i as f32; 128];
        processor.add_samples(chunk);

        samples[(i * 128)..((i + 1) * 128)]
          .iter_mut()
          .map(|v| *v = i as f32)
          .count();
      }

      assert_eq!(processor.get_latest_samples(), samples);
    }

    #[test]
    fn wraps_around_when_max_stored_samples_exceeded() {
      let mut processor = AudioSamplesProcessor::new();

      // Generate monotonically increasing sample values totalling two times the number
      // of capacity.
      const CHUNKS: usize = MIN_CHUNKS_FOR_ANALYSIS * 2;
      let mut samples = vec![0.0; MIN_CHUNKS_FOR_ANALYSIS * AUDIO_SAMPLES_PER_CHUNK];

      for i in 0..CHUNKS {
        let chunk = vec![i as f32; AUDIO_SAMPLES_PER_CHUNK];
        processor.add_samples(chunk);

        if i >= MIN_CHUNKS_FOR_ANALYSIS {
          samples[((i - processor.max_stored_chunks) * processor.chunk_size)
            ..((i - processor.max_stored_chunks + 1) * processor.chunk_size)]
            .iter_mut()
            .map(|v| *v = i as f32)
            .count();
        }
      }

      assert_eq!(processor.get_latest_samples(), samples);
    }
  }

  mod pitch_detector_tests {
    use super::*;

    #[test]
    fn returns_one_if_no_samples() {
      let maybe_analyzer = AudioSamplesProcessor::new().create_pitch_detector();
      assert_eq!(maybe_analyzer.is_some(), true);
    }

    #[test]
    fn copies_samples_into_detector() {
      let mut processor = AudioSamplesProcessor::new();

      const WINDOW: usize = 1024;
      let sine_wave_samples = test_utils::sin_signal(440.0, WINDOW * 2, 44100);

      for i in 0..16 {
        processor.add_samples(sine_wave_samples[(i * 128)..((i + 1) * 128)].to_vec());
      }

      let mut detector = processor.create_pitch_detector().unwrap();

      processor.set_latest_samples_on(&mut detector);

      let next_pitch = detector.next_pitch(String::from("McLeod")).unwrap();

      assert_eq!(
        next_pitch,
        pitch_detector::Pitch {
          t: 0,
          frequency: 441.14816,
          clarity: 0.9018697
        }
      )
    }
  }
}
