/// Computes a moving average over a ring buffer of numbers.
#[derive(Debug)]
pub struct MovingAverage {
    /// Current history of numbers.
    hist: Vec<f64>,

    /// Size of the history, as T.
    size: f64,

    /// Index in the history vector to replace next.
    pos: usize,
}

impl MovingAverage {
    /// Create a new `MovingAverage` that averages over the given amount of numbers.
    pub fn new(size: usize) -> Self {
        MovingAverage {
            hist: vec![0.0; size],
            size: f64::from(size as u32),
            pos: 0,
        }
    }

    /// Add the given number to the history, overwriting the oldest number, and return the
    /// resulting moving average.
    pub fn feed(&mut self, num: f64) -> f64 {
        self.hist[self.pos] = num;

        self.pos += 1;
        self.pos %= self.hist.len();

        self.avg()
    }

    /// Calculate moving average based on the current history.
    fn avg(&self) -> f64 {
        self.hist.iter().fold(0.0, |s, &x| s + x) / self.size
    }
}
