use rand::Rng;

pub fn operands() -> impl Iterator<Item = i32> {
    [i32::MIN, 0, 1, i32::MAX / 2, i32::MAX].into_iter()
}

pub fn bools() -> impl Iterator<Item = bool> {
    [true, false].into_iter()
}

pub fn rand_operand<T>(mut count: usize) -> impl Iterator<Item = T>
where
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    std::iter::from_fn(move || {
        if count == 0 {
            return None;
        }

        count -= 1;
        let mut rng = rand::thread_rng();
        Some(rng.gen())
    })
}
