/**
Utility for handling rotary encoder input. Rotary encoders encode their relative
position based on a state machine of two input wires. But, rotary encoders can
be very "bouncy"; they can give erroneous inputs especially on a breadboard. This
library uses a clever lookup from http://www.buxtronix.net/2011/10/rotary-encoders-done-properly.html
to ensure that impossible state transitions are ignored, which makes the encoder
much more accurate.
*/
use core::sync::atomic::{AtomicI8, AtomicU8, AtomicBool, Ordering};

use crate::{nybl_pair::{NyblPair, ConstNyblPair}, const_traits::{ConstInto, ConstFrom}};
use avr_progmem::progmem;

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

impl const ConstInto<u8> for RotaryState {
    fn const_into(self) -> u8 {
        self as u8
    }
}

impl const ConstFrom<u8> for RotaryState {
    fn const_from(value: u8) -> Self {
        match value {
            const { Self::Start as u8 } => Self::Start,
            const { Self::CwBegin as u8 } => Self::CwBegin,
            const { Self::CwNext as u8 } => Self::CwNext,
            const { Self::CwFinal as u8 } => Self::CwFinal,
            const { Self::CcwBegin as u8 } => Self::CcwBegin,
            const { Self::CcwNext as u8 } => Self::CcwNext,
            const { Self::CcwFinal as u8 } => Self::CcwFinal,
            // This code is reachable if you pass in a random u8. But, this
            // struct is private, and this impl is only used to store/retrieve
            // RotaryStates from memory, so it will never be called.
            // I wasn't sure if the compiler would be smart enough to optimize
            // away a "panic!()" here, and performance is very important here
            // since it will be in an interrupt.
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
enum RotationTrigger {
    None = 0x0,
    Clockwise = 0x1,
    CounterClockwise = 0x2,
}

impl const ConstInto<u8> for RotationTrigger {
    fn const_into(self) -> u8 {
        self as u8
    }
}
impl const ConstFrom<u8> for RotationTrigger {
    fn const_from(value: u8) -> Self {
        match value {
            const { Self::Clockwise as u8 } => Self::Clockwise,
            const { Self::CounterClockwise as u8 } => Self::CounterClockwise,
            _ => Self::None,
        }
    }
}

progmem! {
    /**
    The state machine lookup table. Each of the 7 rows corresponds to one of the
    7 possible states for the state machine. Each column is one of the 4 possible
    combinations of inputs from the 2 input pins. Each cell indicates the result
    of seeing that input in that state. Each cell is a pair. The first item is the
    output to return (wheter the rotary encoder has completed a rotation). The
    second item is the next state to go to.
    */
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

#[repr(align(1))]
pub struct RotaryEncoderHandler {
    invert: AtomicBool,
    state: AtomicU8,
    pub rotation: AtomicI8,
}

impl RotaryEncoderHandler {
    pub const fn new() -> Self {
        Self {
            invert: AtomicBool::new(false),
            state: AtomicU8::new(0),
            rotation: AtomicI8::new(0),
        }
    }
}

impl RotaryEncoderHandler {
    /**
    Updates the rotary encoder state based on the readings from the
    two input pins. Should be called whenever either pin changes,
    ideally in a pin-change interrupt handler.
    */
    pub fn update(&self, pin1: bool, pin2: bool) {
        let pinstate: u8 = ((pin2 as u8) << 1) | pin1 as u8;
        let state = self.state.load(Ordering::SeqCst);
        let next_state = ROTARY_STATE_TABLE
            .at(state.into())
            .at(pinstate.into())
            .load();
        self.state.store(next_state.lsbs().const_into(), Ordering::SeqCst);
        let rotation = self.rotation.load(Ordering::SeqCst);
        let invert = self.invert.load(Ordering::SeqCst);
        let delta = match next_state.msbs() {
            RotationTrigger::None => 0,
            RotationTrigger::Clockwise => if invert { -1 } else { 1 },
            RotationTrigger::CounterClockwise => if invert { 1 } else { -1 },
        };
        self.rotation.store(rotation + delta, Ordering::SeqCst);
    }

    /**
    Returns the total number of detents that the encoder has rotated
    (positive or negative) since the last time this function was called,
    then resets the number to zero. Should be called periodically in the
    main loop of the program.
    */
    pub fn sample_and_reset(&self) -> i8 {
        if self.rotation.load(Ordering::SeqCst) == 0 {
            return 0;
        }

        avr_device::interrupt::free(|_cs| {
            let current_value = self.rotation.load(Ordering::SeqCst);
            self.rotation.store(0, Ordering::SeqCst);
            current_value
        })
    }

    pub fn invert(&self) {
        let invert = self.invert.load(Ordering::SeqCst);
        self.invert.store(!invert, Ordering::SeqCst);
    }
}
