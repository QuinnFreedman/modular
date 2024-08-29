use std::{fs::File, io::Write};

use fixed::{
    traits::{Fixed, ToFixed},
    types::{
        extra::{U15, U16},
        I16F16,
    },
    FixedI16, FixedI32, FixedU16,
};
use rand::prelude::*;
use textplots::{AxisBuilder, Chart, Plot, Shape};

fn triangle_icdf(u: i16) -> i16 {
    // assert!(u <= i16::MAX as u16);
    assert!(u >= 0);

    let p = f64::from(u) / f64::from(i16::MAX);

    let result = if p < 0.5 {
        -1.0 + (2.0 * p).sqrt()
    } else {
        1.0 - (2.0 * (1.0 - p)).sqrt()
    };

    debug_assert!(result <= 1.0);
    (result * f64::from(i16::MAX)).round() as i16
}

fn plot_hist() {
    let mut rng = rand::thread_rng();
    let mut values = vec![];
    for i in 0..10000 {
        let u = rng.gen_range(0..=i16::MAX);
        let transformed = triangle_icdf(u);
        values.push((i as f32, transformed as f32));
    }

    let hist = textplots::utils::histogram(values.as_slice(), i16::MIN as f32, i16::MAX as f32, 16);
    return Chart::new(120, 60, i16::MIN as f32, i16::MAX as f32)
        .lineplot(&Shape::Bars(&hist))
        .nice();
}

fn make_lut() -> [i16; 256] {
    let mut lut = [0i16; 256];
    for i in 0..=255 {
        let u = i << 7;
        debug_assert!(u <= i16::MAX);
        debug_assert!(u >= 0);
        lut[i as usize] = triangle_icdf(u as i16);
    }
    lut
}

fn save_lut() {
    let lut = make_lut();
    let mut f = File::create("icdf_lut.bin").unwrap();
    for n in lut {
        f.write_all(&n.to_le_bytes()).unwrap();
    }
}

fn plot_icdf() {
    let lut = make_lut();
    let mut values = vec![];
    for i in 0..i16::MAX as u16 {
        values.push((i as f32, rand_icdf(i, &lut).to_num::<f32>()))
    }

    let mut f = File::create("values.csv").unwrap();
    for (x, y) in &values {
        writeln!(&mut f, "{},{}", x, y).unwrap();
    }

    return Chart::new(120, 60, i16::MIN as f32, i16::MAX as f32)
        .lineplot(&Shape::Lines(&values))
        .nice();
}

const LUT_SIZE: usize = 256;
const I16_BYTES: usize = i16::BITS as usize / 8;

fn rand_icdf(u: u16, lut: &[i16; LUT_SIZE]) -> FixedI16<U15> {
    debug_assert!(u <= i16::MAX as u16);
    let idx_low = u >> 7;
    let idx_high = u16::min(LUT_SIZE as u16 - 1, idx_low + 1);

    let remainder = FixedI16::<U15>::from_bits(((u << 8) & 0x7FFF) as i16);

    let v_low = FixedI16::<U15>::from_bits(lut[idx_low as usize]);
    let v_high = FixedI16::<U15>::from_bits(lut[idx_high as usize]);
    remainder.lerp(v_low, v_high)
}

fn main() {
    // plot_hist();
    // plot_icdf();
    // save_lut();
    show_perlin();
}

use rand::Rng;

fn fade(t: I16F16) -> I16F16 {
    const SIX: I16F16 = I16F16::from_bits(6 << 16);
    const FIFTEEN: I16F16 = I16F16::from_bits(15 << 16);
    const TEN: I16F16 = I16F16::from_bits(10 << 16);
    t * t * t * (t * (t * SIX - FIFTEEN) + TEN)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn grad(hash: u8, x: I16F16) -> I16F16 {
    let h = hash & 15;
    let grad = I16F16::from_num(1 + (h & 7));
    if (h & 8) != 0 {
        -grad * x
    } else {
        grad * x
    }
}

fn perlin1d(_x: f32, permutation: &[u8]) -> f32 {
    let x = I16F16::from_num(_x);
    let xi = x.int().to_num::<i32>() & 255;
    let xf = x.frac();

    let u = fade(xf);

    let a = permutation[xi as usize] as usize;
    let b = permutation[(xi + 1) as usize & 255] as usize;

    u.lerp(
        grad(permutation[a], xf),
        grad(permutation[b], xf - I16F16::from_num(1)),
    )
    .to_num()
}

fn generate_permutation_table() -> Vec<u8> {
    let mut p = (0..=255).collect::<Vec<u8>>();
    // let mut rng = rand::thread_rng();
    let mut rng = StdRng::seed_from_u64(0);
    p.shuffle(&mut rng);
    // p.extend_from_slice(&p);
    p
}

fn show_perlin() {
    let perm = generate_permutation_table();
    let mut values = vec![];
    for i in (0..u32::MAX).step_by(1 << 12) {
        let x = I16F16::from_bits((i >> 13) as i32).to_num::<f32>();
        values.push((x, perlin1d(x, &perm)));
    }
    return Chart::new(120, 60, 0.0, (u32::MAX / (1 << 29)) as f32)
        .lineplot(&Shape::Lines(&values))
        .nice();
}
