use core::{
    marker::PhantomData,
    sync::atomic::{AtomicI8, AtomicU8, Ordering},
};

use avr_progmem::{progmem, wrapper::ProgMem};

#[derive(Clone, Copy)]
struct NyblPair<A, B>
where
    A: From<u8>,
    B: From<u8>,
    A: ~const Into<u8>,
    B: ~const Into<u8>,
{
    data: u8,
    a: PhantomData<A>,
    b: PhantomData<B>,
}

impl<A, B> NyblPair<A, B>
where
    A: From<u8>,
    B: From<u8>,
    A: ~const Into<u8>,
    B: ~const Into<u8>,
{
    #[inline(always)]
    fn lsb(&self) -> B {
        let value = self.data & 0x0f;
        value.into()
    }
    #[inline(always)]
    fn msb(&self) -> A {
        let value = (self.data & 0xf0) >> 4;
        value.into()
    }
    #[inline(always)]
    fn as_tuple(&self) -> (A, B) {
        (self.msb(), self.lsb())
    }
    #[inline(always)]
    const fn new(msb: A, lsb: B) -> Self {
        let a_value: u8 = msb.into();
        let b_value: u8 = lsb.into();
        debug_assert!(a_value < 16);
        debug_assert!(b_value < 16);
        Self {
            data: a_value << 4 | b_value,
            a: PhantomData,
            b: PhantomData,
        }
    }
}

#[derive(Clone, Copy)]
enum RotaryState {
    Start = 0x0,
    CwBegin = 0x1,
    CwNext = 0x2,
    CwFinal = 0x3,
    CcwBegin = 0x4,
    CcwNext = 0x5,
    CcwFinal = 0x6,
}

impl const Into<u8> for RotaryState {
    fn into(self) -> u8 {
        self as u8
    }
}
impl const From<u8> for RotaryState {
    fn from(value: u8) -> Self {
        match value {
            const { Self::Start as u8 } => Self::Start,
            const { Self::CwFinal as u8 } => Self::CwFinal,
            const { Self::CwBegin as u8 } => Self::CwBegin,
            const { Self::CwNext as u8 } => Self::CwNext,
            const { Self::CcwBegin as u8 } => Self::CcwBegin,
            const { Self::CcwFinal as u8 } => Self::CcwFinal,
            const { Self::CcwNext as u8 } => Self::CcwNext,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
enum RotationTrigger {
    None = 0x0,
    Clockwise = 0x1,
    CounterClockwise = 0x2,
}

impl const Into<u8> for RotationTrigger {
    fn into(self) -> u8 {
        self as u8
    }
}
impl const From<u8> for RotationTrigger {
    fn from(value: u8) -> Self {
        match value {
            const { Self::Clockwise as u8 } => Self::Clockwise,
            const { Self::CounterClockwise as u8 } => Self::CounterClockwise,
            _ => Self::None,
        }
    }
}

progmem! {
    static progmem ROTARY_STATE_TABLE: [[NyblPair<RotationTrigger, RotaryState>; 4]; 7] = [
        // Start
        [
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
            NyblPair::new(RotationTrigger::None, RotaryState::CwBegin),
            NyblPair::new(RotationTrigger::None, RotaryState::CcwBegin),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
        ],
        // CwBegin
        [
            NyblPair::new(RotationTrigger::None, RotaryState::CwNext),
            NyblPair::new(RotationTrigger::None, RotaryState::CwBegin),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
        ],
        // CwNext
        [
            NyblPair::new(RotationTrigger::None, RotaryState::CwNext),
            NyblPair::new(RotationTrigger::None, RotaryState::CwBegin),
            NyblPair::new(RotationTrigger::None, RotaryState::CwFinal),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
        ],
        // CwFinal
        [
            NyblPair::new(RotationTrigger::None, RotaryState::CwNext),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
            NyblPair::new(RotationTrigger::None, RotaryState::CwFinal),
            NyblPair::new(RotationTrigger::Clockwise, RotaryState::Start),
        ],
        // CcwBegin
        [
            NyblPair::new(RotationTrigger::None, RotaryState::CcwNext),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
            NyblPair::new(RotationTrigger::None, RotaryState::CcwBegin),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
        ],
        // CcwNext
        [
            NyblPair::new(RotationTrigger::None, RotaryState::CcwNext),
            NyblPair::new(RotationTrigger::None, RotaryState::CcwFinal),
            NyblPair::new(RotationTrigger::None, RotaryState::CcwBegin),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
        ],
        // CcwFinal
        [
            NyblPair::new(RotationTrigger::None, RotaryState::CcwNext),
            NyblPair::new(RotationTrigger::None, RotaryState::CcwFinal),
            NyblPair::new(RotationTrigger::None, RotaryState::Start),
            NyblPair::new(RotationTrigger::CounterClockwise, RotaryState::Start),
        ],
    ];
}

pub struct RotaryEncoderHandler {
    state: AtomicU8,
    pub rotation: AtomicI8,
}

impl RotaryEncoderHandler {
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
            rotation: AtomicI8::new(0),
        }
    }
}

impl RotaryEncoderHandler {
    pub fn update(&self, pin1: bool, pin2: bool) {
        let pinstate: u8 = ((pin2 as u8) << 1) | pin1 as u8;
        let state = self.state.load(Ordering::SeqCst);
        let next_state = ROTARY_STATE_TABLE
            .at(state.into())
            .at(pinstate.into())
            .load();
        self.state.store(next_state.lsb().into(), Ordering::SeqCst);
        let rotation = self.rotation.load(Ordering::SeqCst);
        let delta = match next_state.msb() {
            RotationTrigger::None => 0,
            RotationTrigger::Clockwise => 1,
            RotationTrigger::CounterClockwise => -1,
        };
        self.rotation.store(rotation + delta, Ordering::SeqCst);
    }

    pub fn sample_and_reset(&self) -> i8 {
        if self.rotation.load(Ordering::SeqCst) != 0 {
            let delta = avr_device::interrupt::free(|_cs| {
                let current_value = self.rotation.load(Ordering::SeqCst);
                self.rotation.store(0, Ordering::SeqCst);
                current_value
            });
            delta
        } else {
            0
        }
    }
}
