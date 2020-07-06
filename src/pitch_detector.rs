use pitch_detection;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

extern crate web_sys;

pub const MAX_WINDOW_SIZE: usize = 8192;

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
    sample_rate: 48000,
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

  // Last returned pitch or None. Used for onset detection and potentially to help
  // produce stable pitches whenever there's ambiguity (between octaves for example).
  current_pitch: Option<f32>,

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
  pub onset: bool,
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
pub struct PitchesResult {
  _code: String,
  _message: String,
  _pitches: Vec<Pitch>,
}

#[wasm_bindgen]
impl PitchesResult {
  pub fn from_error(code: String, message: String) -> PitchesResult {
    PitchesResult {
      _code: code,
      _message: message,
      _pitches: Vec::new(),
    }
  }

  fn from_vec(pitches: Vec<Pitch>) -> PitchesResult {
    PitchesResult {
      _code: String::from("success"),
      _message: String::from(""),
      _pitches: pitches,
    }
  }

  #[wasm_bindgen(getter)]
  pub fn code(&self) -> String {
    self._code.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn message(&self) -> String {
    self._message.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn pitches(&self) -> js_sys::Array {
    self
      ._pitches
      .clone()
      .into_iter()
      .map(JsValue::from)
      .collect()
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
      current_pitch: None,
      audio_samples: vec![],

      params,

      detector: make_detector(detector_type, params),
      history: None,
    }
  }

  pub fn set_audio_samples(&mut self, time_of_first_sample: usize, audio_samples: Vec<f32>) {
    println!("audio_samples.len() {}", audio_samples.len());
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

      match optional_pitch {
        Some(pitch) => {
          // We detected a pitch.
          let onset = match self.current_pitch {
            Some(_current_pitch) => false,
            None => true,
          };

          self.current_pitch = Some(pitch.frequency);

          pitches.push(Pitch {
            clarity: pitch.clarity,
            frequency: pitch.frequency,
            t: self.time_of_next_unprocessed_sample + index,
            onset: onset,
          })
        }
        None => {
          // A break in the sound or sound quality has occurred. Next resumption will be onset
          // of a new note.
          self.current_pitch = None;

          println!(
            "no pitch calculated in window {}, t: {}, delta_t: {}, window: {}",
            i,
            self.time_of_next_unprocessed_sample + index,
            delta,
            window_samples
          );
        }
      }
    }

    pitches
  }

  pub fn pitches(&mut self) -> PitchesResult {
    if self.audio_samples.len() < self.params.window {
      return PitchesResult::from_error(String::from("not_enough_samples"),
        String::from(format!("pitches() requires at least {} samples and there are currently {}. Ensure set_audio_samples() has been called once enough samples are available.", self.params.window, self.audio_samples.len()))
    );
    }

    PitchesResult::from_vec(self.pitches_vec())
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

    const WINDOW: usize = 2048;

    fn sin_signal_samples(freq_hz: f32, duration_secs: f32) -> Vec<f32> {
      const SAMPLE_RATE: usize = 48000;
      let samples: usize = (SAMPLE_RATE as f32 * duration_secs) as usize;

      test_utils::sin_signal(freq_hz, samples, SAMPLE_RATE)
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
      let mut detector = PitchDetector::new(String::from("Autocorrelation"), make_params(WINDOW));

      detector.set_audio_samples(0, sin_signal_samples(440.0, 0.1));
      let pitches = detector.pitches_vec();

      assert_eq!(format!("{:?}", pitches), "[Pitch { t: 512, frequency: 440.36697, clarity: 0.94680345, onset: true }, Pitch { t: 1536, frequency: 440.36697, clarity: 0.94702, onset: false }, Pitch { t: 2560, frequency: 440.36697, clarity: 0.9463327, onset: false }, Pitch { t: 3584, frequency: 440.36697, clarity: 0.9471525, onset: false }, Pitch { t: 4608, frequency: 440.36697, clarity: 0.9465997, onset: false }]");
    }

    #[test]
    fn detects_pitch_mcleod() {
      let mut detector = PitchDetector::new(String::from("McLeod"), make_params(WINDOW));

      detector.set_audio_samples(0, sin_signal_samples(220.0, 0.1));
      let pitches = detector.pitches_vec();

      assert_eq!(format!("{:?}", pitches), "[Pitch { t: 512, frequency: 220.29074, clarity: 0.894376, onset: true }, Pitch { t: 1536, frequency: 221.12888, clarity: 0.89288074, onset: false }, Pitch { t: 2560, frequency: 220.72627, clarity: 0.89353347, onset: false }, Pitch { t: 3584, frequency: 220.17342, clarity: 0.8946273, onset: false }, Pitch { t: 4608, frequency: 220.95581, clarity: 0.89314663, onset: false }]");
    }

    #[test]
    fn returns_only_new_pitches() {
      let mut detector = PitchDetector::new(String::from("McLeod"), make_params(2048));

      detector.set_audio_samples(0, sin_signal_samples(220.0, 0.1));

      // Get the available pitches.
      detector.pitches_vec();
      println!(
        "detector.index_of_next_unprocessed_sample {}",
        detector.index_of_next_unprocessed_sample()
      );

      // Call again. There should be no more to return.
      let pitches = detector.pitches_vec();

      assert_eq!(pitches.len(), 0);

      detector.set_audio_samples(
        detector.time_of_next_unprocessed_sample,
        sin_signal_samples(220.0, 0.1),
      );
      let more_pitches = detector.pitches_vec();
      assert_eq!(more_pitches.len(), 5);
    }

    #[test]
    fn first_pitch_is_an_onset() {
      let mut detector = PitchDetector::new(String::from("McLeod"), make_params(WINDOW));

      detector.set_audio_samples(0, sin_signal_samples(220.0, 0.1));
      let pitches = detector.pitches_vec();

      assert_eq!(pitches[0].onset, true);
    }

    #[test]
    fn second_pitch_is_not_an_onset() {
      let mut detector = PitchDetector::new(String::from("McLeod"), make_params(WINDOW));

      detector.set_audio_samples(0, sin_signal_samples(220.0, 0.1));
      let pitches = detector.pitches_vec();

      assert_eq!(pitches[1].onset, false);
    }

    #[test]
    fn first_pitch_after_silence_is_an_onset() {
      let mut detector = PitchDetector::new(String::from("McLeod"), make_params(WINDOW));

      // Get first round of pitches.
      detector.set_audio_samples(0, sin_signal_samples(220.0, 0.1));
      detector.pitches_vec();

      // Add a some flat signal / noise where no pitches are generated.
      detector.set_audio_samples(
        detector.time_of_next_unprocessed_sample,
        sin_signal_samples(0.0, 0.1),
      );
      detector.pitches_vec();

      // Resumption of a signal that produces pitches.
      detector.set_audio_samples(
        detector.time_of_next_unprocessed_sample,
        sin_signal_samples(440.0, 0.1),
      );
      let pitches = detector.pitches_vec();

      assert_eq!(pitches[0].onset, true);
    }
  }
}
