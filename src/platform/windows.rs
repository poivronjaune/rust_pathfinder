use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BeginPaint, BitBlt, COLOR_WINDOW, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, EndPaint, FillRect, GDI_ERROR,
    GetDC, GetSysColorBrush, HBITMAP, HDC, HGDIOBJ, InvalidateRect, PAINTSTRUCT, ReleaseDC,
    SRCCOPY, SelectObject, StretchDIBits,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{
    ICC_BAR_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx, SB_SETTEXTW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateMenu, CreatePopupMenu,
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetClientRect, GetMessageW,
    HMENU, IDC_ARROW, LoadCursorW, MB_ICONINFORMATION, MB_OK, MF_POPUP, MF_STRING, MSG,
    MessageBoxW, MoveWindow, PostQuitMessage, RegisterClassW, SW_SHOW, SetWindowLongPtrW,
    ShowWindow, TranslateMessage, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN,
    WM_NCCREATE, WM_PAINT, WM_SIZE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use crate::app::{AppState, VehicleKey};
use crate::config::{
    APP_AUTHOR, APP_COPYRIGHT, APP_DESCRIPTION, APP_NAME, APP_VERSION, APP_WEBSITE, HEIGHT, WIDTH,
};

const ID_FILE_NEW: usize = 1001;
const ID_FILE_SAVE: usize = 1002;
const ID_FILE_LOAD: usize = 1003;
const ID_FILE_EXIT: usize = 1004;
const ID_MAP_EDIT: usize = 1101;
const ID_ABOUT: usize = 1201;
const STATUS_HEIGHT: i32 = 24;
const WINDOW_CLASS: &str = "PathPlannerGridWindow";

struct WindowState {
    app: AppState,
    status_bar: HWND,
    back_buffer: Option<BackBuffer>,
}

struct BackBuffer {
    dc: HDC,
    bitmap: HBITMAP,
    previous_bitmap: HGDIOBJ,
    width: i32,
    height: i32,
}

pub(super) fn run() -> Result<(), String> {
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        return Err("Unable to get the application instance.".to_owned());
    }

    let controls = INITCOMMONCONTROLSEX {
        dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_BAR_CLASSES,
    };
    if unsafe { InitCommonControlsEx(&controls) } == 0 {
        return Err("Unable to initialize the Windows common controls.".to_owned());
    }

    let class_name = wide(WINDOW_CLASS);
    let mut window_class: WNDCLASSW = unsafe { zeroed() };
    window_class.style = CS_HREDRAW | CS_VREDRAW;
    window_class.lpfnWndProc = Some(window_proc);
    window_class.hInstance = instance;
    window_class.hCursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    window_class.lpszClassName = class_name.as_ptr();
    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err("Unable to register the main window class.".to_owned());
    }

    let menu = create_menu()?;
    let mut state = Box::new(WindowState {
        app: AppState::new(),
        status_bar: null_mut(),
        back_buffer: None,
    });
    let state_pointer = (&mut *state) as *mut WindowState;
    let title = wide("20 x 20 Grid");
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW
                | WS_VISIBLE
                | windows_sys::Win32::UI::WindowsAndMessaging::WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            480,
            500,
            null_mut(),
            menu,
            instance,
            state_pointer.cast(),
        )
    };
    if hwnd.is_null() {
        return Err("Unable to create the main window.".to_owned());
    }
    let state_pointer = Box::into_raw(state);
    unsafe { ShowWindow(hwnd, SW_SHOW) };

    let mut message: MSG = unsafe { zeroed() };
    let result = loop {
        let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if result == -1 {
            break Err("An error occurred while receiving a Windows message.".to_owned());
        }
        if result == 0 {
            break Ok(());
        }
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    };
    unsafe { drop(Box::from_raw(state_pointer)) };
    result
}

