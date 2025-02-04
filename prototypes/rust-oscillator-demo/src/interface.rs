pub trait SoundAlgorithm: Send {
    fn get_name(&self) -> &'static str;
    fn generate_sample(&mut self) -> f32;
    fn debug_get_freq(&mut self) -> f32;
    fn debug_get_and_clear_cycle_flag(&mut self) -> bool;
    fn set_sample_rate(&mut self, sample_rate: u32);

    fn parameters(&self) -> Vec<SoundParameter>;
    fn update_parameter(&mut self, name: &str, value: f32);
}

pub struct SoundParameter {
    pub value: f32,
    pub name: &'static str,
    pub param_type: ParamType,
}

pub enum ParamType {
    Float { min: f32, max: f32 },
    Bool,
    Select(&'static [&'static str]),
}
