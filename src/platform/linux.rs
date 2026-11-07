use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::cairo::{Format, ImageSurface};
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    AboutDialog, Align, Application, ApplicationWindow, Box as GtkBox, DrawingArea,
    EventControllerKey, FileChooserAction, FileChooserNative, FileFilter, GestureDrag, Label,
    MessageDialog, MessageType, Orientation, PopoverMenuBar, ResponseType, Separator, Statusbar,
};

use crate::app::{AppState, MouseButton, VehicleKey};
use crate::config::{
    APP_AUTHOR, APP_COPYRIGHT, APP_DESCRIPTION, APP_NAME, APP_VERSION, APP_WEBSITE, HEIGHT, WIDTH,
};
use crate::map_file::{DEFAULT_FILE_NAME, FILE_EXTENSION};
use crate::render::LEGEND;

/// Height of the legend panel between the map and the status bar.
const LEGEND_HEIGHT: i32 = 32;
/// Size of the color samples in the legend.
const LEGEND_SWATCH: i32 = 14;
/// Space around the legend and between its entries.
const LEGEND_SPACING: i32 = 8;

pub(super) fn run() -> Result<(), String> {
    let application = Application::builder()
        .application_id("com.poivronjaune.pathplanner")
        .build();
    application.connect_activate(build_window);
    application.run();
    Ok(())
}

fn build_window(application: &Application) {
    let state = Rc::new(RefCell::new(AppState::new()));
    let window = ApplicationWindow::builder()
        .application(application)
        .title("20 x 20 Grid")
        .default_width(WIDTH as i32)
        .default_height(HEIGHT as i32 + 58 + LEGEND_HEIGHT)
        .build();

    let layout = GtkBox::new(Orientation::Vertical, 0);
    let menu_bar = create_menu_bar();
    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(WIDTH as i32);
    drawing_area.set_content_height(HEIGHT as i32);
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.set_focusable(true);

    let status_bar = Statusbar::new();
    let status_context = status_bar.context_id("vehicle-state");

    layout.append(&menu_bar);
    layout.append(&drawing_area);
    layout.append(&Separator::new(Orientation::Horizontal));
    layout.append(&create_legend());
    layout.append(&status_bar);
    window.set_child(Some(&layout));

    update_status(&state, &status_bar, status_context);
    let draw_state = Rc::clone(&state);
    drawing_area.set_draw_func(move |_, context, width, height| {
        draw_grid(&draw_state, context, width, height);
    });

    install_actions(
        application,
        &window,
        &state,
        &drawing_area,
        &status_bar,
        status_context,
    );
    install_vehicle_keys(&drawing_area, &window, &state, &status_bar, status_context);
    install_map_editing(&drawing_area, &state);
    window.present();
    drawing_area.grab_focus();
}

fn create_menu_bar() -> PopoverMenuBar {
    // Commands that act on the window's own map (New, Save, Load and Edit) are window actions.
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New"), Some("win.new"));
    file_menu.append(Some("Save"), Some("win.save"));
    file_menu.append(Some("Load"), Some("win.load"));
    file_menu.append(Some("Exit"), Some("app.exit"));

    let map_menu = gio::Menu::new();
    map_menu.append(Some("Edit"), Some("win.edit"));

    let menu_model = gio::Menu::new();
    menu_model.append_submenu(Some("File"), &file_menu);
    menu_model.append_submenu(Some("Map"), &map_menu);
    menu_model.append(Some("About"), Some("app.about"));
    PopoverMenuBar::from_model(Some(&menu_model))
}