fn create_menu() -> Result<HMENU, String> {
    let menu_bar = unsafe { CreateMenu() };
    let file_menu = unsafe { CreatePopupMenu() };
    let map_menu = unsafe { CreatePopupMenu() };
    if menu_bar.is_null() || file_menu.is_null() || map_menu.is_null() {
        return Err("Unable to create the application menus.".to_owned());
    }

    let commands = [
        ("New", ID_FILE_NEW),
        ("Save", ID_FILE_SAVE),
        ("Load", ID_FILE_LOAD),
        ("Exit", ID_FILE_EXIT),
    ];
    for (label, command) in commands {
        let label = wide(label);
        if unsafe { AppendMenuW(file_menu, MF_STRING, command, label.as_ptr()) } == 0 {
            return Err("Unable to add a File menu command.".to_owned());
        }
    }
    let edit_label = wide("Edit");
    if unsafe { AppendMenuW(map_menu, MF_STRING, ID_MAP_EDIT, edit_label.as_ptr()) } == 0 {
        return Err("Unable to add the Map menu command.".to_owned());
    }

    let file_label = wide("File");
    let map_label = wide("Map");
    let about_label = wide("About");
    if unsafe { AppendMenuW(menu_bar, MF_POPUP, file_menu as usize, file_label.as_ptr()) } == 0
        || unsafe { AppendMenuW(menu_bar, MF_POPUP, map_menu as usize, map_label.as_ptr()) } == 0
        || unsafe { AppendMenuW(menu_bar, MF_STRING, ID_ABOUT, about_label.as_ptr()) } == 0
    {
        return Err("Unable to assemble the application menu bar.".to_owned());
    }
    Ok(menu_bar)
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = unsafe {
                &*(lparam as *const windows_sys::Win32::UI::WindowsAndMessaging::CREATESTRUCTW)
            };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };
            1
        }
        WM_CREATE => {
            let status_class = wide("msctls_statusbar32");
            let state = unsafe { state_mut(hwnd) };
            let Some(state) = state else {
                return -1;
            };
            state.status_bar = unsafe {
                CreateWindowExW(
                    0,
                    status_class.as_ptr(),
                    null(),
                    windows_sys::Win32::UI::WindowsAndMessaging::WS_CHILD
                        | windows_sys::Win32::UI::WindowsAndMessaging::WS_VISIBLE
                        | windows_sys::Win32::UI::Controls::SBARS_SIZEGRIP,
                    0,
                    0,
                    0,
                    0,
                    hwnd,
                    null_mut(),
                    GetModuleHandleW(null()),
                    null(),
                )
            };
            if state.status_bar.is_null() {
                return -1;
            }
            update_status(state);
            0
        }
        WM_SIZE => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                resize_status(hwnd, state.status_bar);
                if let Err(error) = rebuild_back_buffer(hwnd, state) {
                    eprintln!("Unable to resize the off-screen grid buffer: {error}");
                    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
                } else {
                    unsafe { InvalidateRect(hwnd, null(), 0) };
                }
            }
            0
        }
        WM_COMMAND => {
            let command = wparam & 0xffff;
            let caption = match command {
                ID_FILE_NEW => "New",
                ID_FILE_SAVE => "Save",
                ID_FILE_LOAD => "Load",
                ID_FILE_EXIT => "Exit",
                ID_MAP_EDIT => "Edit",
                ID_ABOUT => {
                    show_about(hwnd);
                    return 0;
                }
                _ => return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
            };
            show_placeholder(hwnd, caption);
            0
        }
        WM_KEYDOWN => {
            if lparam & (1 << 30) != 0 {
                return 0;
            }
            if wparam == 0x1b {
                unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
                return 0;
            }
            let key = match wparam {
                0x26 => Some(VehicleKey::Forward),
                0x28 => Some(VehicleKey::Reverse),
                0x25 => Some(VehicleKey::TurnLeft),
                0x27 => Some(VehicleKey::TurnRight),
                _ => None,
            };
            if let (Some(key), Some(state)) = (key, unsafe { state_mut(hwnd) }) {
                let changed_cells = state.app.handle_vehicle_key(key);
                update_status(state);
                if let Err(error) = update_changed_cells(hwnd, state, changed_cells) {
                    eprintln!("Unable to update the off-screen grid buffer: {error}");
                    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
                }
            }
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                paint(hwnd, state);
            }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe fn state_mut(hwnd: HWND) -> Option<&'static mut WindowState> {
    let pointer = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(hwnd, GWLP_USERDATA)
    } as *mut WindowState;
    unsafe { pointer.as_mut() }
}

