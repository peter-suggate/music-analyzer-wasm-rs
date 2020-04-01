mod utils;

use circular_queue::CircularQueue;
use cqt::naive_cqt_octaves;
use std::option::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn audio_float_to_signed16(x: &f32) -> i16 {
  std::cmp::max(std::cmp::min(32767 * *x as i16, 32767), -32767)
}

#[wasm_bindgen]
pub struct MusicAnalyzer {
  audio_samples: Vec<i16>,
}

// Once sufficient samples have been obtained, a MusicAnalyzer can be instantiated which
// provides metrics.
#[wasm_bindgen]
impl MusicAnalyzer {
  pub fn new(recent_audio_samples: Vec<i16>) -> MusicAnalyzer {
    MusicAnalyzer {
      audio_samples: recent_audio_samples,
    }
  }

  pub fn cqt_octaves(&self) -> Vec<u16> {
    let rate: f64 = 44100.0;
    naive_cqt_octaves(
      220.0,
      3,
      rate,
      12,
      &self.audio_samples,
      &cqt::window::hamming,
    )
  }

  pub fn num_samples(&self) -> usize {
    return self.audio_samples.len();
  }
}

#[wasm_bindgen]
pub struct AudioSamplesProcessor {
  recent_audio_samples: CircularQueue<Vec<i16>>,
}

#[wasm_bindgen]
impl AudioSamplesProcessor {
  pub fn new() -> AudioSamplesProcessor {
    const MAX_CHUNKS: usize = 16;

    AudioSamplesProcessor {
      recent_audio_samples: CircularQueue::with_capacity(MAX_CHUNKS),
    }
  }

  pub fn add_samples(&mut self, sample_f32s: Vec<f32>) {
    const EXPECTED_SAMPLES_PER_CHUNK: usize = 128;

    if sample_f32s.len() < EXPECTED_SAMPLES_PER_CHUNK {
      panic!(format!(
        "add_samples() requires {} samples, instead got {}",
        EXPECTED_SAMPLES_PER_CHUNK,
        sample_f32s.len()
      ));
    }

    let sample_i16s = sample_f32s.iter().map(audio_float_to_signed16).collect();

    self.recent_audio_samples.push(sample_i16s);
  }

  fn has_sufficient_samples(&self) -> bool {
    const MIN_CHUNKS_FOR_ANALYSIS: usize = 8;

    self.recent_audio_samples.len() >= MIN_CHUNKS_FOR_ANALYSIS
  }

  pub fn create_analyzer(&self) -> Option<MusicAnalyzer> {
    match self.has_sufficient_samples() {
      true => Some(MusicAnalyzer::new(
        self
          .recent_audio_samples
          .iter()
          .flat_map(|vec| vec.iter())
          .cloned()
          .collect(),
      )),
      false => None,
    }
  }
}

#[wasm_bindgen]
pub fn cqt_octaves(audio_sample_f32s: Vec<f32>) -> Vec<u16> {
  let rate: f64 = 44100.0;
  // let mut circular_buffer = [0_i16; 22050];

  // // saw wave combination of all 12 tones in octave
  // println!("saw wave combination: (do, mi, le) ");
  // let root_freq: f64 = 110.0;
  // for i in 0..22050 {
  //     let mut addend: f64;
  //     for j in 0..3 {
  //         let freq = root_freq * (2.0_f64).powf((j as f64) / 3.0);
  //         addend = (32768.0 / 3.0)
  //             * (2.0 * ((i as f64 * freq / rate) - (0.5 + (i as f64 * freq / rate)).floor()));
  //         circular_buffer[i] += addend as i16;
  //     }
  // }

  let audio_sample_i16s: Vec<_> = audio_sample_f32s
    .iter()
    .map(audio_float_to_signed16)
    .collect();

  // naive_cqt_octaves(220.0, 3, rate, 12, &circular_buffer, &cqt::window::hamming)
  naive_cqt_octaves(
    220.0,
    3,
    rate,
    12,
    &audio_sample_i16s[..],
    &cqt::window::hamming,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn converting_audio_float_to_int() {
    assert_eq!(audio_float_to_signed16(&0.0), 0);
    assert_eq!(audio_float_to_signed16(&1.0), 32767);
    assert_eq!(audio_float_to_signed16(&-1.0), -32767);
  }

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
  }

  mod music_analyzer_creation {
    use super::*;

    #[test]
    fn returns_none_if_no_samples() {
      let maybe_analyzer = AudioSamplesProcessor::new().create_analyzer();
      assert_eq!(maybe_analyzer.is_none(), true);
    }

    #[test]
    fn returns_none_if_insufficient_samples() {
      let maybe_analyzer = AudioSamplesProcessor::new().create_analyzer();
      assert_eq!(maybe_analyzer.is_none(), true);
    }

    #[test]
    fn returns_some_if_sufficient_samples() {
      let mut processor = AudioSamplesProcessor::new();

      for _ in 0..8 {
        processor.add_samples(vec![0.0; 128]);
      }

      let maybe_analyzer = processor.create_analyzer();

      assert_eq!(maybe_analyzer.is_some(), true);
    }
  }
}
