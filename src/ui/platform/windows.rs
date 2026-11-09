use std::ffi::{OsString, c_void};
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLACK_BRUSH, BeginPaint, BitBlt, COLOR_BTNFACE,
    COLOR_BTNSHADOW, COLOR_BTNTEXT, COLOR_WINDOW, COLORONCOLOR, CreateCompatibleBitmap,
    CreateCompatibleDC, CreateSolidBrush, DEFAULT_GUI_FONT, DIB_RGB_COLORS, DT_CALCRECT,
    DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect,
    FrameRect, GDI_ERROR, GetDC, GetStockObject, GetSysColor, GetSysColorBrush, HALFTONE, HBITMAP,
    HBRUSH, HDC, HGDIOBJ, InvalidateRect, PAINTSTRUCT, ReleaseDC, SRCCOPY, SelectObject,
    SetBkMode, SetBrushOrgEx, SetStretchBltMode, SetTextColor, StretchDIBits, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetOpenFileNameW, GetSaveFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST,
    OFN_HIDEREADONLY, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows_sys::Win32::UI::Controls::{
    ICC_BAR_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx, SB_SETTEXTW,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, GetFocus, SetFocus};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, AppendMenuW, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    CheckMenuItem, CreateMenu, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DeleteMenu,
    DestroyWindow, DispatchMessageW, GWLP_USERDATA, GetClientRect, GetMenu, GetMenuItemCount,
    GetMessageW, GetWindowLongPtrW, GetWindowRect, GetWindowTextW, HMENU, IDC_ARROW,
    IsDialogMessageW, LoadCursorW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MESSAGEBOX_STYLE,
    MF_BYCOMMAND, MF_BYPOSITION, MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG,
    MessageBoxW, MoveWindow, PostQuitMessage, RegisterClassW, SW_SHOW, SendMessageW,
    SetWindowLongPtrW, SetWindowTextW, ShowWindow, TranslateMessage, WM_ACTIVATE, WM_CLOSE,
    WM_COMMAND, WM_CREATE, WM_DESTROY, WM_ERASEBKGND, WM_GETFONT, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCREATE, WM_PAINT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETFONT,
    WM_SIZE, WNDCLASSW, WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN, WS_EX_CLIENTEDGE,
    WS_EX_DLGMODALFRAME, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE,
    WS_VSCROLL,
};

use crate::simulator::{Command, DEFAULT_FILE_NAME, FILE_EXTENSION, MAX_GRID_SIZE, MIN_GRID_SIZE};
use crate::ui::app::{AppState, HEADINGS, MapSettings, MouseButton};
use crate::ui::config::{
    APP_AUTHOR, APP_COPYRIGHT, APP_DESCRIPTION, APP_NAME, APP_VERSION, APP_WEBSITE,
};
use crate::ui::recent::{MAX_RECENT_FILES, RecentFiles};
use crate::ui::render::LEGEND;

const ID_FILE_NEW: usize = 1001;
const ID_FILE_SAVE: usize = 1002;
const ID_FILE_LOAD: usize = 1003;
const ID_FILE_EXIT: usize = 1004;
const ID_FILE_CLEAR_RECENT: usize = 1005;
/// Commands of the recent files listed in the File menu, newest first.
const ID_FILE_RECENT_FIRST: usize = 1011;
const ID_FILE_RECENT_LAST: usize = ID_FILE_RECENT_FIRST + MAX_RECENT_FILES - 1;
const ID_MAP_EDIT: usize = 1101;
const ID_MAP_GRID_SIZE: usize = 1102;
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

