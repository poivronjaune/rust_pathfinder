use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::cairo::{Format, ImageSurface};
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    AboutDialog, Align, Application, ApplicationWindow, Box as GtkBox, Button, DrawingArea,
    DropDown, Entry, EventControllerKey, FileChooserAction, FileChooserNative, FileFilter,
    GestureDrag, Grid as GtkGrid, InputPurpose, Label, MessageDialog, MessageType, Orientation,
    PopoverMenuBar, ResponseType, Separator, Statusbar, Window,
};

use crate::app::{AppState, HEADINGS, MapSettings, MouseButton, VehicleKey};
use crate::config::{
    APP_AUTHOR, APP_COPYRIGHT, APP_DESCRIPTION, APP_NAME, APP_VERSION, APP_WEBSITE, CELL_SIZE,
    DEFAULT_GRID_SIZE, MAX_GRID_SIZE, MIN_GRID_SIZE,
};
use crate::map_file::{DEFAULT_FILE_NAME, FILE_EXTENSION};
use crate::recent::RecentFiles;
use crate::render::LEGEND;

/// Size the map area asks for: a map of the default size drawn at its natural size. Other maps
/// are scaled to fit.
const MAP_AREA_SIZE: i32 = (DEFAULT_GRID_SIZE * CELL_SIZE) as i32;
/// Height of the legend panel between the map and the status bar.
const LEGEND_HEIGHT: i32 = 32;
/// Size of the color samples in the legend.
const LEGEND_SWATCH: i32 = 14;
/// Space around the legend and between its entries.
const LEGEND_SPACING: i32 = 8;

/// The main window's widgets and the state they show.
struct Ui {
    window: ApplicationWindow,
    state: RefCell<AppState>,
    recent: RefCell<RecentFiles>,
    /// File menu section listing the recent files, rebuilt whenever they change.
    recent_menu: gio::Menu,
    drawing_area: DrawingArea,
    status_bar: Statusbar,
    status_context: u32,
}

impl Ui {
    fn update_status(&self) {
        self.status_bar.remove_all(self.status_context);
        self.status_bar
            .push(self.status_context, &self.state.borrow().status_text());
    }

    /// Shows a replaced or resized map: the window title, the status bar and the whole grid.
    fn show_new_map(&self) {
        let title = self.state.borrow().title();
        self.window.set_title(Some(title.as_str()));
        self.update_status();
        self.drawing_area.queue_draw();
    }

    fn update_recent_menu(&self) {
        fill_recent_menu(&self.recent_menu, &self.recent.borrow());
    }

    fn remember_file(&self, path: &Path) {
        self.recent.borrow_mut().add(path);
        self.update_recent_menu();
    }

    /// Loads a map file and moves it to the top of the recent files. A recent file that no
    /// longer exists is removed from the list.
    fn open_map_file(&self, path: &Path) {
        let result = self.state.borrow_mut().load_map(path);
        match result {
            Ok(()) => {
                self.remember_file(path);
                self.show_new_map();
            }
            Err(mut error) => {
                if !path.exists() && self.recent.borrow_mut().remove(path) {
                    self.update_recent_menu();
                    error.push_str("\n\nThe file was removed from the recent files list.");
                }
                show_error(&self.window, "Load", &error);
            }
        }
    }
}

pub(super) fn run() -> Result<(), String> {
    let application = Application::builder()
        .application_id("com.poivronjaune.pathplanner")
        .build();
    application.connect_activate(build_window);
    application.run();
    Ok(())
}

fn build_window(application: &Application) {
    let state = AppState::new();
    let window = ApplicationWindow::builder()
        .application(application)
        .title(state.title())
        .default_width(MAP_AREA_SIZE)
        .default_height(MAP_AREA_SIZE + 58 + LEGEND_HEIGHT)
        .build();

    let layout = GtkBox::new(Orientation::Vertical, 0);
    let recent_menu = gio::Menu::new();
    let menu_bar = create_menu_bar(&recent_menu);
    let drawing_area = DrawingArea::new();
    drawing_area.set_content_width(MAP_AREA_SIZE);
    drawing_area.set_content_height(MAP_AREA_SIZE);
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

    let ui = Rc::new(Ui {
        window,
        state: RefCell::new(state),
        recent: RefCell::new(RecentFiles::load()),
        recent_menu,
        drawing_area,
        status_bar,
        status_context,
    });
    ui.update_recent_menu();
    ui.update_status();
    let draw_ui = Rc::clone(&ui);
    ui.drawing_area
        .set_draw_func(move |_, context, width, height| {
            draw_grid(&draw_ui.state.borrow(), context, width, height);
        });

    install_actions(application, &ui);
    install_vehicle_keys(&ui);
    install_map_editing(&ui);
    ui.window.present();
    ui.drawing_area.grab_focus();
}

