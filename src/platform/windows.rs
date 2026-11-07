use std::ffi::{OsString, c_void};
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLACK_BRUSH, BeginPaint, BitBlt, COLOR_BTNFACE,
    COLOR_BTNSHADOW, COLOR_BTNTEXT, COLOR_WINDOW, CreateCompatibleBitmap, CreateCompatibleDC,
    CreateSolidBrush, DEFAULT_GUI_FONT, DIB_RGB_COLORS, DT_CALCRECT, DT_NOPREFIX, DT_SINGLELINE,
    DT_VCENTER, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, FrameRect, GDI_ERROR, GetDC,
    GetStockObject, GetSysColor, GetSysColorBrush, HBITMAP, HBRUSH, HDC, HGDIOBJ, InvalidateRect,
    PAINTSTRUCT, ReleaseDC, SRCCOPY, SelectObject, SetBkMode, SetTextColor, StretchDIBits,
    TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST,
    OFN_HIDEREADONLY, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows_sys::Win32::UI::Controls::{
    ICC_BAR_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx, SB_SETTEXTW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CheckMenuItem, CreateMenu, CreatePopupMenu,
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetClientRect, GetMenu,
    GetMessageW, HMENU, IDC_ARROW, LoadCursorW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
    MESSAGEBOX_STYLE, MF_BYCOMMAND, MF_CHECKED, MF_POPUP, MF_STRING, MF_UNCHECKED, MSG, MessageBoxW,
    MoveWindow, PostQuitMessage, RegisterClassW, SW_SHOW, SetWindowLongPtrW, ShowWindow,
    TranslateMessage, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_ERASEBKGND, WM_GETFONT, WM_KEYDOWN,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE, WM_PAINT, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SIZE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use crate::app::{AppState, MouseButton, VehicleKey};
use crate::config::{
    APP_AUTHOR, APP_COPYRIGHT, APP_DESCRIPTION, APP_NAME, APP_VERSION, APP_WEBSITE, HEIGHT, WIDTH,
};
use crate::map_file::{DEFAULT_FILE_NAME, FILE_EXTENSION};
use crate::render::LEGEND;

const ID_FILE_NEW: usize = 1001;
const ID_FILE_SAVE: usize = 1002;
const ID_FILE_LOAD: usize = 1003;
const ID_FILE_EXIT: usize = 1004;
const ID_MAP_EDIT: usize = 1101;
const ID_ABOUT: usize = 1201;
const STATUS_HEIGHT: i32 = 24;
/// Height of the legend panel between the grid and the status bar.
const LEGEND_HEIGHT: i32 = 28;
/// Size of the color samples in the legend.
const LEGEND_SWATCH: i32 = 14;
/// Space around the legend and between its entries.
const LEGEND_SPACING: i32 = 8;
/// Bits set in a mouse message's `wparam` while the left or right button is held down.
const MK_LBUTTON: usize = 0x0001;
const MK_RBUTTON: usize = 0x0002;
const WINDOW_CLASS: &str = "PathPlannerGridWindow";
/// Size of the file name buffer used by the Save and Load dialogs, in UTF-16 code units.
const FILE_NAME_CAPACITY: usize = 4096;

struct WindowState {
    app: AppState,
    status_bar: HWND,
    back_buffer: Option<BackBuffer>,
    /// Button pressed on the window in edit mode; moving the mouse while it is held paints cells.
    painting: Option<MouseButton>,
}

struct BackBuffer {
    dc: HDC,
    bitmap: HBITMAP,
    previous_bitmap: HGDIOBJ,
    width: i32,
    height: i32,
}

#[derive(Clone, Copy)]
enum MapFileDialog {
    Save,
    Load,
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
        painting: None,
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
            500 + LEGEND_HEIGHT,
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
            match wparam & 0xffff {
                ID_FILE_NEW => new_map(hwnd),
                ID_FILE_SAVE => save_map(hwnd),
                ID_FILE_LOAD => load_map(hwnd),
                ID_FILE_EXIT => {
                    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
                }
                ID_MAP_EDIT => toggle_editing(hwnd),
                ID_ABOUT => show_about(hwnd),
                _ => return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
            }
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
        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            let button = if message == WM_LBUTTONDOWN {
                MouseButton::Left
            } else {
                MouseButton::Right
            };
            start_painting(hwnd, button, lparam);
            0
        }
        WM_MOUSEMOVE => {
            continue_painting(hwnd, wparam, lparam);
            0
        }
        WM_LBUTTONUP | WM_RBUTTONUP => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                state.painting = None;
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

fn new_map(hwnd: HWND) {
    if let Some(state) = unsafe { state_mut(hwnd) } {
        state.app.new_map();
        refresh_grid(hwnd, state);
    }
}

fn save_map(hwnd: HWND) {
    let Some(path) = choose_map_file(hwnd, MapFileDialog::Save) else {
        return;
    };
    let result = match unsafe { state_mut(hwnd) } {
        Some(state) => state.app.save_map(&path),
        None => return,
    };
    if let Err(error) = result {
        show_message(hwnd, "Save", &error, MB_ICONERROR);
    }
}

fn load_map(hwnd: HWND) {
    let Some(path) = choose_map_file(hwnd, MapFileDialog::Load) else {
        return;
    };
    let Some(state) = (unsafe { state_mut(hwnd) }) else {
        return;
    };
    if let Err(error) = state.app.load_map(&path) {
        show_message(hwnd, "Load", &error, MB_ICONERROR);
    } else {
        refresh_grid(hwnd, state);
    }
}

/// Repaints the whole grid after the map has been replaced.
fn refresh_grid(hwnd: HWND, state: &WindowState) {
    let redrawn = match &state.back_buffer {
        Some(back_buffer) => back_buffer.draw_grid(&state.app.buffer),
        None => Ok(()),
    };
    if let Err(error) = redrawn {
        eprintln!("Unable to redraw the off-screen grid buffer: {error}");
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
    } else {
        unsafe { InvalidateRect(hwnd, null(), 0) };
    }
}

/// Turns edit mode on or off from Map → Edit, which shows a check mark while editing.
fn toggle_editing(hwnd: HWND) {
    let Some(state) = (unsafe { state_mut(hwnd) }) else {
        return;
    };
    let editing = !state.app.is_editing();
    state.app.set_editing(editing);
    let check = if editing { MF_CHECKED } else { MF_UNCHECKED };
    unsafe { CheckMenuItem(GetMenu(hwnd), ID_MAP_EDIT as u32, MF_BYCOMMAND | check) };
    update_status(state);
    refresh_grid(hwnd, state);
}

fn start_painting(hwnd: HWND, button: MouseButton, lparam: LPARAM) {
    let Some(state) = (unsafe { state_mut(hwnd) }) else {
        return;
    };
    if state.app.is_editing() {
        state.painting = Some(button);
        paint_cell(hwnd, state, lparam);
    }
}

fn continue_painting(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) {
    let Some(state) = (unsafe { state_mut(hwnd) }) else {
        return;
    };
    // The window does not capture the mouse, so a button released outside it shows up here.
    let held = match state.painting {
        Some(MouseButton::Left) => (wparam & MK_LBUTTON) != 0,
        Some(MouseButton::Right) => (wparam & MK_RBUTTON) != 0,
        None => return,
    };
    if held {
        paint_cell(hwnd, state, lparam);
    } else {
        state.painting = None;
    }
}

/// Applies the held mouse button to the cell under the cursor.
fn paint_cell(hwnd: HWND, state: &mut WindowState, lparam: LPARAM) {
    let (Some(button), Some(back_buffer)) = (state.painting, &state.back_buffer) else {
        return;
    };
    let (x, y) = cursor_position(lparam);
    let Some((row, col)) = back_buffer.cell_at(x, y) else {
        return;
    };
    if !state.app.edit_cell(row, col, button) {
        return;
    }
    if let Err(error) = update_changed_cells(hwnd, state, ((row, col), (row, col))) {
        eprintln!("Unable to update the off-screen grid buffer: {error}");
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
    }
}

/// Client coordinates of the cursor, packed into a mouse message's `lparam` as signed values.
fn cursor_position(lparam: LPARAM) -> (i32, i32) {
    let x = (lparam & 0xffff) as u16 as i16;
    let y = ((lparam >> 16) & 0xffff) as u16 as i16;
    (i32::from(x), i32::from(y))
}

/// Shows the native Save or Open dialog for map files. Returns `None` if the user cancels.
fn choose_map_file(hwnd: HWND, dialog: MapFileDialog) -> Option<PathBuf> {
    let (title, initial_name, flags) = match dialog {
        MapFileDialog::Save => ("Save Map", DEFAULT_FILE_NAME, OFN_OVERWRITEPROMPT),
        MapFileDialog::Load => ("Load Map", "", OFN_FILEMUSTEXIST),
    };
    // Pairs of display name and pattern; the list ends with an empty string.
    let filter = wide(&format!(
        "Map files (*.{FILE_EXTENSION})\0*.{FILE_EXTENSION}\0All files (*.*)\0*.*\0"
    ));
    let title_text = wide(title);
    let default_extension = wide(FILE_EXTENSION);
    let mut file_name: Vec<u16> = initial_name.encode_utf16().collect();
    file_name.resize(FILE_NAME_CAPACITY, 0);

    let mut options: OPENFILENAMEW = unsafe { zeroed() };
    options.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    options.hwndOwner = hwnd;
    options.lpstrFilter = filter.as_ptr();
    options.nFilterIndex = 1;
    options.lpstrFile = file_name.as_mut_ptr();
    options.nMaxFile = FILE_NAME_CAPACITY as u32;
    options.lpstrTitle = title_text.as_ptr();
    options.lpstrDefExt = default_extension.as_ptr();
    options.Flags = flags | OFN_EXPLORER | OFN_HIDEREADONLY | OFN_NOCHANGEDIR | OFN_PATHMUSTEXIST;

    let chosen = unsafe {
        match dialog {
            MapFileDialog::Save => GetSaveFileNameW(&mut options),
            MapFileDialog::Load => GetOpenFileNameW(&mut options),
        }
    };
    if chosen == 0 {
        // The extended error is zero when the user simply cancelled.
        let error = unsafe { CommDlgExtendedError() };
        if error != 0 {
            let message = format!("The file dialog failed (error 0x{error:04X}).");
            show_message(hwnd, title, &message, MB_ICONERROR);
        }
        return None;
    }

    let length = file_name
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(FILE_NAME_CAPACITY);
    Some(PathBuf::from(OsString::from_wide(&file_name[..length])))
}

fn show_message(hwnd: HWND, title: &str, text: &str, icon: MESSAGEBOX_STYLE) {
    let title = wide(title);
    let text = wide(text);
    unsafe { MessageBoxW(hwnd, text.as_ptr(), title.as_ptr(), MB_OK | icon) };
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

    let font = legend_font(state.status_bar);
    let back_buffer = BackBuffer::new(hwnd, width, height, &state.app.buffer, font)?;
    state.back_buffer = Some(back_buffer);
    Ok(())
}

/// The status bar's font, so the legend matches the status bar below it.
fn legend_font(status_bar: HWND) -> HGDIOBJ {
    let font = unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(status_bar, WM_GETFONT, 0, 0)
    } as HGDIOBJ;
    if font.is_null() {
        unsafe { GetStockObject(DEFAULT_GUI_FONT) }
    } else {
        font
    }
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
        .draw_grid(&state.app.buffer)?;
    for rect in dirty_rects {
        unsafe { InvalidateRect(hwnd, &rect, 0) };
    }
    Ok(())
}