const SETTINGS_CLASS: &str = "PathPlannerGridSizeDialog";
/// Client area of the Grid Size dialog.
const SETTINGS_WIDTH: i32 = 300;
const SETTINGS_HEIGHT: i32 = 262;
/// The OK and Cancel button commands; `IsDialogMessageW` also sends them for Enter and Escape.
const ID_OK: usize = 1;
const ID_CANCEL: usize = 2;
const ID_ROWS: usize = 101;
const ID_COLUMNS: usize = 102;
const ID_START_ROW: usize = 103;
const ID_START_COLUMN: usize = 104;
const ID_HEADING: usize = 105;
// Edit, combo box and button styles and messages, as defined by the Windows SDK.
const ES_AUTOHSCROLL: u32 = 0x0080;
const ES_NUMBER: u32 = 0x2000;
const CBS_DROPDOWNLIST: u32 = 0x0003;
const BS_PUSHBUTTON: u32 = 0x0000;
const BS_DEFPUSHBUTTON: u32 = 0x0001;
const EM_SETSEL: u32 = 0x00B1;
const EM_SETLIMITTEXT: u32 = 0x00C5;
const CB_ADDSTRING: u32 = 0x0143;
const CB_GETCURSEL: u32 = 0x0147;
const CB_SETCURSEL: u32 = 0x014E;
/// Low word of `WM_ACTIVATE`'s `wparam` when the window is deactivated.
const WA_INACTIVE: usize = 0;

struct WindowState {
    app: AppState,
    recent: RecentFiles,
    /// The File menu, rebuilt whenever the recent files change.
    file_menu: HMENU,
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

/// State of the Grid Size dialog while it is open.
struct SettingsDialog {
    owner: HWND,
    initial: MapSettings,
    /// Rows, columns, start row and start column fields, in that order.
    fields: [HWND; 4],
    heading: HWND,
    /// Control that had the focus when another window was activated, such as an error message.
    focus: HWND,
    result: Option<MapSettings>,
    done: bool,
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

    let settings_class = wide(SETTINGS_CLASS);
    let mut settings_window_class: WNDCLASSW = unsafe { zeroed() };
    settings_window_class.lpfnWndProc = Some(settings_proc);
    settings_window_class.hInstance = instance;
    settings_window_class.hCursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    settings_window_class.hbrBackground = unsafe { GetSysColorBrush(COLOR_BTNFACE) };
    settings_window_class.lpszClassName = settings_class.as_ptr();
    if unsafe { RegisterClassW(&settings_window_class) } == 0 {
        return Err("Unable to register the Grid Size dialog class.".to_owned());
    }

    let (menu, file_menu) = create_menu()?;
    let recent = RecentFiles::load();
    fill_file_menu(file_menu, &recent)?;
    let app = AppState::new();
    let title = wide(&app.title());
    let mut state = Box::new(WindowState {
        app,
        recent,
        file_menu,
        status_bar: null_mut(),
        back_buffer: None,
        painting: None,
    });
    let state_pointer = (&mut *state) as *mut WindowState;
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
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

/// Creates the menu bar. Returns it with the File menu, which `fill_file_menu` fills.
fn create_menu() -> Result<(HMENU, HMENU), String> {
    let menu_bar = unsafe { CreateMenu() };
    let file_menu = unsafe { CreatePopupMenu() };
    let map_menu = unsafe { CreatePopupMenu() };
    if menu_bar.is_null() || file_menu.is_null() || map_menu.is_null() {
        return Err("Unable to create the application menus.".to_owned());
    }

    for (label, command) in [("Edit", ID_MAP_EDIT), ("Grid Size...", ID_MAP_GRID_SIZE)] {
        let label = wide(label);
        if unsafe { AppendMenuW(map_menu, MF_STRING, command, label.as_ptr()) } == 0 {
            return Err("Unable to add a Map menu command.".to_owned());
        }
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
    Ok((menu_bar, file_menu))
}

/// Fills the File menu with the map commands, the recent files numbered the way Word and Excel
/// list them, and Exit.
fn fill_file_menu(menu: HMENU, recent: &RecentFiles) -> Result<(), String> {
    while unsafe { GetMenuItemCount(menu) } > 0 {
        if unsafe { DeleteMenu(menu, 0, MF_BYPOSITION) } == 0 {
            return Err("Unable to update the File menu.".to_owned());
        }
    }

    let append = |command: usize, label: &str| {
        let label = wide(label);
        (unsafe { AppendMenuW(menu, MF_STRING, command, label.as_ptr()) }) != 0
    };
    let separator = || unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, null()) } != 0;

