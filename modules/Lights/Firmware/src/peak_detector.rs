use fixed::types::I1F31;

pub struct PeakDetector {
    value: I1F31,
}

impl PeakDetector {
    pub fn new() -> Self {
        Self { value: I1F31::ZERO }
    }

    pub fn step(&mut self, x: I1F31) -> I1F31 {
        I1F31::max(
            x.abs(),
            I1F31::max(I1F31::ZERO, self.value - I1F31::from_bits(0x2222222)),
        )

        // if x >= self.value {
        //     self.value = x;
        //     x
        // }
    }
}
