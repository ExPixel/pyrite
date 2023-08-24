#[macro_export]
macro_rules! mark_frame_end {
    () => {
        $crate::tracy_client::Client::running()
            .expect("mark_frame_start without running Tracy client")
            .frame_mark()
    };
}

#[macro_export]
macro_rules! scope {
    // Note: literal patterns provided as an optimization since they can skip an allocation.
    ($name:literal) => {
        // Note: callstack_depth is 0 since this has significant overhead
        let _tracy_span = $crate::tracy_client::span!($name, 0);
    };
    ($name:literal, $data:expr) => {
        // Note: callstack_depth is 0 since this has significant overhead
        let _tracy_span = $crate::tracy_client::span!($name, 0);
        _tracy_span.emit_text($data);
    };
    ($name:expr) => {
        let function_name = {
            struct S;
            let type_name = core::any::type_name::<S>();
            &type_name[..type_name.len() - 3]
        };
        let _tracy_span = $crate::tracy_client::Client::running()
            .expect("scope! without a running tracy_client::Client")
            // Note: callstack_depth is 0 since this has significant overhead
            .span_alloc(Some($name), function_name, file!(), line!(), 0);
    };
    ($name:expr, $data:expr) => {
        let function_name = {
            struct S;
            let type_name = core::any::type_name::<S>();
            &type_name[..type_name.len() - 3]
        };
        let _tracy_span = $crate::tracy_client::Client::running()
            .expect("scope! without a running tracy_client::Client")
            // Note: callstack_depth is 0 since this has significant overhead
            .span_alloc(Some($name), function_name, file!(), line!(), 0);
        _tracy_span.emit_text($data);
    };
}

#[macro_export]
macro_rules! register_thread {
    () => {
        let thread_name = std::thread::current()
            .name()
            .map(|x| x.to_string())
            .unwrap_or_else(|| format!("Thread {:?}", std::thread::current().id()));

        $crate::register_thread!(&thread_name);
    };
    ($name:expr) => {
        $crate::tracy_client::Client::running()
            .expect("register_thread! without a running tracy_client::Client")
            .set_thread_name($name);
    };
}

pub fn init() -> crate::Handle {
    crate::Handle(crate::HandleInner::Tracy(tracy_client::Client::start()))
}
