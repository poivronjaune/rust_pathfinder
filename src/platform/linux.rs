use std::cell::RefCell;
use std::rc::Rc;

use gtk4::cairo::{Format, ImageSurface};
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    AboutDialog, Application, ApplicationWindow, Box as GtkBox, DrawingArea, EventControllerKey,
    MessageDialog, MessageType, Orientation, PopoverMenuBar, Statusbar,
};

use crate::app::{AppState, VehicleKey};
use crate::config::{
    APP_AUTHOR, APP_COPYRIGHT, APP_DESCRIPTION, APP_NAME, APP_VERSION, APP_WEBSITE, HEIGHT, WIDTH,
};

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
        .default_height(HEIGHT as i32 + 58)
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
    layout.append(&status_bar);
    window.set_child(Some(&layout));

    update_status(&state, &status_bar, status_context);
    let draw_state = Rc::clone(&state);
    drawing_area.set_draw_func(move |_, context, width, height| {
        draw_grid(&draw_state, context, width, height);
    });

    install_actions(application, &window);
    install_vehicle_keys(&drawing_area, &window, &state, &status_bar, status_context);
    window.present();
    drawing_area.grab_focus();
}

fn create_menu_bar() -> PopoverMenuBar {
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New"), Some("app.new"));
    file_menu.append(Some("Save"), Some("app.save"));
    file_menu.append(Some("Load"), Some("app.load"));
    file_menu.append(Some("Exit"), Some("app.exit"));

    let map_menu = gio::Menu::new();
    map_menu.append(Some("Edit"), Some("app.edit"));

    let menu_model = gio::Menu::new();
    menu_model.append_submenu(Some("File"), &file_menu);
    menu_model.append_submenu(Some("Map"), &map_menu);
    menu_model.append(Some("About"), Some("app.about"));
    PopoverMenuBar::from_model(Some(&menu_model))
}

fn install_actions(application: &Application, window: &ApplicationWindow) {
    for (name, label) in [
        ("new", "New"),
        ("save", "Save"),
        ("load", "Load"),
        ("exit", "Exit"),
        ("edit", "Edit"),
    ] {
        let action = gio::SimpleAction::new(name, None);
        let parent = window.clone();
        action.connect_activate(move |_, _| {
            show_placeholder(&parent, label);
        });
        application.add_action(&action);
    }

    let about = gio::SimpleAction::new("about", None);
    let parent = window.clone();
    about.connect_activate(move |_, _| show_about(&parent));
    application.add_action(&about);
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

    let scale = (width as f64 / WIDTH as f64).min(height as f64 / HEIGHT as f64);
    if scale <= 0.0 {
        return;
    }
    let x = (width as f64 - WIDTH as f64 * scale) / 2.0;
    let y = (height as f64 - HEIGHT as f64 * scale) / 2.0;
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

fn show_placeholder(parent: &ApplicationWindow, title: &str) {
    let dialog = MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(MessageType::Info)
        .buttons(gtk4::ButtonsType::Ok)
        .text(format!("{title} is not implemented yet."))
        .build();
    dialog.set_title(Some(title));
    dialog.connect_response(|dialog, _| dialog.close());
    dialog.present();
}

fn update_status(state: &Rc<RefCell<AppState>>, status: &Statusbar, status_context: u32) {
    status.push(status_context, &state.borrow().status_text());
}
