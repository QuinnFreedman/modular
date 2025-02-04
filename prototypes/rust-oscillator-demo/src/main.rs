mod complex_osc;
mod harmonic_wt_osc;
mod interface;
mod simplex_terrain;
mod sin_oscillator;
mod utils;

use complex_osc::ComplexOscillator;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui::{containers, CentralPanel, ComboBox, Context, Slider};
use eframe::emath;
use eframe::emath::{pos2, vec2, Pos2};
use eframe::epaint::{self, Color32, PathStroke, Rect};
use harmonic_wt_osc::HarmonicWtOsc;
use interface::SoundAlgorithm;
use simplex_terrain::SimplexOscillator;
use sin_oscillator::SinOscillator;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

struct AudioGenerationSate {
    selected_algorithm: usize,
    algorithms: Vec<Box<dyn SoundAlgorithm>>,
    volume: f32,
}

impl AudioGenerationSate {
    fn get_selected_algorithm(&mut self) -> &mut dyn SoundAlgorithm {
        self.algorithms[self.selected_algorithm].as_mut()
    }
}

struct UiState {
    visualizer_data: [f32; 100],
}

struct AppState {
    audio_generation_state: Arc<Mutex<AudioGenerationSate>>,
    ui_state: Arc<Mutex<UiState>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            audio_generation_state: Arc::new(Mutex::new(AudioGenerationSate {
                selected_algorithm: 0,
                algorithms: vec![
                    Box::new(SinOscillator::new()),
                    Box::new(HarmonicWtOsc::new()),
                    Box::new(ComplexOscillator::new()),
                    Box::new(SimplexOscillator::new()),
                ],
                volume: 0.25,
            })),
            ui_state: Arc::new(Mutex::new(UiState {
                visualizer_data: [0.0; 100],
            })),
        }
    }
}

fn main() {
    let app_state = AppState::new();

    let state_clone = AppState {
        audio_generation_state: Arc::clone(&app_state.audio_generation_state),
        ui_state: Arc::clone(&app_state.ui_state),
    };

    thread::spawn(move || {
        start_audio_thread(state_clone);
    });

    start_ui(app_state);
}

fn start_audio_thread(app_state: AppState) {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = device.default_output_config().unwrap();

    let sample_rate = config.sample_rate().0;
    {
        let mut state = app_state.audio_generation_state.lock().unwrap();
        for algo in state.algorithms.iter_mut() {
            algo.set_sample_rate(sample_rate);
        }
    }
    println!("Sample rate: {}", sample_rate);

    let (tx, rx) = mpsc::channel::<(bool, f32)>();

    let mut sample_counter = Box::<f32>::new(0.0);

    let audio_state = app_state.audio_generation_state;

    let debug_num_visualizer_samples = {
        let ui_state = app_state.ui_state.lock().unwrap();
        ui_state.visualizer_data.len()
    };

    const CYCLES_PER_DEBUG_WINDOW: u32 = 3;

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut state = audio_state.lock().unwrap();
                let volume = state.volume;
                let algorithm = state.get_selected_algorithm();

                let debug_target_sample_rate = sample_rate as f32
                    / ((algorithm.debug_get_freq() / CYCLES_PER_DEBUG_WINDOW as f32)
                        * debug_num_visualizer_samples as f32);

                for sample in data.iter_mut() {
                    let unscaled = algorithm.generate_sample();
                    *sample = unscaled * volume;
                    *sample_counter += 1.0;
                    if *sample_counter >= debug_target_sample_rate {
                        *sample_counter -= debug_target_sample_rate;
                        tx.send((algorithm.debug_get_and_clear_cycle_flag(), unscaled * 0.75))
                            .unwrap();
                    }
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )
        .expect("Failed to build audio stream");

    stream.play().expect("Failed to play audio stream");

    thread::spawn(move || {
        let mut ptr: usize = 0;
        let mut samples = [0f32; 100];
        let mut cycle_ctr: u32 = 0;

        while let Ok((did_rollover, sample)) = rx.recv() {
            if did_rollover {
                cycle_ctr += 1;
                if cycle_ctr == CYCLES_PER_DEBUG_WINDOW {
                    cycle_ctr = 0;
                    ptr = 0;

                    let mut state = app_state.ui_state.lock().unwrap();
                    state.visualizer_data.copy_from_slice(&samples);
                }
            }
            if ptr < samples.len() {
                samples[ptr] = sample;
                ptr += 1;
            }
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn start_ui(app_state: AppState) {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Audio Synth UI",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp {
                app_state: Arc::new(app_state),
            }))
        }),
    )
    .unwrap();
}

struct MyApp {
    app_state: Arc<AppState>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            {
                let mut state = self.app_state.audio_generation_state.lock().unwrap();

                let algorithm_names: Vec<&str> =
                    state.algorithms.iter().map(|x| x.get_name()).collect();
                ComboBox::from_label("Algorithm")
                    .selected_text(state.get_selected_algorithm().get_name())
                    .show_ui(ui, |ui| {
                        for (i, name) in algorithm_names.iter().enumerate() {
                            ui.selectable_value(&mut state.selected_algorithm, i, *name);
                        }
                    });

                let algorithm = state.get_selected_algorithm();
                for param in algorithm.parameters() {
                    let mut val = param.value;
                    match param.param_type {
                        interface::ParamType::Float { min, max } => {
                            if ui
                                .add(
                                    Slider::new(&mut val, min..=max)
                                        .drag_value_speed(0.01)
                                        .text(param.name),
                                )
                                .changed()
                            {
                                algorithm.update_parameter(param.name, val);
                            }
                        }
                        interface::ParamType::Bool => todo!(),
                        interface::ParamType::Select(options) => {
                            ComboBox::from_label(param.name)
                                .selected_text(options[param.value as usize])
                                .show_ui(ui, |ui| {
                                    for (i, name) in options.iter().enumerate() {
                                        if ui.selectable_value(&mut val, i as f32, *name).changed()
                                        {
                                            algorithm.update_parameter(param.name, val);
                                        }
                                    }
                                });
                        }
                    };
                }

                ui.add(Slider::new(&mut state.volume, 0.0..=0.5).text("volume"));
            }

            containers::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.ctx().request_repaint();

                let desired_size = ui.available_width() * vec2(1.0, 0.35);
                let (_id, rect) = ui.allocate_space(desired_size);

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                    rect,
                );

                let mut shapes = vec![];

                let points: Vec<Pos2> = {
                    let ui_state = self.app_state.ui_state.lock().unwrap();
                    let num_samples = ui_state.visualizer_data.len();

                    ui_state
                        .visualizer_data
                        .iter()
                        .enumerate()
                        .map(|(i, y)| {
                            let t = i as f64 / (num_samples as f64);
                            to_screen * pos2(t as f32, -*y)
                        })
                        .collect()
                };

                let color = if ui.visuals().dark_mode {
                    Color32::from_additive_luminance(196)
                } else {
                    Color32::from_black_alpha(240)
                };
                let thickness = 5.0;
                shapes.push(epaint::Shape::line(
                    points,
                    PathStroke::new(thickness, color),
                ));

                ui.painter().extend(shapes);
            });
        });
    }
}
