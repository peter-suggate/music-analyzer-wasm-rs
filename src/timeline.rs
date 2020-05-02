use circular_queue::CircularQueue;

#[derive(Copy, Clone, Debug)]
pub struct EventTime {
  pub ms: f32,
}

impl EventTime {
  pub fn new(ms: f32) -> Option<EventTime> {
    match ms >= 0.0 {
      true => Some(EventTime { ms }),
      false => None,
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct SeriesEvent {
  pub time_from_start_ms: EventTime,
  pub pitch_hz: f32,
}

pub struct Series {
  pub name: String,
  events: CircularQueue<SeriesEvent>,
}

impl Series {
  pub fn new(name: String) -> Series {
    const DEFAULT_CAPACITY: usize = 100;

    Series {
      name,
      events: CircularQueue::with_capacity(DEFAULT_CAPACITY),
    }
  }

  pub fn add_pitch_event(&mut self, time_from_start_ms: f32, pitch_hz: f32) {
    assert!(
      self.events.len() == 0
        || self.time_of_most_recent_event().unwrap().ms < time_from_start_ms,
      format!("Events must be added in chronological order. Got event t = {} when most recent event t = {}", time_from_start_ms, self.time_of_most_recent_event().map(|t| { t.ms }).unwrap_or(-1.0))
    );

    let event_time = EventTime::new(time_from_start_ms).expect(
      format!(
        "Events must have a time >= 0 but got {}",
        time_from_start_ms
      )
      .as_str(),
    );

    self.events.push(SeriesEvent {
      time_from_start_ms: event_time,
      pitch_hz,
    })
  }

  fn time_of_most_recent_event(&self) -> Option<EventTime> {
    self.events.iter().map(|e| e.time_from_start_ms).next()
  }

  pub fn events_after(&self, after_ms: f32) -> Vec<SeriesEvent> {
    return self
      .events
      .iter()
      .filter(|e| e.time_from_start_ms.ms >= after_ms)
      .rev()
      .cloned()
      .collect();
  }
}

pub struct Timeline {
  series: Vec<Series>,
}

impl Timeline {
  pub fn new() -> Timeline {
    Timeline { series: Vec::new() }
  }

  pub fn add_series(&mut self, series: Series) {
    self.series.push(series);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod series {
    use super::*;

    #[test]
    fn starts_empty() {
      let series = Series::new(String::from("Series"));

      assert_eq!(series.events.len(), 0);
    }

    #[test]
    fn adding_first_event() {
      let mut series = Series::new(String::from("Series"));

      series.add_pitch_event(0.0, 440.0);

      assert_eq!(series.events.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Events must have a time >= 0 but got -1")]
    fn adding_event_with_neg_time() {
      let mut series = Series::new(String::from("Series"));

      series.add_pitch_event(-1.0, 440.0);
    }

    #[test]
    #[should_panic(
      expected = "Events must be added in chronological order. Got event t = 1 when most recent event t = 2"
    )]
    fn adding_events_out_of_chrono_order() {
      let mut series = Series::new(String::from("Series"));

      series.add_pitch_event(0.0, 220.0);

      series.add_pitch_event(2.0, 440.0);

      // Whoops, earlier than the most recent event. Panics.
      series.add_pitch_event(1.0, 880.0);
    }

    #[test]
    fn adding_first_event_beyond_capacity() {
      let mut series = Series::new(String::from("Series"));

      for i in 0..101 {
        series.add_pitch_event(i as f32, 440.0);
      }

      assert_eq!(series.events.len(), 100);
    }

    #[test]
    fn retrieve_all_events_by_time() {
      let mut series = Series::new(String::from("Series"));

      series.add_pitch_event(0.0, 220.0);
      series.add_pitch_event(2.0, 440.0);
      series.add_pitch_event(3.0, 220.0);
      series.add_pitch_event(4.0, 880.0);

      let times: Vec<f32> = series
        .events_after(0.0)
        .iter()
        .map(|e| e.time_from_start_ms.ms)
        .collect();
      assert_eq!(times, [0.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn retrieve_events_beyond_specific_time() {
      let mut series = Series::new(String::from("Series"));

      series.add_pitch_event(0.0, 220.0);
      series.add_pitch_event(2.0, 440.0);
      series.add_pitch_event(3.0, 220.0);
      series.add_pitch_event(4.0, 880.0);

      let times: Vec<f32> = series
        .events_after(2.5)
        .iter()
        .map(|e| e.time_from_start_ms.ms)
        .collect();
      assert_eq!(times, [3.0, 4.0]);
    }
  }

  mod timeline {
    use super::*;

    #[test]
    fn timeline_starts_empty() {
      let timeline = Timeline::new();

      assert_eq!(timeline.series.len(), 0);
    }

    #[test]
    fn adding_new_series() {
      let mut timeline = Timeline::new();

      timeline.add_series(Series::new(String::from("Series A")));
      timeline.add_series(Series::new(String::from("Series B")));

      assert_eq!(timeline.series.len(), 2);
    }
  }
}
