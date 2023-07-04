use proptest::strategy::Strategy;

pub fn operand() -> impl Strategy<Value = u32> {
    const VALUES: &[u32] = &[
        0, 1, 2, 0x00BEEF00, 0x7FFFFFFF, 0xFFFFFFFC, 0xFFFFFFFE, 0xFFFFFFFF,
    ];

    proptest::sample::select(VALUES)
}
