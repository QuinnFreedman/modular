use fixed::{
    traits::{Fixed, FromFixed},
    types::{I0F16, I16F16, I1F15, U0F16},
};
use rand::prelude::*;
use textplots::{Chart, Plot, Shape};

fn main() {
    // show_perlin();
    make_svg();
    // let mut values = vec![];
    // for i in (0..u16::MAX).step_by(1 << 10) {
    //     let x = i as f32 / u16::MAX as f32;
    //     values.push((x, fade(I16F16::from_num(x)).to_num::<f32>()));
    // }
    // println!("{:?}", values);
    // return Chart::new(120, 60, 0.0, 1.0)
    //     .lineplot(&Shape::Lines(&values))
    //     .nice();
    // println!("{}", fade(I16F16::from_bits(u16::MAX as i32)))
}

// TODO convert fade to I1F15 -- cleaner code from less converting + can take advantage of expansion optimization
fn fade(t: U0F16) -> U0F16 {
    const SIX: U0F16 = U0F16::from_bits(6 << 12);
    const FIFTEEN: U0F16 = U0F16::from_bits(15 << 12);
    const TEN: U0F16 = U0F16::from_bits(10 << 12);
    debug_assert!(t < 1);
    debug_assert!(t >= 0);
    let t_3 = t * t * t;
    let t_4 = t_3 * t;
    let t_5 = t_4 * t;
    // println!("t_3: {t_3}, t_4: {t_4}, t_5: {t_5}");
    let result = (TEN * t_3).saturating_add(SIX * t_5) - FIFTEEN * t_4;
    // println!("result: {result} | {}", result.to_bits());
    let result_bits = u16::min(result.to_bits(), 4095);
    U0F16::from_bits(result_bits << 4)
}

fn fade1(t: I16F16) -> I16F16 {
    const SIX: I16F16 = I16F16::from_bits(6 << 16);
    const FIFTEEN: I16F16 = I16F16::from_bits(15 << 16);
    const TEN: I16F16 = I16F16::from_bits(10 << 16);
    // t * t * t * (t * (t * SIX - FIFTEEN) + TEN)
    // println!("{t} | {}", t.to_bits());
    debug_assert!(t < 1);
    debug_assert!(t >= 0);
    let t_3 = t * t * t;
    let t_4 = t_3 * t;
    let t_5 = t_4 * t;
    (TEN * t_3).saturating_add(SIX * t_5) - FIFTEEN * t_4
}

fn grad(hash: u8, x: I1F15) -> I1F15 {
    debug_assert!(x < 1);
    debug_assert!(x > -1);
    // let x_1_15 = I1F15::from_bits(((x.to_bits() as u16) >> 1) as i16);
    let h = hash & 15;
    let grad_int = 1 + (h & 7);
    // TODO
    // let grad = I16F16::from_bits(((grad_int as u16) << 14) as i32);
    // let grad = I16F16::from_num(grad_int) / 8;
    let grad = I1F15::from_bits(((grad_int as u16) << 11) as i16);
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

fn to_signed(x: U0F16) -> I1F15 {
    I1F15::from_bits((x.to_bits() >> 1) as i16)
}

fn perlin1d(_x: f32, permutation: &[u8]) -> f32 {
    let x = I16F16::from_num(_x);
    let xi = x.int().to_num::<i32>() & 255;
    let xf = U0F16::from_bits(x.frac().to_bits() as u16);

    let u = fade(xf);

    let a = permutation[xi as usize];
    let b = permutation[(xi + 1) as usize & 255];

    let xf_signed = to_signed(xf);
    const ONE: I1F15 = I1F15::from_bits(0x7FFF);
    let result = to_signed(u).lerp(grad(a, xf_signed), grad(b, xf_signed - ONE));
    I1F15::from_bits(result.to_bits() * 4).to_num()
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

fn make_svg() {
    use std::{fs::File, io::Write};

    let perm = generate_permutation_table();
    let mut path = String::from("");
    let mut path2 = String::from("");
    let num_samples = 300;
    for i in 0..num_samples {
        let x = ((i as f32) / (num_samples as f32)) * 8.0;
        let value = perlin1d(x, &perm) * 3.5;
        let octave = perlin1d(100.0 + x * 4.0, &perm) * 0.5;
        let y1 = 4.0 - value;
        let y2 = 4.0 - (value + octave);
        if path.len() == 0 {
            path = format!("M {},{}", x, y1);
            path2 = format!("M {},{}", x, y2);
        } else {
            path += format!("L {},{}", x, y1).as_str();
            path2 += format!("L {},{}", x, y2).as_str();
        }
    }

    let svg = format!(
        r#"<svg viewBox="0 0 8 8" xmlns="http://www.w3.org/2000/svg"><path stroke="black" fill="none" stroke-width="0.1" d="{}" /><path stroke="black" fill="none" stroke-width="0.1" d="{}" /></svg>"#,
        path, path2
    );
    let mut f = File::create("perlin.svg").unwrap();
    f.write(&svg.into_bytes()).unwrap();
}
