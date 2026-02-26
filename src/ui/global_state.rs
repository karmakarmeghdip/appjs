use std::sync::OnceLock;
use masonry::vello::wgpu;
use masonry_winit::app::{EventLoopProxy, WindowId};

#[derive(Clone)]
pub struct ClonedWgpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

pub struct VellumGlobalState {
    pub wgpu: Option<ClonedWgpu>,
    pub proxy: EventLoopProxy,
    pub window_id: WindowId,
}

pub static GLOBAL_STATE: OnceLock<std::sync::Mutex<VellumGlobalState>> = OnceLock::new();

pub fn init_global_app_context(proxy: EventLoopProxy, window_id: WindowId) {
    let _ = GLOBAL_STATE.set(std::sync::Mutex::new(VellumGlobalState {
        wgpu: None,
        proxy,
        window_id,
    }));
}

pub fn set_global_wgpu(device: wgpu::Device, queue: wgpu::Queue) {
    if let Some(state) = GLOBAL_STATE.get() {
        if let Ok(mut lock) = state.lock() {
            lock.wgpu = Some(ClonedWgpu { device, queue });
        }
    }
}

pub fn get_wgpu_context() -> Option<ClonedWgpu> {
    GLOBAL_STATE.get()
        .and_then(|state| state.lock().ok())
        .and_then(|lock| lock.wgpu.clone())
}

pub fn get_event_loop_proxy() -> Option<(EventLoopProxy, WindowId)> {
    GLOBAL_STATE.get()
        .and_then(|state| state.lock().ok())
        .map(|lock| (lock.proxy.clone(), lock.window_id))
}
