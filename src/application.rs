use log::{debug, info};

use adw::{prelude::*, subclass::prelude::*};
use glib::clone;
use gtk::{gio, glib};

use crate::config::{self, APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::ui::window::MainWindow;

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::OnceCell;

    #[derive(Debug, Default)]
    pub struct Application {
        pub window: OnceCell<WeakRef<MainWindow>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            debug!("GtkApplication<Application>::activate");
            self.parent_activate();
            let app = self.instance();

            if let Some(window) = self.window.get() {
                let window = window.upgrade().unwrap();
                window.present();
                return;
            }

            let window = MainWindow::new(&*app);
            self.window
                .set(window.downgrade())
                .expect("Window already set.");

            app.main_window().present();
        }

        fn startup(&self) {
            debug!("GtkApplication<Application>::startup");
            self.parent_startup();
            let app = self.instance();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            app.setup_gactions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for Application {}

    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[
            ("application-id", &Some(APP_ID)),
            ("flags", &gio::ApplicationFlags::empty()),
            ("resource-base-path", &Some("/me/nalux/Resources/")),
        ])
    }

    fn main_window(&self) -> MainWindow {
        self.imp().window.get().unwrap().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(clone!(@weak self as app => move |_, _| {
            // This is needed to trigger the delete event and saving the window state
            app.main_window().close();
            app.quit();
        }));
        self.add_action(&action_quit);

        // About
        let action_about = gio::SimpleAction::new("about", None);
        action_about.connect_activate(clone!(@weak self as app => move |_, _| {
            app.show_about_dialog();
        }));
        self.add_action(&action_about);
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
    }

    fn show_about_dialog(&self) {
        let about = adw::AboutWindow::builder()
            .application_name(&gettextrs::gettext("Resources"))
            .application_icon(config::APP_ID)
            .developer_name(&gettextrs::gettext("The Nalux Team"))
            .developers(vec!["ManicRobot <manicrobot@protonmail.com>".to_string()])
            .license_type(gtk::License::Gpl30)
            .version(config::VERSION)
            .website("https://github.com/NaluxOS/resources")
            .build();

        about.add_link(
            &gettextrs::gettext("Report Issues"),
            "https://github.com/NaluxOS/resources/issues",
        );

        about.add_credit_section(Some(&gettextrs::gettext("Icon by")), &["Avhiren"]);

        about.set_transient_for(Some(&self.main_window()));
        about.set_modal(true);

        about.present();
    }

    pub fn run(&self) {
        info!("Resources ({})", APP_ID);
        info!("Version: {} ({})", VERSION, PROFILE);
        info!("Datadir: {}", PKGDATADIR);

        ApplicationExtManual::run(self);
    }
}

impl Default for Application {
    fn default() -> Self {
        gio::Application::default()
            .expect("Could not get default GApplication")
            .downcast()
            .unwrap()
    }
}
