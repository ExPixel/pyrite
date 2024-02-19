use pyrite_derive::IoRegister;

#[derive(Default)]
pub struct Keypad {
    pub keyinput: RegKeyInput,
}

impl Keypad {
    pub fn reset(&mut self) {
        self.keyinput.reset();
    }
}

/// 4000130h - KEYINPUT - Key Status (R)
///   Bit   Expl.
///   0     Button A        (0=Pressed, 1=Released)
///   1     Button B        (etc.)
///   2     Select          (etc.)
///   3     Start           (etc.)
///   4     Right           (etc.)
///   5     Left            (etc.)
///   6     Up              (etc.)
///   7     Down            (etc.)
///   8     Button R        (etc.)
///   9     Button L        (etc.)
///   10-15 Not used
#[derive(IoRegister, Copy, Clone)]
#[field(button_a: KeyInputState = 0)]
#[field(button_b: KeyInputState = 1)]
#[field(select: KeyInputState = 2)]
#[field(start: KeyInputState = 3)]
#[field(right: KeyInputState = 4)]
#[field(left: KeyInputState = 5)]
#[field(up: KeyInputState = 6)]
#[field(down: KeyInputState = 7)]
#[field(button_r: KeyInputState = 8)]
#[field(button_l: KeyInputState = 9)]
pub struct RegKeyInput {
    value: u16,
}

impl RegKeyInput {
    pub fn reset(&mut self) {
        self.set_button_a(KeyInputState::Released);
        self.set_button_b(KeyInputState::Released);
        self.set_select(KeyInputState::Released);
        self.set_start(KeyInputState::Released);
        self.set_right(KeyInputState::Released);
        self.set_left(KeyInputState::Released);
        self.set_up(KeyInputState::Released);
        self.set_down(KeyInputState::Released);
        self.set_button_r(KeyInputState::Released);
        self.set_button_l(KeyInputState::Released);
    }
}

#[derive(Copy, Clone)]
pub enum KeyInputState {
    Released,
    Pressed,
}

impl From<u16> for KeyInputState {
    fn from(value: u16) -> Self {
        match value {
            0 => KeyInputState::Pressed,
            1 => KeyInputState::Released,
            _ => unreachable!(),
        }
    }
}

impl From<KeyInputState> for u16 {
    fn from(value: KeyInputState) -> Self {
        match value {
            KeyInputState::Pressed => 0,
            KeyInputState::Released => 1,
        }
    }
}