impl BackBuffer {
    fn new(
        hwnd: HWND,
        width: i32,
        height: i32,
        pixels: &[u32],
        legend_font: HGDIOBJ,
    ) -> Result<Self, String> {
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
        buffer.draw_legend(legend_font);
        buffer.draw_grid(pixels)?;
        Ok(buffer)
    }

    /// Height of the area above the legend, where the grid is drawn.
    fn grid_area_height(&self) -> i32 {
        (self.height - LEGEND_HEIGHT).max(0)
    }

    /// Where the grid is drawn: as large as fits above the legend, and centered.
    fn grid_rect(&self) -> RECT {
        let area_height = self.grid_area_height();
        let scale = (self.width as f64 / WIDTH as f64).min(area_height as f64 / HEIGHT as f64);
        let width = (WIDTH as f64 * scale).round() as i32;
        let height = (HEIGHT as f64 * scale).round() as i32;
        let left = (self.width - width) / 2;
        let top = (area_height - height) / 2;
        RECT {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    fn draw_grid(&self, pixels: &[u32]) -> Result<(), String> {
        let background_rect = RECT {
            left: 0,
            top: 0,
            right: self.width,
            bottom: self.grid_area_height(),
        };
        unsafe {
            FillRect(self.dc, &background_rect, GetSysColorBrush(COLOR_WINDOW));
        }

        let grid = self.grid_rect();
        if unsafe {
            StretchDIBits(
                self.dc,
                grid.left,
                grid.top,
                grid.right - grid.left,
                grid.bottom - grid.top,
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
        let (left, top, right, bottom) =
            AppState::cell_bounds(row, col, self.width, self.grid_area_height());
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Cell drawn under the client point `(x, y)`, if any.
    fn cell_at(&self, x: i32, y: i32) -> Option<(usize, usize)> {
        let grid = self.grid_rect();
        AppState::cell_at(
            f64::from(x),
            f64::from(y),
            f64::from(grid.left),
            f64::from(grid.top),
            f64::from(grid.right - grid.left),
            f64::from(grid.bottom - grid.top),
        )
    }

    /// Draws the legend panel below the grid: a color sample and a label for each map color.
    fn draw_legend(&self, font: HGDIOBJ) {
        let top = self.grid_area_height();
        let panel = RECT {
            left: 0,
            top,
            right: self.width,
            bottom: self.height,
        };
        let separator = RECT {
            bottom: top + 1,
            ..panel
        };
        unsafe {
            FillRect(self.dc, &panel, GetSysColorBrush(COLOR_BTNFACE));
            FillRect(self.dc, &separator, GetSysColorBrush(COLOR_BTNSHADOW));
        }

        let previous_font = unsafe { SelectObject(self.dc, font) };
        unsafe {
            SetBkMode(self.dc, TRANSPARENT as i32);
            SetTextColor(self.dc, GetSysColor(COLOR_BTNTEXT));
        }
        let swatch_top = top + (self.height - top - LEGEND_SWATCH) / 2;
        let mut left = LEGEND_SPACING;
        for (label, color) in LEGEND {
            let swatch = RECT {
                left,
                top: swatch_top,
                right: left + LEGEND_SWATCH,
                bottom: swatch_top + LEGEND_SWATCH,
            };
            unsafe {
                let brush = CreateSolidBrush(colorref(color));
                FillRect(self.dc, &swatch, brush);
                DeleteObject(brush as HGDIOBJ);
                FrameRect(self.dc, &swatch, GetStockObject(BLACK_BRUSH) as HBRUSH);
            }

            let text = wide(label);
            let text_left = swatch.right + LEGEND_SPACING / 2;
            let mut text_rect = RECT {
                left: text_left,
                top,
                right: text_left,
                bottom: self.height,
            };
            // Measure the label first, so the next entry starts right after it.
            unsafe {
                DrawTextW(
                    self.dc,
                    text.as_ptr(),
                    -1,
                    &mut text_rect,
                    DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX,
                )
            };
            text_rect.top = top;
            text_rect.bottom = self.height;
            unsafe {
                DrawTextW(
                    self.dc,
                    text.as_ptr(),
                    -1,
                    &mut text_rect,
                    DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
                )
            };
            left = text_rect.right + 2 * LEGEND_SPACING;
        }
        unsafe { SelectObject(self.dc, previous_font) };
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

/// Converts a `0x00RRGGBB` color, as stored in the grid's pixel buffer, to a GDI `COLORREF`
/// (`0x00BBGGRR`).
fn colorref(color: u32) -> COLORREF {
    ((color & 0xff) << 16) | (color & 0xff00) | ((color >> 16) & 0xff)
}