fn install_actions(
    application: &Application,
    window: &ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    drawing_area: &DrawingArea,
    status_bar: &Statusbar,
    status_context: u32,
) {
    let new_map = gio::SimpleAction::new("new", None);
    let new_state = Rc::clone(state);
    let new_area = drawing_area.clone();
    new_map.connect_activate(move |_, _| {
        new_state.borrow_mut().new_map();
        new_area.queue_draw();
    });
    window.add_action(&new_map);

    let save_map = gio::SimpleAction::new("save", None);
    let save_state = Rc::clone(state);
    let save_window = window.clone();
    save_map.connect_activate(move |_, _| {
        let state = Rc::clone(&save_state);
        let parent = save_window.clone();
        choose_map_file(&save_window, FileChooserAction::Save, move |path| {
            let result = state.borrow().save_map(&path);
            if let Err(error) = result {
                show_error(&parent, "Save", &error);
            }
        });
    });
    window.add_action(&save_map);

    let load_map = gio::SimpleAction::new("load", None);
    let load_state = Rc::clone(state);
    let load_window = window.clone();
    let load_area = drawing_area.clone();
    load_map.connect_activate(move |_, _| {
        let state = Rc::clone(&load_state);
        let parent = load_window.clone();
        let area = load_area.clone();
        choose_map_file(&load_window, FileChooserAction::Open, move |path| {
            let result = state.borrow_mut().load_map(&path);
            match result {
                Ok(()) => area.queue_draw(),
                Err(error) => show_error(&parent, "Load", &error),
            }
        });
    });
    window.add_action(&load_map);

    let exit = gio::SimpleAction::new("exit", None);
    let exit_application = application.clone();
    exit.connect_activate(move |_, _| {
        // Closing every window ends the application, the same way their close buttons do.
        for app_window in exit_application.windows() {
            app_window.close();
        }
    });
    application.add_action(&exit);

    // A boolean state makes the menu show Edit with a check box.
    let edit = gio::SimpleAction::new_stateful("edit", None, &false.to_variant());
    let edit_state = Rc::clone(state);
    let edit_area = drawing_area.clone();
    let edit_status = status_bar.clone();
    edit.connect_activate(move |action, _| {
        let editing = !edit_state.borrow().is_editing();
        edit_state.borrow_mut().set_editing(editing);
        action.set_state(&editing.to_variant());
        update_status(&edit_state, &edit_status, status_context);
        edit_area.queue_draw();
    });
    window.add_action(&edit);

    let about = gio::SimpleAction::new("about", None);
    let parent = window.clone();
    about.connect_activate(move |_, _| show_about(&parent));
    application.add_action(&about);
}

/// Shows the native file chooser for map files and passes the chosen path to `on_chosen`.
fn choose_map_file(
    parent: &ApplicationWindow,
    action: FileChooserAction,
    on_chosen: impl Fn(PathBuf) + 'static,
) {
    let (title, accept_label) = if action == FileChooserAction::Save {
        ("Save Map", "_Save")
    } else {
        ("Load Map", "_Load")
    };
    let dialog = FileChooserNative::new(
        Some(title),
        Some(parent),
        action,
        Some(accept_label),
        Some("_Cancel"),
    );
    dialog.set_modal(true);
    let map_files = FileFilter::new();
    let map_files_name = format!("Map files (*.{FILE_EXTENSION})");
    map_files.set_name(Some(map_files_name.as_str()));
    map_files.add_pattern(&format!("*.{FILE_EXTENSION}"));
    dialog.add_filter(&map_files);
    let all_files = FileFilter::new();
    all_files.set_name(Some("All files"));
    all_files.add_pattern("*");
    dialog.add_filter(&all_files);
    if action == FileChooserAction::Save {
        dialog.set_current_name(DEFAULT_FILE_NAME);
    }

    // GTK does not keep native dialogs alive, so hold a reference until the user responds.
    let keep_alive = RefCell::new(Some(dialog.clone()));
    dialog.connect_response(move |dialog, response| {
        let file = if response == ResponseType::Accept {
            dialog.file()
        } else {
            None
        };
        if let Some(path) = file.and_then(|file| file.path()) {
            on_chosen(path);
        }
        drop(keep_alive.take());
    });
    dialog.show();
}