fn create_menu_bar(recent_menu: &gio::Menu) -> PopoverMenuBar {
    // Commands that act on the window's own map are window actions.
    let file_commands = gio::Menu::new();
    file_commands.append(Some("New"), Some("win.new"));
    file_commands.append(Some("Save"), Some("win.save"));
    file_commands.append(Some("Load"), Some("win.load"));
    let exit_section = gio::Menu::new();
    exit_section.append(Some("Exit"), Some("app.exit"));

    // Sections are separated by lines, which frame the recent files like Word and Excel do.
    let file_menu = gio::Menu::new();
    file_menu.append_section(None, &file_commands);
    file_menu.append_section(None, recent_menu);
    file_menu.append_section(None, &exit_section);

    let edit_section = gio::Menu::new();
    edit_section.append(Some("Edit"), Some("win.edit"));
    let size_section = gio::Menu::new();
    size_section.append(Some("Grid Size..."), Some("win.grid-size"));
    let map_menu = gio::Menu::new();
    map_menu.append_section(None, &edit_section);
    map_menu.append_section(None, &size_section);

    let menu_model = gio::Menu::new();
    menu_model.append_submenu(Some("File"), &file_menu);
    menu_model.append_submenu(Some("Map"), &map_menu);
    menu_model.append(Some("About"), Some("app.about"));
    PopoverMenuBar::from_model(Some(&menu_model))
}

/// Lists the recent files, numbered with their access keys, then Clear Recent Files.
fn fill_recent_menu(menu: &gio::Menu, recent: &RecentFiles) {
    menu.remove_all();
    for index in 0..recent.paths().len() {
        let Some(label) = recent.label(index) else {
            continue;
        };
        // An underscore marks the access key in a menu, so literal underscores are doubled.
        let text = format!("_{} {}", index + 1, label.replace('_', "__"));
        let item = gio::MenuItem::new(Some(text.as_str()), None);
        item.set_action_and_target_value(
            Some("win.open-recent"),
            Some(&(index as u32).to_variant()),
        );
        menu.append_item(&item);
    }
    if !recent.paths().is_empty() {
        menu.append(Some("Clear Recent Files"), Some("win.clear-recent"));
    }
}

fn install_actions(application: &Application, ui: &Rc<Ui>) {
    let window = &ui.window;

    let new_map = gio::SimpleAction::new("new", None);
    let new_ui = Rc::clone(ui);
    new_map.connect_activate(move |_, _| {
        new_ui.state.borrow_mut().new_map();
        new_ui.show_new_map();
    });
    window.add_action(&new_map);

    let save_map = gio::SimpleAction::new("save", None);
    let save_ui = Rc::clone(ui);
    save_map.connect_activate(move |_, _| {
        let ui = Rc::clone(&save_ui);
        choose_map_file(&save_ui.window, FileChooserAction::Save, move |path| {
            let result = ui.state.borrow().save_map(&path);
            match result {
                Ok(()) => ui.remember_file(&path),
                Err(error) => show_error(&ui.window, "Save", &error),
            }
        });
    });
    window.add_action(&save_map);

    let load_map = gio::SimpleAction::new("load", None);
    let load_ui = Rc::clone(ui);
    load_map.connect_activate(move |_, _| {
        let ui = Rc::clone(&load_ui);
        choose_map_file(&load_ui.window, FileChooserAction::Open, move |path| {
            ui.open_map_file(&path);
        });
    });
    window.add_action(&load_map);

    // Each recent file in the File menu passes its position in the list.
    let open_recent = gio::SimpleAction::new("open-recent", Some(glib::VariantTy::UINT32));
    let open_recent_ui = Rc::clone(ui);
    open_recent.connect_activate(move |_, parameter| {
        let index = parameter.and_then(|parameter| parameter.get::<u32>());
        let path = index.and_then(|index| {
            let recent = open_recent_ui.recent.borrow();
            recent.get(index as usize).map(Path::to_path_buf)
        });
        if let Some(path) = path {
            open_recent_ui.open_map_file(&path);
        }
    });
    window.add_action(&open_recent);

    let clear_recent = gio::SimpleAction::new("clear-recent", None);
    let clear_ui = Rc::clone(ui);
    clear_recent.connect_activate(move |_, _| {
        clear_ui.recent.borrow_mut().clear();
        clear_ui.update_recent_menu();
    });
    window.add_action(&clear_recent);

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
    let edit_ui = Rc::clone(ui);
    edit.connect_activate(move |action, _| {
        let editing = !edit_ui.state.borrow().is_editing();
        edit_ui.state.borrow_mut().set_editing(editing);
        action.set_state(&editing.to_variant());
        edit_ui.update_status();
        edit_ui.drawing_area.queue_draw();
    });
    window.add_action(&edit);

    let grid_size = gio::SimpleAction::new("grid-size", None);
    let grid_size_ui = Rc::clone(ui);
    grid_size.connect_activate(move |_, _| {
        let ui = Rc::clone(&grid_size_ui);
        let initial = grid_size_ui.state.borrow().settings();
        show_settings_dialog(&grid_size_ui.window, initial, move |settings| {
            ui.state.borrow_mut().apply_settings(settings);
            ui.show_new_map();
        });
    });
    window.add_action(&grid_size);

    let about = gio::SimpleAction::new("about", None);
    let parent = window.clone();
    about.connect_activate(move |_, _| show_about(&parent));
    application.add_action(&about);
}

