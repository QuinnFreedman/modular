use core::marker::ConstParamTy;

/**
An implementation of the LFSR113 pseudo-random number generator algorithm (taken from
https://web.mst.edu/vojtat/class_5403/lfsr113/). This isn't used right now, since I
don't need 32-bit randomness for anything, but I'm leaving it here for now.
*/
pub struct LFSR113 {
    z1: u32,
    z2: u32,
    z3: u32,
    z4: u32,
}

impl LFSR113 {
    pub fn new(seed: i32) -> Self {
        const IA: i32 = 16807;
        const IM: i32 = 2147483647;
        const IQ: i32 = 127773;
        const IR: i32 = 2836;

        let mut idum = seed;
        if idum <= 0 {
            idum = 1;
        }

        let mut z = [0u32; 4];
        for i in 0..4 {
            let k: i32 = (idum) / IQ;
            idum = IA * (idum - k * IQ) - IR * k;
            if idum < 0 {
                idum += IM;
            }
            let pow = [2, 8, 16, 128][i];
            z[i] = (if idum < pow { idum + pow } else { idum }).unsigned_abs();
        }

        let mut result = Self {
            z1: z[0],
            z2: z[1],
            z3: z[2],
            z4: z[3],
        };
        let _ = result.getu32();
        result
    }

    pub fn getu32(&mut self) -> u32 {
        let mut b: u32 = ((self.z1 << 6) ^ self.z1) >> 13;
        self.z1 = ((self.z1 & 4294967294) << 18) ^ b;
        b = ((self.z2 << 2) ^ self.z2) >> 27;
        self.z2 = ((self.z2 & 4294967288) << 2) ^ b;
        b = ((self.z3 << 13) ^ self.z3) >> 21;
        self.z3 = ((self.z3 & 4294967280) << 7) ^ b;
        b = ((self.z4 << 3) ^ self.z4) >> 12;
        self.z4 = ((self.z4 & 4294967168) << 13) ^ b;
        return (self.z1 ^ self.z2 ^ self.z3 ^ self.z4) >> 1;
    }
}

#[derive(PartialEq, Eq, ConstParamTy)]
pub struct LfsrConfig([u8; 4]);

pub const LFSR_CONFIG_1: LfsrConfig = LfsrConfig([16, 14, 13, 11]);
pub const LFSR_CONFIG_2: LfsrConfig = LfsrConfig([16, 15, 13, 4]);
pub const LFSR_CONFIG_3: LfsrConfig = LfsrConfig([16, 15, 14, 11]);

pub struct LFSR<const TAPS: LfsrConfig> {
    state: u16,
}

impl<const TAPS: LfsrConfig> LFSR<TAPS> {
    pub fn new(seed: u16) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u16 {
        let mut bit = 0;
        for tap in TAPS.0 {
            bit ^= self.state >> (u16::BITS - tap as u32);
        }
        self.state = (self.state >> 1) | (bit << 15);
        self.state
    }
}

pub struct ParallelLfsr {
    lfsr1: LFSR<LFSR_CONFIG_1>,
    lfsr2: LFSR<LFSR_CONFIG_2>,
}

impl ParallelLfsr {
    pub fn new(seed: u16) -> Self {
        // Generate second seed from first; I don't know if this method is ideal
        // but this just a way to try to get low correlation between seeds
        let seed2 = !(((seed >> 8) & 0xff) | (seed << 8));
        Self {
            lfsr1: LFSR::new(seed),
            lfsr2: LFSR::new(seed2),
        }
    }

    pub fn next(&mut self) -> u16 {
        let a = self.lfsr1.next();
        let b = self.lfsr2.next();
        a ^ b
    }
}