fn show_about(parent: &ApplicationWindow) {
    let dialog = AboutDialog::builder()
        .transient_for(parent)
        .modal(true)
        .program_name(APP_NAME)
        .version(APP_VERSION)
        .comments(APP_DESCRIPTION)
        .authors([APP_AUTHOR])
        .copyright(APP_COPYRIGHT)
        .license_type(gtk4::License::MitX11)
        .website(APP_WEBSITE)
        .build();
    dialog.present();
}

fn install_vehicle_keys(
    drawing_area: &DrawingArea,
    window: &ApplicationWindow,
    state: &Rc<RefCell<AppState>>,
    status: &Statusbar,
    status_context: u32,
) {
    let pressed_keys = Rc::new(RefCell::new(Vec::<gdk::Key>::new()));
    let controller = EventControllerKey::new();
    let press_state = Rc::clone(state);
    let press_status = status.clone();
    let press_area = drawing_area.clone();
    let press_window = window.clone();
    let press_keys = Rc::clone(&pressed_keys);
    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            press_window.close();
            return glib::Propagation::Stop;
        }
        if press_keys.borrow().contains(&key) {
            return glib::Propagation::Stop;
        }
        press_keys.borrow_mut().push(key);

        let vehicle_key = match key {
            gdk::Key::Up => Some(VehicleKey::Forward),
            gdk::Key::Down => Some(VehicleKey::Reverse),
            gdk::Key::Left => Some(VehicleKey::TurnLeft),
            gdk::Key::Right => Some(VehicleKey::TurnRight),
            _ => None,
        };
        if let Some(vehicle_key) = vehicle_key {
            let mut state = press_state.borrow_mut();
            state.handle_vehicle_key(vehicle_key);
            press_status.remove_all(status_context);
            press_status.push(status_context, &state.status_text());
            press_area.queue_draw();
        }
        glib::Propagation::Stop
    });

    let release_keys = Rc::clone(&pressed_keys);
    controller.connect_key_released(move |_, key, _, _| {
        release_keys.borrow_mut().retain(|pressed| *pressed != key);
    });
    drawing_area.add_controller(controller);
}

fn draw_grid(
    state: &Rc<RefCell<AppState>>,
    context: &gtk4::cairo::Context,
    width: i32,
    height: i32,
) {
    let state = state.borrow();
    let pixels = state
        .buffer
        .iter()
        .flat_map(|pixel| pixel.to_ne_bytes())
        .collect::<Vec<_>>();
    let surface = match ImageSurface::create_for_data(
        pixels,
        Format::Rgb24,
        WIDTH as i32,
        HEIGHT as i32,
        WIDTH as i32 * 4,
    ) {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("Unable to create the grid image: {error}");
            return;
        }
    };

    let (x, y, scale) = grid_placement(f64::from(width), f64::from(height));
    if scale <= 0.0 {
        return;
    }
    if let Err(error) = context.save() {
        eprintln!("Unable to prepare the grid drawing: {error}");
        return;
    }
    context.translate(x, y);
    context.scale(scale, scale);
    if let Err(error) = context.set_source_surface(&surface, 0.0, 0.0) {
        eprintln!("Unable to display the grid image: {error}");
    } else if let Err(error) = context.paint() {
        eprintln!("Unable to draw the grid image: {error}");
    }
    if let Err(error) = context.restore() {
        eprintln!("Unable to restore the grid drawing context: {error}");
    }
}

/// Where the grid is drawn in an area of the given size: as large as fits, and centered. Returns
/// the left and top edges of the grid and its scale.
fn grid_placement(width: f64, height: f64) -> (f64, f64, f64) {
    let scale = (width / WIDTH as f64).min(height / HEIGHT as f64);
    let left = (width - WIDTH as f64 * scale) / 2.0;
    let top = (height - HEIGHT as f64 * scale) / 2.0;
    (left, top, scale)
}

