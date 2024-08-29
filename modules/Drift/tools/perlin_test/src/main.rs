use fixed::types::I16F16;
use rand::prelude::*;
use textplots::{Chart, Plot, Shape};

fn main() {
    show_perlin();
}

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

// fn grad(hash: u8, x: I16F16) -> I16F16 {
//     const ZERO: I16F16 = I16F16::from_bits(0);
//     match hash & 15 {
//         0 => x,
//         1 => -x,
//         2 => x,
//         3 => -x,
//         4 => x,
//         5 => -x,
//         6 => x,
//         7 => -x,
//         8 => ZERO,
//         9 => ZERO,
//         10 => ZERO,
//         11 => ZERO,
//         12 => x,
//         13 => ZERO,
//         14 => -x,
//         15 => ZERO,
//         _ => unreachable!(),
//     }
// }

fn perlin1d(_x: f32, permutation: &[u8]) -> f32 {
    let x = I16F16::from_num(_x);
    let xi = x.int().to_num::<i32>() & 255;
    let xf = x.frac();

    let u = fade(xf);

    let a = permutation[xi as usize];
    let b = permutation[(xi + 1) as usize & 255];

    u.lerp(grad(a, xf), grad(b, xf - I16F16::from_num(1)))
        .to_num()
}

fn generate_permutation_table() -> Vec<u8> {
    let mut p = (0..=255).collect::<Vec<u8>>();
    let mut rng = StdRng::seed_from_u64(0);
    p.shuffle(&mut rng);
    p
}

fn show_perlin() {
    let perm = generate_permutation_table();
    let mut values = vec![];
    for i in (0..u32::MAX).step_by(1 << 12) {
        let x = I16F16::from_bits((i >> 12) as i32).to_num::<f32>();
        values.push((x, perlin1d(x, &perm)));
    }
    return Chart::new(120, 60, 0.0, 16.0)
        .lineplot(&Shape::Lines(&values))
        .nice();
}