/// Shows the Grid Size dialog, which asks for the size of the map and where the car starts, and
/// passes the accepted settings to `on_accept`.
fn show_settings_dialog(
    parent: &ApplicationWindow,
    initial: MapSettings,
    on_accept: impl Fn(MapSettings) + 'static,
) {
    let dialog = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Grid Size")
        .resizable(false)
        .build();

    let rows = number_entry(initial.rows);
    let cols = number_entry(initial.cols);
    let start_row = number_entry(initial.start.row + 1);
    let start_col = number_entry(initial.start.col + 1);
    let heading_names: Vec<&str> = HEADINGS.iter().map(|&(name, _)| name).collect();
    let heading = DropDown::from_strings(&heading_names);
    heading.set_selected(initial.heading_index() as u32);
    heading.set_halign(Align::Start);

    let size_hint = format!("({MIN_GRID_SIZE} to {MAX_GRID_SIZE})");
    let form = GtkGrid::new();
    form.set_row_spacing(6);
    form.set_column_spacing(12);
    form.attach(&section_title("Grid size"), 0, 0, 3, 1);
    attach_field(&form, 1, "Rows:", &rows, &size_hint);
    attach_field(&form, 2, "Columns:", &cols, &size_hint);
    let start_title = section_title("Car start");
    start_title.set_margin_top(6);
    form.attach(&start_title, 0, 3, 3, 1);
    attach_field(&form, 4, "Row:", &start_row, "(1 to rows)");
    attach_field(&form, 5, "Column:", &start_col, "(1 to columns)");
    form.attach(&form_label("Heading:"), 0, 6, 1, 1);
    form.attach(&heading, 1, 6, 2, 1);

    let cancel = Button::with_label("Cancel");
    let ok = Button::with_label("OK");
    ok.add_css_class("suggested-action");
    let buttons = GtkBox::new(Orientation::Horizontal, 8);
    buttons.set_halign(Align::End);
    buttons.set_margin_top(16);
    buttons.append(&cancel);
    buttons.append(&ok);

    let content = GtkBox::new(Orientation::Vertical, 0);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.append(&form);
    content.append(&buttons);
    dialog.set_child(Some(&content));
    // Enter in a field presses OK.
    dialog.set_default_widget(Some(&ok));

    let cancel_dialog = dialog.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(dialog) = cancel_dialog.upgrade() {
            dialog.close();
        }
    });

    let ok_dialog = dialog.downgrade();
    let fields = [rows.clone(), cols, start_row, start_col];
    ok.connect_clicked(move |_| {
        let Some(dialog) = ok_dialog.upgrade() else {
            return;
        };
        let [rows, cols, start_row, start_col] = fields.each_ref().map(|field| field.text());
        let heading = usize::try_from(heading.selected()).unwrap_or(usize::MAX);
        match MapSettings::parse(&rows, &cols, &start_row, &start_col, heading) {
            Ok(settings) => {
                dialog.close();
                on_accept(settings);
            }
            Err(error) => show_error(&dialog, "Grid Size", &error),
        }
    });

    let keys = EventControllerKey::new();
    let escape_dialog = dialog.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key != gdk::Key::Escape {
            return glib::Propagation::Proceed;
        }
        if let Some(dialog) = escape_dialog.upgrade() {
            dialog.close();
        }
        glib::Propagation::Stop
    });
    dialog.add_controller(keys);

    dialog.present();
    rows.grab_focus();
    rows.select_region(0, -1);
}

/// Bold title of a group of fields in the Grid Size dialog.
fn section_title(text: &str) -> Label {
    Label::builder()
        .label(format!("<b>{text}</b>"))
        .use_markup(true)
        .halign(Align::Start)
        .build()
}

fn form_label(text: &str) -> Label {
    Label::builder().label(text).halign(Align::Start).build()
}

/// Adds a labeled field to row `row` of the Grid Size form, with a hint about its range.
fn attach_field(form: &GtkGrid, row: i32, label: &str, field: &Entry, hint: &str) {
    form.attach(&form_label(label), 0, row, 1, 1);
    form.attach(field, 1, row, 1, 1);
    let hint = form_label(hint);
    hint.add_css_class("dim-label");
    form.attach(&hint, 2, row, 1, 1);
}