    let mut appended =
        append(ID_FILE_NEW, "New") && append(ID_FILE_SAVE, "Save") && append(ID_FILE_LOAD, "Load");
    if !recent.paths().is_empty() {
        appended &= separator();
        for index in 0..recent.paths().len() {
            if let Some(label) = recent.label(index) {
                // `&` marks the access key in a menu, so a literal ampersand is doubled.
                let text = format!("&{} {}", index + 1, label.replace('&', "&&"));
                appended &= append(ID_FILE_RECENT_FIRST + index, &text);
            }
        }
        appended &= separator() && append(ID_FILE_CLEAR_RECENT, "Clear Recent Files");
    }
    appended &= separator() && append(ID_FILE_EXIT, "Exit");
    if appended {
        Ok(())
    } else {
        Err("Unable to add a File menu command.".to_owned())
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };
            // The default handling sets the window title.
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
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
                    WS_CHILD | WS_VISIBLE | windows_sys::Win32::UI::Controls::SBARS_SIZEGRIP,
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
                    unsafe { DestroyWindow(hwnd) };
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
                ID_FILE_CLEAR_RECENT => clear_recent_files(hwnd),
                command @ ID_FILE_RECENT_FIRST..=ID_FILE_RECENT_LAST => {
                    open_recent_file(hwnd, command - ID_FILE_RECENT_FIRST);
                }
                ID_FILE_EXIT => {
                    unsafe { DestroyWindow(hwnd) };
                }
                ID_MAP_EDIT => toggle_editing(hwnd),
                ID_MAP_GRID_SIZE => change_grid_size(hwnd),
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
                unsafe { DestroyWindow(hwnd) };
                return 0;
            }
            let command = match wparam {
                0x26 => Some(Command::Forward),
                0x28 => Some(Command::Reverse),
                0x25 => Some(Command::TurnLeft),
                0x27 => Some(Command::TurnRight),
                _ => None,
            };
            if let (Some(command), Some(state)) = (command, unsafe { state_mut(hwnd) }) {
                // The keys do nothing in edit mode.
                if let Some(changed_cells) = state.app.drive(command) {
                    update_status(state);
                    if let Err(error) = update_changed_cells(hwnd, state, changed_cells) {
                        eprintln!("Unable to update the off-screen grid buffer: {error}");
                        unsafe { DestroyWindow(hwnd) };
                    }
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
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
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
    unsafe { SendMessageW(state.status_bar, SB_SETTEXTW, 0, text.as_ptr() as LPARAM) };
}

fn new_map(hwnd: HWND) {
    if let Some(state) = unsafe { state_mut(hwnd) } {
        state.app.new_map();
        show_new_map(hwnd, state);
    }
}

fn save_map(hwnd: HWND) {
    let Some(path) = choose_map_file(hwnd, MapFileDialog::Save) else {
        return;
    };
    let Some(state) = (unsafe { state_mut(hwnd) }) else {
        return;
    };
    match state.app.save_map(&path) {
        Ok(()) => remember_file(state, &path),
        Err(error) => show_message(hwnd, "Save", &error, MB_ICONERROR),
    }
}

fn load_map(hwnd: HWND) {
    if let Some(path) = choose_map_file(hwnd, MapFileDialog::Load) {
        open_map_file(hwnd, &path);
    }
}

/// Opens the recent file listed at `index` in the File menu.
fn open_recent_file(hwnd: HWND, index: usize) {
    let path = match unsafe { state_mut(hwnd) } {
        Some(state) => state.recent.get(index).map(Path::to_path_buf),
        None => None,
    };
    if let Some(path) = path {
        open_map_file(hwnd, &path);
    }
}

/// Loads a map file and moves it to the top of the recent files. A recent file that no longer
/// exists is removed from the list.
fn open_map_file(hwnd: HWND, path: &Path) {
    let Some(state) = (unsafe { state_mut(hwnd) }) else {
        return;
    };
    match state.app.load_map(path) {
        Ok(()) => {
            remember_file(state, path);
            show_new_map(hwnd, state);
        }
        Err(mut error) => {
            if !path.exists() && state.recent.remove(path) {
                update_recent_menu(state);
                error.push_str("\n\nThe file was removed from the recent files list.");
            }
            show_message(hwnd, "Load", &error, MB_ICONERROR);
        }
    }
}

fn clear_recent_files(hwnd: HWND) {
    if let Some(state) = unsafe { state_mut(hwnd) } {
        state.recent.clear();
        update_recent_menu(state);
    }
}

fn remember_file(state: &mut WindowState, path: &Path) {
    state.recent.add(path);
    update_recent_menu(state);
}

fn update_recent_menu(state: &WindowState) {
    if let Err(error) = fill_file_menu(state.file_menu, &state.recent) {
        eprintln!("{error}");
    }
}

/// Shows a replaced or resized map: the window title, the status bar and the whole grid.
fn show_new_map(hwnd: HWND, state: &WindowState) {
    let title = wide(&state.app.title());
    unsafe { SetWindowTextW(hwnd, title.as_ptr()) };
    update_status(state);
    refresh_grid(hwnd, state);
}

/// Repaints the whole grid after the map has been replaced.
fn refresh_grid(hwnd: HWND, state: &WindowState) {
    let redrawn = match &state.back_buffer {
        Some(back_buffer) => back_buffer.draw_grid(&state.app),
        None => Ok(()),
    };
    if let Err(error) = redrawn {
        eprintln!("Unable to redraw the off-screen grid buffer: {error}");
        unsafe { DestroyWindow(hwnd) };
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

/// Map → Grid Size: asks for the size of the map and where the car starts, then applies them.
fn change_grid_size(hwnd: HWND) {
    let Some(initial) = (unsafe { state_mut(hwnd) }).map(|state| state.app.settings()) else {
        return;
    };
    let Some(settings) = show_settings_dialog(hwnd, initial) else {
        return;
    };
    if let Some(state) = unsafe { state_mut(hwnd) } {
        state.app.apply_settings(settings);
        show_new_map(hwnd, state);
    }
}

/// Shows the modal Grid Size dialog. Returns the accepted settings, or `None` if it is canceled.
fn show_settings_dialog(owner: HWND, initial: MapSettings) -> Option<MapSettings> {
    let dialog = Box::into_raw(Box::new(SettingsDialog {
        owner,
        initial,
        fields: [null_mut(); 4],
        heading: null_mut(),
        focus: null_mut(),
        result: None,
        done: false,
    }));
    let class_name = wide(SETTINGS_CLASS);
    let title = wide("Grid Size");
    let style = WS_POPUP | WS_CAPTION | WS_SYSMENU;
    let mut frame = RECT {
        left: 0,
        top: 0,
        right: SETTINGS_WIDTH,
        bottom: SETTINGS_HEIGHT,
    };
    unsafe { AdjustWindowRectEx(&mut frame, style, 0, WS_EX_DLGMODALFRAME) };
    let (width, height) = (frame.right - frame.left, frame.bottom - frame.top);
    let mut owner_rect = RECT::default();
    unsafe { GetWindowRect(owner, &mut owner_rect) };
    let x = owner_rect.left + (owner_rect.right - owner_rect.left - width) / 2;
    let y = owner_rect.top + (owner_rect.bottom - owner_rect.top - height) / 2;

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_DLGMODALFRAME,
            class_name.as_ptr(),
            title.as_ptr(),
            style,
            x,
            y,
            width,
            height,
            owner,
            null_mut(),
            GetModuleHandleW(null()),
            dialog.cast(),
        )
    };
    if hwnd.is_null() {
        unsafe { drop(Box::from_raw(dialog)) };
        show_message(
            owner,
            "Grid Size",
            "Unable to open the Grid Size dialog.",
            MB_ICONERROR,
        );
        return None;
    }

    unsafe {
        EnableWindow(owner, 0);
        ShowWindow(hwnd, SW_SHOW);
        let first_field = (*dialog).fields[0];
        SetFocus(first_field);
        SendMessageW(first_field, EM_SETSEL, 0, -1);
    }

    // A message loop of its own keeps the dialog modal while it is open.
    let mut message: MSG = unsafe { zeroed() };
    while !unsafe { (*dialog).done } {
        let received = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if received == 0 {
            // Leave the quit request for the main message loop.
            unsafe { PostQuitMessage(message.wParam as i32) };
            break;
        }
        if received == -1 {
            break;
        }
        if unsafe { IsDialogMessageW(hwnd, &message) } == 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
    if !unsafe { (*dialog).done } {
        unsafe {
            EnableWindow(owner, 1);
            DestroyWindow(hwnd);
        }
    }
    let dialog = unsafe { Box::from_raw(dialog) };
    dialog.result
}

unsafe extern "system" fn settings_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = unsafe { &*(lparam as *const CREATESTRUCTW) };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_CREATE => match unsafe { dialog_mut(hwnd) } {
            Some(dialog) => {
                create_settings_controls(hwnd, dialog);
                0
            }
            None => -1,
        },
        WM_COMMAND => {
            match wparam & 0xffff {
                ID_OK => accept_settings(hwnd),
                ID_CANCEL => close_settings(hwnd),
                _ => return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
            }
            0
        }
        WM_ACTIVATE => {
            // Like a dialog box, give the focus back to the control that had it.
            if let Some(dialog) = unsafe { dialog_mut(hwnd) } {
                if wparam & 0xffff == WA_INACTIVE {
                    let focused = unsafe { GetFocus() };
                    if !focused.is_null() {
                        dialog.focus = focused;
                    }
                } else if !dialog.focus.is_null() {
                    unsafe { SetFocus(dialog.focus) };
                    return 0;
                }
            }
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
        WM_CLOSE => {
            close_settings(hwnd);
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

unsafe fn dialog_mut(hwnd: HWND) -> Option<&'static mut SettingsDialog> {
    let pointer = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut SettingsDialog;
    unsafe { pointer.as_mut() }
}

/// Creates the Grid Size dialog's labels, fields and buttons, filled with the current settings.
fn create_settings_controls(hwnd: HWND, dialog: &mut SettingsDialog) {
    let font = unsafe { GetStockObject(DEFAULT_GUI_FONT) };
    let control = |class: &str,
                   text: &str,
                   style: u32,
                   ex_style: u32,
                   bounds: [i32; 4],
                   id: usize| {
        let class = wide(class);
        let text = wide(text);
        let [x, y, width, height] = bounds;
        let control = unsafe {
            CreateWindowExW(
                ex_style,
                class.as_ptr(),
                text.as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                x,
                y,
                width,
                height,
                hwnd,
                id as HMENU,
                GetModuleHandleW(null()),
                null(),
            )
        };
        if !control.is_null() {
            unsafe { SendMessageW(control, WM_SETFONT, font as WPARAM, 0) };
        }
        control
    };

    let initial = dialog.initial;
    control("STATIC", "Grid size", 0, 0, [12, 12, 276, 18], 0);
    control("STATIC", "Car start", 0, 0, [12, 100, 276, 18], 0);
    let size_hint = format!("({MIN_GRID_SIZE} to {MAX_GRID_SIZE})");
    let fields = [
        ("Rows:", initial.rows, size_hint.as_str(), 34, ID_ROWS),
        ("Columns:", initial.cols, size_hint.as_str(), 62, ID_COLUMNS),
        ("Row:", initial.start.row + 1, "(1 to rows)", 122, ID_START_ROW),
        ("Column:", initial.start.col + 1, "(1 to columns)", 150, ID_START_COLUMN),
    ];
    for (index, (label, value, hint, top, id)) in fields.into_iter().enumerate() {
        control("STATIC", label, 0, 0, [24, top + 3, 96, 18], 0);
        let style = WS_TABSTOP | ES_NUMBER | ES_AUTOHSCROLL;
        let field = control(
            "EDIT",
            &value.to_string(),
            style,
            WS_EX_CLIENTEDGE,
            [124, top, 56, 22],
            id,
        );
        unsafe { SendMessageW(field, EM_SETLIMITTEXT, 3, 0) };
        control("STATIC", hint, 0, 0, [188, top + 3, 100, 18], 0);
        dialog.fields[index] = field;
    }

    control("STATIC", "Heading:", 0, 0, [24, 181, 96, 18], 0);
    let style = WS_TABSTOP | WS_VSCROLL | CBS_DROPDOWNLIST;
    let heading = control("COMBOBOX", "", style, 0, [124, 178, 100, 160], ID_HEADING);
    for (name, _) in HEADINGS {
        let name = wide(name);
        unsafe { SendMessageW(heading, CB_ADDSTRING, 0, name.as_ptr() as LPARAM) };
    }
    unsafe { SendMessageW(heading, CB_SETCURSEL, initial.heading_index(), 0) };
    dialog.heading = heading;

    let ok_style = WS_TABSTOP | BS_DEFPUSHBUTTON;
    control("BUTTON", "OK", ok_style, 0, [120, 222, 80, 26], ID_OK);
    let cancel_style = WS_TABSTOP | BS_PUSHBUTTON;
    control("BUTTON", "Cancel", cancel_style, 0, [208, 222, 80, 26], ID_CANCEL);
}

/// OK button: closes the dialog with the entered settings, or explains what to correct.
fn accept_settings(hwnd: HWND) {
    let Some(dialog) = (unsafe { dialog_mut(hwnd) }) else {
        return;
    };
    let [rows, cols, start_row, start_col] = dialog.fields.map(window_text);
    let heading = unsafe { SendMessageW(dialog.heading, CB_GETCURSEL, 0, 0) };
    let heading = usize::try_from(heading).unwrap_or(usize::MAX);
    match MapSettings::parse(&rows, &cols, &start_row, &start_col, heading) {
        Ok(settings) => {
            dialog.result = Some(settings);
            close_settings(hwnd);
        }
        Err(error) => show_message(hwnd, "Grid Size", &error, MB_ICONERROR),
    }
}

/// Closes the Grid Size dialog. The owner is enabled first so that it gets the focus back.
fn close_settings(hwnd: HWND) {
    let Some(dialog) = (unsafe { dialog_mut(hwnd) }) else {
        return;
    };
    dialog.done = true;
    unsafe {
        EnableWindow(dialog.owner, 1);
        DestroyWindow(hwnd);
    }
}

fn window_text(hwnd: HWND) -> String {
    let mut text = [0u16; 64];
    let length = unsafe { GetWindowTextW(hwnd, text.as_mut_ptr(), text.len() as i32) };
    String::from_utf16_lossy(&text[..length.max(0) as usize])
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
    let Some((row, col)) = back_buffer.cell_at(&state.app, x, y) else {
        return;
    };
    if !state.app.edit_cell(row, col, button) {
        return;
    }
    if let Err(error) = update_changed_cells(hwnd, state, ((row, col), (row, col))) {
        eprintln!("Unable to update the off-screen grid buffer: {error}");
        unsafe { DestroyWindow(hwnd) };
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
    let back_buffer = BackBuffer::new(hwnd, width, height, &state.app, font)?;
    state.back_buffer = Some(back_buffer);
    Ok(())
}

/// The status bar's font, so the legend matches the status bar below it.
fn legend_font(status_bar: HWND) -> HGDIOBJ {
    let font = unsafe { SendMessageW(status_bar, WM_GETFONT, 0, 0) } as HGDIOBJ;
    if font.is_null() {
        unsafe { GetStockObject(DEFAULT_GUI_FONT) }
    } else {
        font
    }
}

fn update_changed_cells(
    hwnd: HWND,
    state: &WindowState,
    changed: ((usize, usize), (usize, usize)),
) -> Result<(), String> {
    let Some(back_buffer) = &state.back_buffer else {
        return Ok(());
    };

    let mut dirty_rects = Vec::with_capacity(2);
    let mut last_cell = None;
    for (row, col) in [changed.0, changed.1] {
        if last_cell == Some((row, col)) {
            continue;
        }
        last_cell = Some((row, col));
        let rect = back_buffer.cell_rect(&state.app, row, col);
        if rect.right > rect.left && rect.bottom > rect.top {
            dirty_rects.push(rect);
        }
    }

    back_buffer.draw_grid(&state.app)?;
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
        app: &AppState,
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
        buffer.draw_grid(app)?;
        Ok(buffer)
    }

    /// Height of the area above the legend, where the grid is drawn.
    fn grid_area_height(&self) -> i32 {
        (self.height - LEGEND_HEIGHT).max(0)
    }

    /// Where the grid is drawn: as large as fits above the legend, and centered.
    fn grid_rect(&self, app: &AppState) -> RECT {
        let area_height = self.grid_area_height();
        let (map_width, map_height) = (app.pixel_width() as f64, app.pixel_height() as f64);
        let scale = (self.width as f64 / map_width).min(area_height as f64 / map_height);
        let width = (map_width * scale).round() as i32;
        let height = (map_height * scale).round() as i32;
        let left = (self.width - width) / 2;
        let top = (area_height - height) / 2;
        RECT {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    fn draw_grid(&self, app: &AppState) -> Result<(), String> {
        let background_rect = RECT {
            left: 0,
            top: 0,
            right: self.width,
            bottom: self.grid_area_height(),
        };
        unsafe {
            FillRect(self.dc, &background_rect, GetSysColorBrush(COLOR_WINDOW));
        }

        let grid = self.grid_rect(app);
        let (map_width, map_height) = (app.pixel_width() as i32, app.pixel_height() as i32);
        // When a large map is shrunk, halftone blends the dropped pixels instead of losing the
        // grid lines.
        let mode = if grid.right - grid.left < map_width {
            HALFTONE
        } else {
            COLORONCOLOR
        };
        unsafe {
            SetStretchBltMode(self.dc, mode);
            SetBrushOrgEx(self.dc, 0, 0, null_mut());
        }
        if unsafe {
            StretchDIBits(
                self.dc,
                grid.left,
                grid.top,
                grid.right - grid.left,
                grid.bottom - grid.top,
                0,
                0,
                map_width,
                map_height,
                app.buffer.as_ptr().cast::<c_void>(),
                &bitmap_info(map_width, map_height),
                DIB_RGB_COLORS,
                SRCCOPY,
            )
        } == GDI_ERROR
        {
            return Err("GDI failed to draw the grid.".to_owned());
        }
        Ok(())
    }

    fn cell_rect(&self, app: &AppState, row: usize, col: usize) -> RECT {
        let (left, top, right, bottom) =
            app.cell_bounds(row, col, self.width, self.grid_area_height());
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Cell drawn under the client point `(x, y)`, if any.
    fn cell_at(&self, app: &AppState, x: i32, y: i32) -> Option<(usize, usize)> {
        let grid = self.grid_rect(app);
        app.cell_at(
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

/// Describes a top-down 32-bit pixel buffer of `width` by `height` pixels.
fn bitmap_info(width: i32, height: i32) -> BITMAPINFO {
    let mut bitmap: BITMAPINFO = unsafe { zeroed() };
    bitmap.bmiHeader = BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
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
