#[allow(unused_macros)]
macro_rules! test_combinations {
    (@ $body:block @ $current_name:ident in $current_thing:expr, $($name:ident in $thing:expr),+ ) => {
        for $current_name in $current_thing {
            test_combinations!(@ $body @ $($name in $thing),*);
        }
    };

    (@ $body:block @ $current_name:ident in $current_thing:expr ) => {
        for $current_name in $current_thing {
            $body
        }
    };

    (
        $(
            $(#[$meta:meta])*
            $vis:vis fn $fn_name:ident( $($name:ident in $thing:expr),* ) $body:block
        )*
    ) => {
        $(
            $(#[$meta])*
            $vis fn $fn_name() {
                test_combinations!(@ $body @ $($name in $thing),*)
            }
        )*
    };
}
