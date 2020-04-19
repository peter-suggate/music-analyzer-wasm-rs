pub fn new_real_buffer(size: usize) -> Vec<f32> {
  vec![0.0; size]
}

pub fn sin_signal(freq: f32, size: usize, sample_rate: usize) -> Vec<f32> {
  let mut signal = new_real_buffer(size);
  let two_pi = 2.0 * std::f32::consts::PI;
  let dx = two_pi * freq / sample_rate as f32;
  for i in 0..size {
    let x = i as f32 * dx;
    let y = x.sin();
    signal[i] = y;
  }
  signal
}
