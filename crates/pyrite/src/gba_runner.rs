use gba::{
    video::{ScreenBuffer, VISIBLE_LINE_WIDTH, VISIBLE_PIXELS},
    Gba, GbaVideoOutput,
};
use parking_lot::{Condvar, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use spin_sleep::LoopHelper;
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedGba {
    inner: Arc<RwLock<GbaData>>,
}

impl SharedGba {
    pub fn new() -> Self {
        let shared = SharedGba {
            inner: Arc::new(RwLock::new(GbaData {
                gba: Gba::new(),
                frame_buffer: Box::new([gba::video::rgb5(31, 0, 31); VISIBLE_PIXELS]),
                ready_buffer: Box::new([gba::video::rgb5(31, 0, 31); VISIBLE_PIXELS]),
                current_mode: GbaRunMode::Paused,
                paused_cond: Arc::default(),
                request_repaint: None,
                painted: false,
                profling_enabled: false,
            })),
        };

        let locked = shared.inner.write();
        let cloned_instance = shared.clone();

        std::thread::Builder::new()
            .name("gba".into())
            .spawn(move || gba_run_loop(cloned_instance))
            .unwrap();

        drop(locked);

        shared
    }

    pub fn unpause(&self) {
        let mut inner = self.inner.write();
        inner.current_mode = GbaRunMode::Run;
        inner.paused_cond.1.notify_all();
    }

    #[allow(dead_code)]
    pub fn pause(&self) {
        self.inner.write().current_mode = GbaRunMode::Paused;
    }

    #[allow(dead_code)]
    pub fn step(&self) {
        self.inner.write().current_mode = GbaRunMode::Step;
    }

    pub(crate) fn with<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&GbaData) -> T,
    {
        let locked = self.inner.read();
        (f)(&locked)
    }

    pub(crate) fn with_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut GbaData) -> T,
    {
        let mut locked = self.inner.write();
        (f)(&mut locked)
    }

    #[allow(dead_code)]
    pub fn read(&self) -> RwLockReadGuard<'_, GbaData> {
        self.inner.read()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, GbaData> {
        self.inner.write()
    }
}

pub struct GbaData {
    pub gba: Gba,
    /// The frame buffer that the GBA is currently drawing into.
    pub frame_buffer: Box<ScreenBuffer>,
    /// The last completed frame buffer ready for display.
    pub ready_buffer: Box<ScreenBuffer>,
    pub current_mode: GbaRunMode,
    paused_cond: Arc<(Mutex<()>, Condvar)>,

    /// This function will be called when the GBA wants to request a repaint.
    /// The first argument passed to the callback is the `ready` flag. When this
    /// is `true` the [`GbaData::ready_buffer`] should be displayed on screen. If is
    /// `false` the `frame_buffer` should be used instead.
    pub request_repaint: Option<Box<dyn Fn(bool) + Send + Sync>>,

    /// This is set to false before [`GbaData::request_repaint`] is called. It is
    /// the responsibility of whatever is doing the painting to set and maintain
    /// this flag in order to reduce work done.
    pub painted: bool,

    pub profling_enabled: bool,
}

fn gba_run_loop(gba: SharedGba) {
    tracing::debug!("starting GBA run loop");

    let mut loop_helper = LoopHelper::builder()
        .report_interval_s(1.0)
        .build_with_target_rate(60.0);
    loop {
        loop_helper.loop_start();
        if Arc::strong_count(&gba.inner) == 0 {
            tracing::debug!("no more references to shared GBA");
            break;
        }

        let mut data = gba.inner.write();
        match data.current_mode {
            GbaRunMode::Run => {
                gba_frame_tick(&mut data);
                RwLockWriteGuard::unlocked(&mut data, || loop_helper.loop_sleep());
            }
            GbaRunMode::Frame => {
                gba_frame_tick(&mut data);
                data.current_mode = GbaRunMode::Paused;
            }
            GbaRunMode::Step => {
                gba_step_tick(&mut data);
                data.current_mode = GbaRunMode::Paused;
            }
            GbaRunMode::Paused => {
                tracing::debug!("GBA paused");
                let paused_cond = Arc::clone(&data.paused_cond);
                let (lock, cvar) = &*paused_cond;
                RwLockWriteGuard::unlocked(&mut data, move || {
                    let mut locked = lock.lock();
                    cvar.wait(&mut locked);
                });
                tracing::debug!("GBA wakeup");
            }
            GbaRunMode::Shutdown => {
                tracing::debug!("explicit GBA shutdown requested");
            }
        };
    }

    tracing::debug!("shutdown GBA run loop");
}

fn gba_frame_tick(data: &mut GbaData) {
    let mut fb = FrameBuffer::new(&mut data.frame_buffer);
    let mut ab = gba::NoopGbaAudioOutput;

    {
        #[cfg(feature = "puffin")]
        puffin::GlobalProfiler::lock().new_frame();

        #[cfg(feature = "puffin")]
        puffin::profile_scope!("render_frame");

        while !fb.ready {
            data.gba.step(&mut fb, &mut ab);
        }
    }

    std::mem::swap::<Box<ScreenBuffer>>(&mut data.frame_buffer, &mut data.ready_buffer);

    if let Some(ref request_repaint) = data.request_repaint {
        data.painted = false;
        request_repaint(true);
    }
}

fn gba_step_tick(data: &mut GbaData) {
    let mut fb = FrameBuffer::new(&mut data.frame_buffer);
    let mut ab = gba::NoopGbaAudioOutput;
    data.gba.step(&mut fb, &mut ab);
    let frame_ready = fb.ready;

    if frame_ready {
        std::mem::swap::<Box<ScreenBuffer>>(&mut data.frame_buffer, &mut data.ready_buffer);
    }

    if let Some(ref request_repaint) = data.request_repaint {
        data.painted = false;
        request_repaint(frame_ready);
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GbaRunMode {
    Run,
    #[allow(dead_code)]
    Frame,
    Step,
    Paused,
    #[allow(dead_code)]
    Shutdown,
}

struct FrameBuffer<'b> {
    buffer: &'b mut ScreenBuffer,
    ready: bool,
}

impl<'b> FrameBuffer<'b> {
    fn new(buffer: &'b mut ScreenBuffer) -> Self {
        Self {
            buffer,
            ready: false,
        }
    }
}

impl<'b> GbaVideoOutput for FrameBuffer<'b> {
    fn gba_line_ready(&mut self, line: usize, data: &gba::video::LineBuffer) {
        let pos = VISIBLE_LINE_WIDTH * line;
        self.buffer[pos..(pos + VISIBLE_LINE_WIDTH)].copy_from_slice(data);
        if line == gba::video::VISIBLE_LINE_COUNT - 1 {
            self.ready = true;
        }
    }
}