fn resize_status(hwnd: HWND, status_bar: HWND) {
    let mut client = RECT::default();
    if unsafe { GetClientRect(hwnd, &mut client) } == 0 || status_bar.is_null() {
        return;
    }
    let width = client.right - client.left;
    let height = client.bottom - client.top;
    unsafe {
        MoveWindow(
            status_bar,
            0,
            (height - STATUS_HEIGHT).max(0),
            width,
            STATUS_HEIGHT,
            1,
        )
    };
}

fn update_status(state: &WindowState) {
    let text = wide(&state.app.status_text());
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(
            state.status_bar,
            SB_SETTEXTW,
            0,
            text.as_ptr() as LPARAM,
        )
    };
}

fn show_placeholder(hwnd: HWND, command: &str) {
    let title = wide(command);
    let message = wide(&format!("{command} is not implemented yet."));
    unsafe {
        MessageBoxW(
            hwnd,
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        )
    };
}

fn show_about(hwnd: HWND) {
    let title = wide(&format!("About {APP_NAME}"));
    let message = wide(&format!(
        "{APP_NAME}\nVersion {APP_VERSION}\n\n{APP_DESCRIPTION}\n\nAuthor: {APP_AUTHOR}\n{APP_COPYRIGHT}\n{APP_WEBSITE}"
    ));
    unsafe {
        MessageBoxW(
            hwnd,
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        )
    };
}

