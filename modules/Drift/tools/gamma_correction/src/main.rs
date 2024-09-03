use std::{fs::File, io::Write};
use textplots::{Chart, Plot, Shape};

fn main() {
    let mut values = [0u8; 256];
    let gamma = 2.2;
    for i in 0..256 {
        let x = i as f64 / 255 as f64;
        let y = (x.powf(gamma) * 255.0).round();
        assert!(y >= 0.0);
        assert!(y <= 255.0);
        values[i] = y as u8;
    }

    Chart::new(128, 64, 0.0, 255.0)
        .lineplot(&Shape::Lines(
            &values
                .iter()
                .enumerate()
                .map(|(i, x)| (i as f32, *x as f32))
                .collect::<Vec<(f32, f32)>>(),
        ))
        .nice();

    let mut f = File::create("gamma_lut.bin").unwrap();
    for n in values {
        f.write(&[n]).unwrap();
    }
}