/// Text field for a number of up to three digits.
fn number_entry(value: usize) -> Entry {
    let entry = Entry::new();
    entry.set_text(&value.to_string());
    entry.set_width_chars(5);
    entry.set_max_length(3);
    entry.set_input_purpose(InputPurpose::Digits);
    entry.set_activates_default(true);
    entry.set_halign(Align::Start);
    entry
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

fn install_vehicle_keys(ui: &Rc<Ui>) {
    let pressed_keys = Rc::new(RefCell::new(Vec::<gdk::Key>::new()));
    let controller = EventControllerKey::new();
    let press_ui = Rc::clone(ui);
    let press_keys = Rc::clone(&pressed_keys);
    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            press_ui.window.close();
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
            // The keys do nothing in edit mode.
            let handled = press_ui
                .state
                .borrow_mut()
                .handle_vehicle_key(vehicle_key)
                .is_some();
            if handled {
                press_ui.update_status();
                press_ui.drawing_area.queue_draw();
            }
        }
        glib::Propagation::Stop
    });

    let release_keys = Rc::clone(&pressed_keys);
    controller.connect_key_released(move |_, key, _, _| {
        release_keys.borrow_mut().retain(|pressed| *pressed != key);
    });
    ui.drawing_area.add_controller(controller);
}

fn draw_grid(state: &AppState, context: &gtk4::cairo::Context, width: i32, height: i32) {
    let (map_width, map_height) = (state.pixel_width() as i32, state.pixel_height() as i32);
    let pixels = state
        .buffer
        .iter()
        .flat_map(|pixel| pixel.to_ne_bytes())
        .collect::<Vec<_>>();
    let surface = match ImageSurface::create_for_data(
        pixels,
        Format::Rgb24,
        map_width,
        map_height,
        map_width * 4,
    ) {
        Ok(surface) => surface,
        Err(error) => {
            eprintln!("Unable to create the grid image: {error}");
            return;
        }
    };

    let (x, y, scale) = grid_placement(state, f64::from(width), f64::from(height));
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

/// Where the map is drawn in an area of the given size: as large as fits, and centered. Returns
/// the left and top edges of the map and its scale.
fn grid_placement(state: &AppState, width: f64, height: f64) -> (f64, f64, f64) {
    let (map_width, map_height) = (state.pixel_width() as f64, state.pixel_height() as f64);
    let scale = (width / map_width).min(height / map_height);
    let left = (width - map_width * scale) / 2.0;
    let top = (height - map_height * scale) / 2.0;
    (left, top, scale)
}

/// Lets the mouse edit the map in edit mode. Holding a button while moving the pointer paints
/// every cell it crosses.
fn install_map_editing(ui: &Rc<Ui>) {
    let drag = GestureDrag::new();
    // Listen to every mouse button, not only the left one.
    drag.set_button(0);

    let begin_ui = Rc::clone(ui);
    drag.connect_drag_begin(move |gesture, x, y| {
        begin_ui.drawing_area.grab_focus();
        paint_cell(&begin_ui, gesture.current_button(), x, y);
    });

    let update_ui = Rc::clone(ui);
    drag.connect_drag_update(move |gesture, offset_x, offset_y| {
        if let Some((x, y)) = gesture.start_point() {
            let button = gesture.current_button();
            paint_cell(&update_ui, button, x + offset_x, y + offset_y);
        }
    });
    ui.drawing_area.add_controller(drag);
}

/// Applies a mouse button to the cell under `(x, y)`, in drawing area coordinates.
fn paint_cell(ui: &Ui, button: u32, x: f64, y: f64) {
    let button = match button {
        gdk::BUTTON_PRIMARY => MouseButton::Left,
        gdk::BUTTON_SECONDARY => MouseButton::Right,
        _ => return,
    };
    let area = &ui.drawing_area;
    let changed = {
        let mut state = ui.state.borrow_mut();
        let (area_width, area_height) = (f64::from(area.width()), f64::from(area.height()));
        let (left, top, scale) = grid_placement(&state, area_width, area_height);
        let width = state.pixel_width() as f64 * scale;
        let height = state.pixel_height() as f64 * scale;
        match state.cell_at(x, y, left, top, width, height) {
            Some((row, col)) => state.edit_cell(row, col, button),
            None => false,
        }
    };
    if changed {
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

fn show_error(parent: &impl IsA<Window>, title: &str, message: &str) {
    show_message(parent, title, message, MessageType::Error);
}

fn show_message(parent: &impl IsA<Window>, title: &str, text: &str, message_type: MessageType) {
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
