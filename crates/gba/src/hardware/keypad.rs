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

    pub fn set_key_state(&mut self, key: Key, state: KeyInputState) {
        match key {
            Key::A => self.set_button_a(state),
            Key::B => self.set_button_b(state),
            Key::Select => self.set_select(state),
            Key::Start => self.set_start(state),
            Key::Right => self.set_right(state),
            Key::Left => self.set_left(state),
            Key::Up => self.set_up(state),
            Key::Down => self.set_down(state),
            Key::R => self.set_button_r(state),
            Key::L => self.set_button_l(state),
        }
    }

    pub fn release_all(&mut self) {
        self.reset();
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Key {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
    R,
    L,
}

impl Key {
    pub const COUNT: usize = 10;
}

impl TryFrom<u8> for Key {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Key::A),
            1 => Ok(Key::B),
            2 => Ok(Key::Select),
            3 => Ok(Key::Start),
            4 => Ok(Key::Right),
            5 => Ok(Key::Left),
            6 => Ok(Key::Up),
            7 => Ok(Key::Down),
            8 => Ok(Key::R),
            9 => Ok(Key::L),
            _ => Err(()),
        }
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        match value {
            Key::A => 0,
            Key::B => 1,
            Key::Select => 2,
            Key::Start => 3,
            Key::Right => 4,
            Key::Left => 5,
            Key::Up => 6,
            Key::Down => 7,
            Key::R => 8,
            Key::L => 9,
        }
    }
}

impl TryFrom<usize> for Key {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let value: u8 = value.try_into().map_err(|_| ())?;
        value.try_into()
    }
}

impl From<Key> for usize {
    fn from(value: Key) -> Self {
        u8::from(value) as usize
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
