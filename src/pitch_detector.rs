use pitch_detection;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

extern crate web_sys;

const MAX_WINDOW_SIZE: usize = 8192;

fn fill_chunk(signal: &[f32], start: usize, window: usize, output: &mut [f32]) {
  let start = match signal.len() > start {
    true => start,
    false => signal.len(),
  };

  let stop = match signal.len() >= start + window {
    true => start + window,
    false => signal.len(),
  };

  for i in 0..stop - start {
    output[i] = signal[start + i];
  }

  for i in stop - start..output.len() {
    output[i] = 0.0;
  }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct Params {
  sample_rate: usize,
  window: usize,
  padding: usize,
  power_threshold: f32,
  clarity_threshold: f32,
}

pub fn make_params(window: usize) -> Params {
  Params {
    window,
    sample_rate: 44100,
    padding: window / 2,
    power_threshold: 0.25,
    clarity_threshold: 0.6,
  }
}

#[wasm_bindgen]
pub struct PitchDetector {
  params: Params,
  pub time_of_first_sample: usize,
  pub time_of_next_unprocessed_sample: usize,
  audio_samples: Vec<f32>,

  detector: Box<dyn pitch_detection::PitchDetector<f32>>,
  history: Option<pitch_detection::PitchDetectorHistory>,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Pitch {
  pub t: usize,
  pub frequency: f32,
  pub clarity: f32,
}

fn make_detector(
  detector_type: String,
  params: Params,
) -> Box<dyn pitch_detection::PitchDetector<f32>> {
  match detector_type.as_str() {
    "Autocorrelation" => Box::new(pitch_detection::AutocorrelationDetector::<f32>::new(
      params.window,
      params.padding,
    )),
    "McLeod" => Box::new(pitch_detection::McLeodDetector::<f32>::new(
      params.window,
      params.padding,
    )),
    "Smoothed McLeod" => Box::new(pitch_detection::SmoothedMcLeodDetector::<f32>::new(
      params.window,
      params.padding,
    )),
    _ => panic!(format!("unsupported detector type {}", detector_type)),
  }
}

#[wasm_bindgen]
impl PitchDetector {
  pub fn new(detector_type: String, params: Params) -> PitchDetector {
    if params.window > MAX_WINDOW_SIZE {
      panic!(format!(
        "PitchDetector::new() window size exceeded maximum window size {}",
        MAX_WINDOW_SIZE
      ))
    }

    PitchDetector {
      time_of_first_sample: 0,
      time_of_next_unprocessed_sample: 0,
      audio_samples: vec![],

      params,

      detector: make_detector(detector_type, params),
      history: None,
    }
  }

  pub fn set_audio_samples(&mut self, time_of_first_sample: usize, audio_samples: Vec<f32>) {
    if audio_samples.len() < self.params.window {
      panic!(
        "pitches() insufficient audio samples to analyze. Got {}, need: {} samples",
        audio_samples.len(),
        self.params.window
      );
    }

    self.time_of_first_sample = time_of_first_sample;

    if time_of_first_sample > self.time_of_next_unprocessed_sample {
      self.time_of_next_unprocessed_sample = time_of_first_sample;
    }

    self.audio_samples = audio_samples;
  }

  pub fn index_of_next_unprocessed_sample(&self) -> usize {
    self.time_of_next_unprocessed_sample - self.time_of_first_sample
  }

  pub fn num_audio_samples(&self) -> usize {
    self.audio_samples.len()
  }

  fn pitches_vec(&mut self) -> Vec<Pitch> {
    let mut pitches: Vec<Pitch> = Vec::<Pitch>::new();

    if self.audio_samples.len() < self.params.window {
      return pitches;
    }

    let num_unprocessed_samples =
      self.audio_samples.len() - self.index_of_next_unprocessed_sample();
    let window_samples = self.params.window;
    if num_unprocessed_samples < window_samples {
      return pitches;
    }

    let delta: usize = window_samples / 4;
    let num_windows = (num_unprocessed_samples - window_samples) / delta;

    if num_windows == 0 {
      return pitches;
    }

    // The chunk is our working memory.
    let mut chunk = vec![0.0; MAX_WINDOW_SIZE];

    let index_of_next_unprocessed_sample = self.index_of_next_unprocessed_sample();

    let detector = self.detector.as_mut();

    for i in 0..num_windows {
      let index: usize = i * delta + index_of_next_unprocessed_sample;
      fill_chunk(&self.audio_samples, index, window_samples, &mut chunk);

      let optional_pitch = detector.get_pitch(
        &chunk[0..window_samples],
        self.params.sample_rate,
        self.params.power_threshold,
        self.params.clarity_threshold,
        self.history,
      );

      // Update next unprocessed sample.
      self.time_of_next_unprocessed_sample += delta;
      // let t_end: usize = self.time_of_first_sample + index + delta;
      // if t_end > self.time_of_next_unprocessed_sample {
      //   self.time_of_next_unprocessed_sample = t_end;
      // }

      match optional_pitch {
        Some(pitch) => pitches.push(Pitch {
          clarity: pitch.clarity,
          frequency: pitch.frequency,
          t: self.time_of_next_unprocessed_sample + index,
        }),
        None => {
          println!(
            "no pitch calculated in window {}, t: {}, delta_t: {}, window: {}",
            i, index, delta, window_samples
          );
        }
      }
    }

    pitches
  }

  pub fn pitches(&mut self) -> js_sys::Array {
    self.pitches_vec().into_iter().map(JsValue::from).collect()
  }
}

#[cfg(test)]
use super::test_utils;

#[cfg(test)]
mod tests {
  use super::*;

  mod adding_samples {
    use super::*;

    #[test]
    #[should_panic(expected = "pitches() insufficient audio samples to analyze")]
    fn panics_on_insufficient_samples() {
      PitchDetector::new(String::from("McLeod"), make_params(2)).set_audio_samples(0, vec![]);
    }
  }

  mod detecting_pitches {
    use super::*;

    #[test]
    #[should_panic(
      expected = "pitches() no audio samples to analyze. Call set_audio_samples() first"
    )]
    fn panics_on_missing_samples() {
      PitchDetector::new(String::from("Autocorrelation"), make_params(4)).pitches();
    }

    #[test]
    #[should_panic(expected = "unsupported detector type Not a real pitch detector type")]
    fn panics_on_missing_detector_type() {
      PitchDetector::new(
        String::from("Not a real pitch detector type"),
        make_params(4),
      );
    }

    #[test]
    fn detects_pitch_autocorrelation() {
      const SAMPLE_RATE: usize = 44100;
      const FREQUENCY: f32 = 440.0;
      const WINDOW: usize = 1024;
      const DURATION: f32 = 0.05;
      const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;

      let signal = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);

      let mut detector = PitchDetector::new(String::from("Autocorrelation"), make_params(WINDOW));
      detector.set_audio_samples(0, signal);
      let pitches = detector.pitches_vec();

      assert_eq!(format!("{:?}", pitches), "[Pitch { t: 0, frequency: 441.0, clarity: 0.9018389 }, Pitch { t: 256, frequency: 441.0, clarity: 0.9028598 }, Pitch { t: 512, frequency: 441.0, clarity: 0.9035957 }, Pitch { t: 768, frequency: 441.0, clarity: 0.90373415 }]");
    }

    #[test]
    fn detects_pitch_mcleod() {
      const SAMPLE_RATE: usize = 44100;
      const FREQUENCY: f32 = 220.0;
      const DURATION: f32 = 0.1;
      const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;

      let signal = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);

      let mut detector = PitchDetector::new(String::from("McLeod"), make_params(2048));
      detector.set_audio_samples(0, signal);
      let pitches = detector.pitches_vec();

      assert_eq!(format!("{:?}", pitches), "[Pitch { t: 0, frequency: 220.5782, clarity: 0.9019028 }, Pitch { t: 512, frequency: 220.65475, clarity: 0.9029258 }, Pitch { t: 1024, frequency: 220.70842, clarity: 0.903646 }, Pitch { t: 1536, frequency: 220.71652, clarity: 0.90375465 }]");
    }

    #[test]
    fn returns_only_new_pitches() {
      const SAMPLE_RATE: usize = 44100;
      const FREQUENCY: f32 = 220.0;
      const DURATION: f32 = 0.1;
      const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;

      let signal = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);

      let mut detector = PitchDetector::new(String::from("McLeod"), make_params(2048));
      detector.set_audio_samples(0, signal);

      // Get the available pitches.
      detector.pitches_vec();
      println!(
        "detector.index_of_next_unprocessed_sample {}",
        detector.index_of_next_unprocessed_sample()
      );

      // Call again. There should be no more to return.
      let pitches = detector.pitches_vec();

      assert_eq!(pitches.len(), 0);

      let more_signal = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);
      detector.set_audio_samples(detector.time_of_next_unprocessed_sample, more_signal);
      let more_pitches = detector.pitches_vec();
      assert_eq!(more_pitches.len(), 4);
    }
  }
}
