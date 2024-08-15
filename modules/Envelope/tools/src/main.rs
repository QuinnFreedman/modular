use std::{fs::File, io::Write};

/**
Computes the curve 2^x on the interval [0, 16), sampled at 256 uniform points,
encoded as (16.16) fixed point
*/
pub fn make_exp2_lut() -> [u32; 256] {
    let mut table = [0u32; 256];
    for i in 0..256 {
        let float_value = ((i as f64) / 256.0 * 16.0).exp2();
        table[i] = (float_value * (1 << 16) as f64).round() as u32;
    }
    table
}

/**
Computes the curve log_2( (x+1)/2 ) + 1 on the interval [0, 1), (which naturally
has the range [0, 1)) scaled to the range [0, 2^16), sampled at 256 uniform points,
rounded to 16 bit integers
*/
pub fn make_log2_lut() -> [u16; 256] {
    const TABLE_SIZE: usize = 256;
    let mut lookup_table: [u16; TABLE_SIZE] = [0; TABLE_SIZE];

    for i in 0..TABLE_SIZE {
        let value = (0.5 + (i as f64) / (TABLE_SIZE as f64) / 2.0) as f64;
        let log_value = (value.log2() + 1.0) * (u16::MAX as f64 + 1.0);
        assert!(log_value < u16::MAX as f64);
        lookup_table[i] = log_value.round() as u16;
    }

    lookup_table
}

fn main() {
    {
        let exp2_lut = make_exp2_lut();
        let mut f = File::create("exp2lut.bin").unwrap();
        for n in exp2_lut {
            f.write_all(&n.to_le_bytes()).unwrap();
        }
    }

    {
        let log2_lut = make_log2_lut();
        let mut f = File::create("log2lut.bin").unwrap();
        for n in log2_lut {
            f.write_all(&n.to_le_bytes()).unwrap();
        }
    }
}
