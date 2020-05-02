use pitch_detection;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
  audio_samples: Vec<f32>,

  auto_correlation: Option<Box<dyn pitch_detection::PitchDetector<f32>>>,
  mcleod: Option<Box<dyn pitch_detection::PitchDetector<f32>>>,
  smoothed_mcleod: Option<Box<dyn pitch_detection::PitchDetector<f32>>>,
  history: Option<pitch_detection::PitchDetectorHistory>,
}

pub enum DetectorType {
  Autocorrelation,
  McLeod,
  SmoothedMcLeod,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Pitch {
  pub t: usize,
  pub frequency: f32,
  pub clarity: f32,
}

#[wasm_bindgen]
impl PitchDetector {
  pub fn new(params: Params) -> PitchDetector {
    if params.window > MAX_WINDOW_SIZE {
      panic!(format!(
        "PitchDetector::new() window size exceeded maximum window size {}",
        MAX_WINDOW_SIZE
      ))
    }

    PitchDetector {
      audio_samples: vec![],

      params,

      auto_correlation: Some(Box::new(
        pitch_detection::AutocorrelationDetector::<f32>::new(params.window, params.padding),
      )),
      mcleod: Some(Box::new(pitch_detection::McLeodDetector::<f32>::new(
        params.window,
        params.padding,
      ))),
      smoothed_mcleod: Some(Box::new(
        pitch_detection::SmoothedMcLeodDetector::<f32>::new(params.window, params.padding),
      )),
      history: None,
    }
  }

  pub fn set_audio_samples(&mut self, audio_samples: Vec<f32>) {
    if audio_samples.len() < self.params.window {
      panic!("pitches() insufficient audio samples to analyze.");
    }

    self.audio_samples = audio_samples;
  }

  fn pitches_vec(&mut self, detector_type_str: String) -> Vec<Pitch> {
    if self.audio_samples.len() < self.params.window {
      panic!("pitches() no audio samples to analyze. Call set_audio_samples() first.");
    }

    let detector_type = match detector_type_str.as_str() {
      "Autocorrelation" => DetectorType::Autocorrelation,
      "McLeod" => DetectorType::McLeod,
      "Smoothed McLeod" => DetectorType::SmoothedMcLeod,
      _ => panic!(format!(
        "piches() unsupported detector type {}",
        detector_type_str
      )),
    };

    let window = self.params.window;
    let delta_t: usize = window / 4;
    let num_windows = (&self.audio_samples.len() - window) / delta_t;
    assert!(
      num_windows > 0,
      "Insufficient data samples to perform pitch analysis"
    );

    let mut pitches: Vec<Pitch> = Vec::<Pitch>::new();

    // The chunk is our working memory.
    let mut chunk = vec![0.0; MAX_WINDOW_SIZE];

    let detector = match detector_type {
      DetectorType::Autocorrelation => self.auto_correlation.as_mut().unwrap(),
      DetectorType::McLeod => self.mcleod.as_mut().unwrap(),
      DetectorType::SmoothedMcLeod => self.smoothed_mcleod.as_mut().unwrap(),
    };

    for i in 0..num_windows {
      let t: usize = i * delta_t;
      fill_chunk(&self.audio_samples, t, window, &mut chunk);

      let optional_pitch = detector.get_pitch(
        &chunk[0..window],
        self.params.sample_rate,
        self.params.power_threshold,
        self.params.clarity_threshold,
        self.history,
      );

      match optional_pitch {
        Some(pitch) => pitches.push(Pitch {
          clarity: pitch.clarity,
          frequency: pitch.frequency,
          t,
        }),
        None => {
          println!(
            "no pitch calculated in window {}, t: {}, delta_t: {}, window: {}",
            i, t, delta_t, window
          );
        }
      }
    }

    pitches
  }

  pub fn next_pitch(&mut self, detector_type_str: String) -> Option<Pitch> {
    let pitches = self.pitches_vec(detector_type_str);

    if pitches.len() > 0 {
      return Some(pitches[0]);
    }

    None
  }

  pub fn pitches(&mut self, detector_type_str: String) -> js_sys::Array {
    self
      .pitches_vec(detector_type_str)
      .into_iter()
      .map(JsValue::from)
      .collect()
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
      PitchDetector::new(make_params(2)).set_audio_samples(vec![]);
    }
  }

  mod detecting_pitches {
    use super::*;

    #[test]
    #[should_panic(
      expected = "pitches() no audio samples to analyze. Call set_audio_samples() first"
    )]
    fn panics_on_missing_samples() {
      PitchDetector::new(make_params(4)).pitches(String::from("Autocorrelation"));
    }

    #[test]
    #[should_panic(expected = "piches() unsupported detector type Not a real pitch detector type")]
    fn panics_on_missing_detector_type() {
      let mut detector = PitchDetector::new(make_params(4));
      detector.set_audio_samples(vec![0.2, 0.4, 1.5, 0.2]);
      detector.pitches_vec(String::from("Not a real pitch detector type"));
    }

    #[test]
    fn detects_pitch_autocorrelation() {
      const SAMPLE_RATE: usize = 44100;
      const FREQUENCY: f32 = 440.0;
      const WINDOW: usize = 1024;
      const DURATION: f32 = 0.05;
      const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;

      let signal = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);

      let mut detector = PitchDetector::new(make_params(WINDOW));
      detector.set_audio_samples(signal);
      let pitches = detector.pitches_vec(String::from("Autocorrelation"));

      assert_eq!(format!("{:?}", pitches), "[Pitch { t: 0, frequency: 441.0, clarity: 0.9018389 }, Pitch { t: 256, frequency: 441.0, clarity: 0.9028598 }, Pitch { t: 512, frequency: 441.0, clarity: 0.9035957 }, Pitch { t: 768, frequency: 441.0, clarity: 0.90373415 }]");
    }

    #[test]
    fn detects_pitch_mcleod() {
      const SAMPLE_RATE: usize = 44100;
      const FREQUENCY: f32 = 220.0;
      const DURATION: f32 = 0.1;
      const SAMPLE_SIZE: usize = (SAMPLE_RATE as f32 * DURATION) as usize;

      let signal = test_utils::sin_signal(FREQUENCY, SAMPLE_SIZE, SAMPLE_RATE);

      let mut detector = PitchDetector::new(make_params(2048));
      detector.set_audio_samples(signal);
      let pitches = detector.pitches_vec(String::from("McLeod"));

      assert_eq!(format!("{:?}", pitches), "[Pitch { t: 0, frequency: 220.5782, clarity: 0.9019028 }, Pitch { t: 512, frequency: 220.65475, clarity: 0.9029258 }, Pitch { t: 1024, frequency: 220.70842, clarity: 0.903646 }, Pitch { t: 1536, frequency: 220.71652, clarity: 0.90375465 }]");
    }
  }
}
