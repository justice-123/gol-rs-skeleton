const BUF_SIZE: usize = 3;

pub struct AvgTurns {
    count: usize,
    last_completed_turns: u32,
    last_called: std::time::Instant,
    buf_turns: [u32; BUF_SIZE],
    buf_durations: [tokio::time::Duration; BUF_SIZE],
}

impl AvgTurns {
    pub fn new() -> Self {
        AvgTurns {
            count: 0,
            last_completed_turns: 0,
            last_called: std::time::Instant::now(),
            buf_turns: [Default::default(); BUF_SIZE],
            buf_durations: [Default::default(); BUF_SIZE],
        }
    }

    pub fn get(&mut self, completed_turns: u32) -> u32 {
        self.buf_turns[self.count % BUF_SIZE] = completed_turns - self.last_completed_turns;
        self.buf_durations[self.count % BUF_SIZE] = self.last_called.elapsed();
        self.last_called = std::time::Instant::now();
        self.last_completed_turns = completed_turns;
        self.count += 1;
        return self.buf_turns.iter().sum::<u32>() / self.buf_durations.iter()
            .map(|t| t.as_secs_f32().round() as u32).sum::<u32>().clamp(1, u32::MAX);
    }
}
