use fixed::types::I1F31;

pub struct PeakDetector {
    value: I1F31,
}

impl PeakDetector {
    pub fn new() -> Self {
        Self { value: I1F31::ZERO }
    }

    pub fn step(&mut self, x: I1F31) -> I1F31 {
        const ALPHA: I1F31 = I1F31::from_bits(2146624653);
        let p = x * x;
        self.value = ALPHA.lerp(p, self.value);
        self.value //.sqrt()

        // self.value = I1F31::max(
        //     x * x,
        //     I1F31::max(I1F31::ZERO, self.value - I1F31::from_bits(429496)),
        // );
        // self.value
    }
}