/// Lets the mouse edit the map in edit mode. Holding a button while moving the pointer paints
/// every cell it crosses.
fn install_map_editing(drawing_area: &DrawingArea, state: &Rc<RefCell<AppState>>) {
    let drag = GestureDrag::new();
    // Listen to every mouse button, not only the left one.
    drag.set_button(0);

    let begin_state = Rc::clone(state);
    let begin_area = drawing_area.clone();
    drag.connect_drag_begin(move |gesture, x, y| {
        begin_area.grab_focus();
        paint_cell(&begin_state, &begin_area, gesture.current_button(), x, y);
    });

    let update_state = Rc::clone(state);
    let update_area = drawing_area.clone();
    drag.connect_drag_update(move |gesture, offset_x, offset_y| {
        if let Some((x, y)) = gesture.start_point() {
            let button = gesture.current_button();
            paint_cell(&update_state, &update_area, button, x + offset_x, y + offset_y);
        }
    });
    drawing_area.add_controller(drag);
}

/// Applies a mouse button to the cell under `(x, y)`, in drawing area coordinates.
fn paint_cell(state: &Rc<RefCell<AppState>>, area: &DrawingArea, button: u32, x: f64, y: f64) {
    let button = match button {
        gdk::BUTTON_PRIMARY => MouseButton::Left,
        gdk::BUTTON_SECONDARY => MouseButton::Right,
        _ => return,
    };
    let (left, top, scale) = grid_placement(f64::from(area.width()), f64::from(area.height()));
    let (width, height) = (WIDTH as f64 * scale, HEIGHT as f64 * scale);
    let Some((row, col)) = AppState::cell_at(x, y, left, top, width, height) else {
        return;
    };
    if state.borrow_mut().edit_cell(row, col, button) {
        area.queue_draw();
    }
}

/// Panel that explains the map colors, shown between the map and the status bar.
fn create_legend() -> GtkBox {
    let legend = GtkBox::new(Orientation::Horizontal, 2 * LEGEND_SPACING);
    legend.set_height_request(LEGEND_HEIGHT);
    legend.set_margin_start(LEGEND_SPACING);
    legend.set_margin_end(LEGEND_SPACING);
    for (label, color) in LEGEND {
        let swatch = DrawingArea::new();
        swatch.set_content_width(LEGEND_SWATCH);
        swatch.set_content_height(LEGEND_SWATCH);
        swatch.set_valign(Align::Center);
        swatch.set_draw_func(move |_, context, width, height| {
            draw_swatch(context, color, width, height);
        });

        let entry = GtkBox::new(Orientation::Horizontal, LEGEND_SPACING / 2);
        entry.append(&swatch);
        entry.append(&Label::new(Some(label)));
        legend.append(&entry);
    }
    legend
}

/// Fills a legend color sample and outlines it in black.
fn draw_swatch(context: &gtk4::cairo::Context, color: u32, width: i32, height: i32) {
    let channel = |shift: u32| f64::from((color >> shift) & 0xff) / 255.0;
    let (width, height) = (f64::from(width), f64::from(height));
    context.set_source_rgb(channel(16), channel(8), channel(0));
    context.rectangle(0.0, 0.0, width, height);
    if let Err(error) = context.fill() {
        eprintln!("Unable to draw the legend: {error}");
        return;
    }
    context.set_source_rgb(0.0, 0.0, 0.0);
    context.set_line_width(1.0);
    context.rectangle(0.5, 0.5, width - 1.0, height - 1.0);
    if let Err(error) = context.stroke() {
        eprintln!("Unable to draw the legend: {error}");
    }
}

fn show_error(parent: &ApplicationWindow, title: &str, message: &str) {
    show_message(parent, title, message, MessageType::Error);
}

fn show_message(parent: &ApplicationWindow, title: &str, text: &str, message_type: MessageType) {
    let dialog = MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(message_type)
        .buttons(gtk4::ButtonsType::Ok)
        .text(text)
        .build();
    dialog.set_title(Some(title));
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}

fn update_status(state: &Rc<RefCell<AppState>>, status: &Statusbar, status_context: u32) {
    status.remove_all(status_context);
    status.push(status_context, &state.borrow().status_text());
}
