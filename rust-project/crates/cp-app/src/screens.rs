use egui::{pos2, vec2, Rect, ViewportBuilder};

#[derive(Clone)]
pub struct Screen {
    pub name: String,
    pub bounds: Rect,
    pub work: Rect,
    pub scale: f32,
    pub primary: bool,
}

pub fn screens() -> Vec<Screen> {
    let mut list = platform();
    list.sort_by_key(|s| !s.primary);
    if list.is_empty() {
        let bounds = Rect::from_min_size(pos2(0., 0.), vec2(1280., 720.));
        list.push(Screen {
            name: "显示器".into(),
            bounds,
            work: bounds,
            scale: 1.,
            primary: true,
        });
    }
    list
}

pub fn main_window(screen: &Screen) -> ViewportBuilder {
    let work = screen.work;
    let size = (work.size() / screen.scale * 0.86).min(vec2(1360., 860.));
    let pos = (work.center().to_vec2() / screen.scale - size / 2.).to_pos2();
    ViewportBuilder::default()
        .with_title("见证之窗 · ChurchPresenter")
        .with_inner_size(size)
        .with_position(pos)
        .with_maximized(false)
        .with_fullscreen(false)
        .with_min_inner_size(size.min(vec2(520., 360.)))
}

#[cfg(windows)]
fn platform() -> Vec<Screen> {
    use windows_sys::Win32::{
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW},
        UI::HiDpi::{
            GetDpiForMonitor, SetThreadDpiAwarenessContext,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, MDT_EFFECTIVE_DPI,
        },
    };
    unsafe extern "system" fn collect(
        monitor: HMONITOR,
        _dc: HDC,
        _rect: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if unsafe { GetMonitorInfoW(monitor, &mut info.monitorInfo) } != 0 {
            let to_rect = |r: RECT| {
                Rect::from_min_max(
                    pos2(r.left as f32, r.top as f32),
                    pos2(r.right as f32, r.bottom as f32),
                )
            };
            let (mut x, mut y) = (96, 96);
            unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y) };
            let len = info
                .szDevice
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(info.szDevice.len());
            // EnumDisplayMonitors synchronously supplies the vector passed below to each callback.
            let list = unsafe { &mut *(data as *mut Vec<Screen>) };
            list.push(Screen {
                name: String::from_utf16_lossy(&info.szDevice[..len]),
                bounds: to_rect(info.monitorInfo.rcMonitor),
                work: to_rect(info.monitorInfo.rcWork),
                scale: x.max(1) as f32 / 96.,
                primary: info.monitorInfo.dwFlags & 1 != 0,
            });
        }
        1
    }
    let mut list = Vec::new();
    unsafe {
        let old = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(collect),
            &mut list as *mut _ as LPARAM,
        );
        if !old.is_null() {
            SetThreadDpiAwarenessContext(old);
        }
    }
    list
}
#[cfg(not(windows))]
fn platform() -> Vec<Screen> {
    display_info::DisplayInfo::all()
        .unwrap_or_default()
        .into_iter()
        .map(|s| {
            let bounds = Rect::from_min_size(
                pos2(s.x as f32, s.y as f32),
                vec2(s.width as f32, s.height as f32),
            );
            Screen {
                name: s.name,
                bounds,
                work: bounds,
                scale: s.scale_factor.max(1.),
                primary: s.is_primary,
            }
        })
        .collect()
}