fn paint(hwnd: HWND, state: &mut WindowState) {
    let mut paint: PAINTSTRUCT = unsafe { zeroed() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
    if let Some(back_buffer) = &state.back_buffer {
        let dirty = &paint.rcPaint;
        unsafe {
            BitBlt(
                hdc,
                dirty.left,
                dirty.top,
                dirty.right - dirty.left,
                dirty.bottom - dirty.top,
                back_buffer.dc,
                dirty.left,
                dirty.top,
                SRCCOPY,
            );
        }
    }
    unsafe { EndPaint(hwnd, &paint) };
}

fn rebuild_back_buffer(hwnd: HWND, state: &mut WindowState) -> Result<(), String> {
    let mut client = RECT::default();
    if unsafe { GetClientRect(hwnd, &mut client) } == 0 {
        return Err("Unable to read the window client area.".to_owned());
    }
    let width = client.right - client.left;
    let height = (client.bottom - client.top - STATUS_HEIGHT).max(0);
    if width <= 0 || height <= 0 {
        state.back_buffer = None;
        return Ok(());
    }

    let back_buffer = BackBuffer::new(hwnd, width, height, &state.app.buffer)?;
    state.back_buffer = Some(back_buffer);
    Ok(())
}

fn update_changed_cells(
    hwnd: HWND,
    state: &mut WindowState,
    changed: ((usize, usize), (usize, usize)),
) -> Result<(), String> {
    if state.back_buffer.is_none() {
        return Ok(());
    }

    let cells = [changed.0, changed.1];
    let mut dirty_rects = Vec::with_capacity(cells.len());
    {
        let Some(back_buffer) = state.back_buffer.as_ref() else {
            return Err("The off-screen grid buffer is not initialized.".to_owned());
        };
        let mut last_cell = None;
        for (row, col) in cells {
            if last_cell == Some((row, col)) {
                continue;
            }
            last_cell = Some((row, col));
            let rect = back_buffer.cell_rect(row, col);
            if rect.right > rect.left && rect.bottom > rect.top {
                dirty_rects.push(rect);
            }
        }
    }

    state
        .back_buffer
        .as_mut()
        .ok_or_else(|| "The off-screen grid buffer is not initialized.".to_owned())?
        .draw_full(&state.app.buffer)?;
    for rect in dirty_rects {
        unsafe { InvalidateRect(hwnd, &rect, 0) };
    }
    Ok(())
}

impl BackBuffer {
    fn new(hwnd: HWND, width: i32, height: i32, pixels: &[u32]) -> Result<Self, String> {
        let window_dc = unsafe { GetDC(hwnd) };
        if window_dc.is_null() {
            return Err("Unable to acquire the window drawing context.".to_owned());
        }
        let dc = unsafe { CreateCompatibleDC(window_dc) };
        if dc.is_null() {
            unsafe { ReleaseDC(hwnd, window_dc) };
            return Err("Unable to create the off-screen drawing context.".to_owned());
        }
        let bitmap = unsafe { CreateCompatibleBitmap(window_dc, width, height) };
        if bitmap.is_null() {
            unsafe {
                DeleteDC(dc);
                ReleaseDC(hwnd, window_dc);
            }
            return Err("Unable to create the off-screen bitmap.".to_owned());
        }
        let previous_bitmap = unsafe { SelectObject(dc, bitmap as HGDIOBJ) };
        unsafe { ReleaseDC(hwnd, window_dc) };
        if previous_bitmap.is_null() || previous_bitmap == -1_isize as HGDIOBJ {
            unsafe {
                DeleteObject(bitmap as HGDIOBJ);
                DeleteDC(dc);
            }
            return Err("Unable to select the off-screen bitmap.".to_owned());
        }

        let buffer = Self {
            dc,
            bitmap,
            previous_bitmap,
            width,
            height,
        };
        buffer.draw_full(pixels)?;
        Ok(buffer)
    }

    fn draw_full(&self, pixels: &[u32]) -> Result<(), String> {
        let background_rect = RECT {
            left: 0,
            top: 0,
            right: self.width,
            bottom: self.height,
        };
        unsafe {
            FillRect(self.dc, &background_rect, GetSysColorBrush(COLOR_WINDOW));
        }

        let scale = (self.width as f64 / WIDTH as f64).min(self.height as f64 / HEIGHT as f64);
        let grid_width = (WIDTH as f64 * scale).round() as i32;
        let grid_height = (HEIGHT as f64 * scale).round() as i32;
        let x = (self.width - grid_width) / 2;
        let y = (self.height - grid_height) / 2;
        if unsafe {
            StretchDIBits(
                self.dc,
                x,
                y,
                grid_width,
                grid_height,
                0,
                0,
                WIDTH as i32,
                HEIGHT as i32,
                pixels.as_ptr().cast::<c_void>(),
                &bitmap_info(),
                DIB_RGB_COLORS,
                SRCCOPY,
            )
        } == GDI_ERROR
        {
            return Err("GDI failed to draw the initial grid.".to_owned());
        }
        Ok(())
    }

    fn cell_rect(&self, row: usize, col: usize) -> RECT {
        let (left, top, right, bottom) = AppState::cell_bounds(row, col, self.width, self.height);
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl Drop for BackBuffer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous_bitmap);
            DeleteObject(self.bitmap as HGDIOBJ);
            DeleteDC(self.dc);
        }
    }
}

fn bitmap_info() -> BITMAPINFO {
    let mut bitmap: BITMAPINFO = unsafe { zeroed() };
    bitmap.bmiHeader = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: WIDTH as i32,
        biHeight: -(HEIGHT as i32),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        ..unsafe { zeroed() }
    };
    bitmap
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
