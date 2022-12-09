use std::cell::Ref;

use adw::{prelude::*, subclass::prelude::*};
use adw::{ResponseAppearance, Toast};
use gtk::glib::{self, clone, BoxedAnyObject};
use gtk::{gio, CustomSorter, SortType};

use crate::config::PROFILE;
use crate::ui::dialogs::app_dialog::ResAppDialog;
use crate::ui::widgets::application_name_cell::ResApplicationNameCell;
use crate::ui::window::MainWindow;
use crate::utils::processes::{Apps, SimpleItem};
use crate::utils::units::{to_largest_unit, Base};

mod imp {
    use std::cell::RefCell;

    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/me/nalux/Resources/ui/pages/applications.ui")]
    pub struct ResApplications {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub applications_scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub information_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub end_application_button: TemplateChild<adw::SplitButton>,

        pub apps: RefCell<Apps>,
        pub store: RefCell<gio::ListStore>,
        pub selection_model: RefCell<gtk::SingleSelection>,
        pub sort_model: RefCell<gtk::SortListModel>,
        pub column_view: RefCell<gtk::ColumnView>,
        pub open_dialog: RefCell<Option<(String, ResAppDialog)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResApplications {
        const NAME: &'static str = "ResApplications";
        type Type = super::ResApplications;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action(
                "applications.kill-application",
                None,
                move |resapplications, _, _| {
                    resapplications.kill_selected_application();
                },
            );

            klass.install_action(
                "applications.halt-application",
                None,
                move |resapplications, _, _| {
                    resapplications.halt_selected_application();
                },
            );

            klass.install_action(
                "applications.continue-application",
                None,
                move |resapplications, _, _| {
                    resapplications.continue_selected_application();
                },
            );

            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResApplications {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.instance();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }
    }

    impl WidgetImpl for ResApplications {}
    impl BinImpl for ResApplications {}
}

glib::wrapper! {
    pub struct ResApplications(ObjectSubclass<imp::ResApplications>)
        @extends gtk::Widget, adw::Bin;
}

impl ResApplications {
    pub fn new() -> Self {
        glib::Object::new::<Self>(&[])
    }

    pub fn init(&self) {
        self.setup_widgets();
        self.setup_signals();
        self.setup_listener();
    }

    pub fn setup_widgets(&self) {
        let imp = self.imp();

        let column_view = gtk::ColumnView::new(None::<&gtk::SingleSelection>);
        let store = gio::ListStore::new(BoxedAnyObject::static_type());
        let sort_model = gtk::SortListModel::new(Some(&store), column_view.sorter().as_ref());
        let selection_model = gtk::SingleSelection::new(Some(&sort_model));
        column_view.set_model(Some(&selection_model));
        selection_model.set_can_unselect(true);
        selection_model.set_autoselect(false);

        *imp.selection_model.borrow_mut() = selection_model;
        *imp.sort_model.borrow_mut() = sort_model;
        *imp.store.borrow_mut() = store;

        let name_col_factory = gtk::SignalListItemFactory::new();
        let name_col = gtk::ColumnViewColumn::new(
            Some(&gettextrs::gettext("Application")),
            Some(&name_col_factory),
        );
        name_col.set_resizable(true);
        name_col.set_expand(true);
        name_col_factory.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = ResApplicationNameCell::new();
            item.set_child(Some(&row));
        });
        name_col_factory.connect_bind(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let child = item
                .child()
                .unwrap()
                .downcast::<ResApplicationNameCell>()
                .unwrap();
            let entry = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
            let r: Ref<SimpleItem> = entry.borrow();
            child.set_name(&r.display_name);
            child.set_icon(Some(&r.icon));
        });
        let name_col_sorter = CustomSorter::new(move |a, b| {
            let item_a = a
                .downcast_ref::<BoxedAnyObject>()
                .unwrap()
                .borrow::<SimpleItem>();
            let item_b = b
                .downcast_ref::<BoxedAnyObject>()
                .unwrap()
                .borrow::<SimpleItem>();
            item_a.display_name.cmp(&item_b.display_name).into()
        });
        name_col.set_sorter(Some(&name_col_sorter));

        let memory_col_factory = gtk::SignalListItemFactory::new();
        let memory_col = gtk::ColumnViewColumn::new(
            Some(&gettextrs::gettext("Memory")),
            Some(&memory_col_factory),
        );
        memory_col.set_resizable(true);
        memory_col_factory.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = gtk::Inscription::new(None);
            item.set_child(Some(&row));
        });
        memory_col_factory.connect_bind(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let child = item
                .child()
                .unwrap()
                .downcast::<gtk::Inscription>()
                .unwrap();
            let entry = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
            let r: Ref<SimpleItem> = entry.borrow();
            let (number, prefix) = to_largest_unit(r.memory_usage as f64, &Base::Decimal);
            child.set_text(Some(&format!("{number:.1} {prefix}B")));
        });
        let memory_col_sorter = CustomSorter::new(move |a, b| {
            let item_a = a
                .downcast_ref::<BoxedAnyObject>()
                .unwrap()
                .borrow::<SimpleItem>();
            let item_b = b
                .downcast_ref::<BoxedAnyObject>()
                .unwrap()
                .borrow::<SimpleItem>();
            item_a.memory_usage.cmp(&item_b.memory_usage).into()
        });
        memory_col.set_sorter(Some(&memory_col_sorter));

        let cpu_col_factory = gtk::SignalListItemFactory::new();
        let cpu_col = gtk::ColumnViewColumn::new(
            Some(&gettextrs::gettext("Processor")),
            Some(&cpu_col_factory),
        );
        cpu_col.set_resizable(true);
        cpu_col_factory.connect_setup(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let row = gtk::Inscription::new(None);
            item.set_child(Some(&row));
        });
        cpu_col_factory.connect_bind(move |_factory, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let child = item
                .child()
                .unwrap()
                .downcast::<gtk::Inscription>()
                .unwrap();
            let entry = item.item().unwrap().downcast::<BoxedAnyObject>().unwrap();
            let r: Ref<SimpleItem> = entry.borrow();
            child.set_text(Some(&format!(
                "{:.1} %",
                r.cpu_time_ratio.unwrap_or(0.0) * 100.0
            )));
        });
        let cpu_col_sorter = CustomSorter::new(move |a, b| {
            let item_a = a
                .downcast_ref::<BoxedAnyObject>()
                .unwrap()
                .borrow::<SimpleItem>();
            let item_b = b
                .downcast_ref::<BoxedAnyObject>()
                .unwrap()
                .borrow::<SimpleItem>();
            // floats can only be partially ordered, so just multiply our
            // 0.0-1.0 floats by u16::MAX to get an "accurate enough" integer
            // and then compare them instead
            let ratio_a = (item_a.cpu_time_ratio.unwrap_or(0.0) * f32::from(u16::MAX)) as u16;
            let ratio_b = (item_b.cpu_time_ratio.unwrap_or(0.0) * f32::from(u16::MAX)) as u16;
            ratio_a.cmp(&ratio_b).into()
        });
        cpu_col.set_sorter(Some(&cpu_col_sorter));

        column_view.append_column(&name_col);
        column_view.append_column(&memory_col);
        column_view.append_column(&cpu_col);
        column_view.sort_by_column(Some(&name_col), SortType::Ascending);
        column_view.set_enable_rubberband(true);
        imp.applications_scrolled_window
            .set_child(Some(&column_view));
        *imp.column_view.borrow_mut() = column_view;
    }

    pub fn setup_signals(&self) {
        let imp = self.imp();

        imp.selection_model.borrow().connect_selection_changed(
            clone!(@strong self as this => move |model, _, _| {
                let imp = this.imp();
                let is_system_processes = model.selected_item().map_or(false, |object| {
                    object
                    .downcast::<BoxedAnyObject>()
                    .unwrap()
                    .borrow::<SimpleItem>()
                    .clone()
                    .id
                    .is_none()
                });
                imp.information_button.set_sensitive(model.selected() != u32::MAX);
                imp.end_application_button.set_sensitive(model.selected() != u32::MAX && !is_system_processes);
            }),
        );

        imp.information_button
            .connect_clicked(clone!(@strong self as this => move |_| {
                let imp = this.imp();
                let selection_option = imp.selection_model.borrow()
                .selected_item()
                .map(|object| {
                    object
                    .downcast::<BoxedAnyObject>()
                    .unwrap()
                    .borrow::<SimpleItem>()
                    .clone()
                });
                if let Some(selection) = selection_option {
                    let app_dialog = ResAppDialog::new();
                    app_dialog.init(&selection);
                    app_dialog.show();
                    *imp.open_dialog.borrow_mut() = Some((selection.id.unwrap_or_default(), app_dialog));
                }
            }));

        imp.end_application_button
            .connect_clicked(clone!(@strong self as this => move |_| {
                this.end_selected_application();
            }));
    }

    pub fn setup_listener(&self) {
        let imp = self.imp();
        let model = imp.store.borrow();
        // TODO: don't use unwrap()
        *imp.apps.borrow_mut() = Apps::new().unwrap();
        imp.apps
            .borrow()
            .simple()
            .iter()
            .map(|simple_item| BoxedAnyObject::new(simple_item.clone()))
            .for_each(|item_box| model.append(&item_box));
        let statistics_update = clone!(@strong self as this => move || {
            this.refresh_apps_list()
        });
        glib::timeout_add_seconds_local(2, statistics_update);
    }

    fn get_selected_simple_item(&self) -> Option<SimpleItem> {
        self.imp()
            .selection_model
            .borrow()
            .selected_item()
            .map(|object| {
                object
                    .downcast::<BoxedAnyObject>()
                    .unwrap()
                    .borrow::<SimpleItem>()
                    .clone()
            })
    }

    fn refresh_apps_list(&self) -> Continue {
        let imp = self.imp();
        let selection = imp.selection_model.borrow();
        let mut apps = imp.apps.borrow_mut();

        // if we reuse the old ListStore, for some reason setting the
        // vadjustment later just doesn't work most of the time.
        // so we just make a new one every refresh instead :')
        // TODO: make this less hacky
        let new_store = gio::ListStore::new(BoxedAnyObject::static_type());

        // this might be very hacky, but remember the ID of the currently
        // selected item, clear the list model and repopulate it with the
        // refreshed apps and stats, then reselect the remembered app.
        // TODO: make this even less hacky
        let selected_item = self
            .get_selected_simple_item()
            .map(|simple_item| simple_item.id);
        if apps.refresh().is_ok() {
            apps.simple()
                .iter()
                .map(|simple_item| {
                    if let Some((id, dialog)) = &*imp.open_dialog.borrow() && simple_item.id.clone().unwrap_or_default().as_str() == id.as_str() {
                        dialog.set_cpu_usage(simple_item.cpu_time_ratio.unwrap_or(0.0));
                        dialog.set_memory_usage(simple_item.memory_usage);
                        dialog.set_processes_amount(simple_item.processes_amount);
                    }
                    BoxedAnyObject::new(simple_item.clone())
                })
                .for_each(|item_box| new_store.append(&item_box));
        }
        imp.sort_model.borrow().set_model(Some(&new_store));
        *imp.store.borrow_mut() = new_store;

        // find the (potentially) new index of the process that was selected
        // before the refresh and set our selection to that index
        if let Some(selected_item) = selected_item {
            let new_index = selection
                .iter::<glib::Object>()
                .unwrap()
                .position(|object| {
                    object
                        .unwrap()
                        .downcast::<BoxedAnyObject>()
                        .unwrap()
                        .borrow::<SimpleItem>()
                        .id
                        == selected_item
                })
                .map(|index| index as u32);
            if let Some(index) = new_index && index != u32::MAX {
                selection.set_selected(index);
            }
        }

        glib::Continue(true)
    }

    fn end_selected_application(&self) {
        let imp = self.imp();
        let apps = imp.apps.borrow();
        let selection_option = self
            .get_selected_simple_item()
            .and_then(|simple_item| simple_item.id);
        if let Some(selection) = selection_option && let Some(app) = apps.get_app(selection) {
            let dialog = adw::MessageDialog::builder()
            .transient_for(&MainWindow::default())
            .modal(true)
            .heading(&gettextrs::gettext!("End {}?", app.display_name()))
            .body(&gettextrs::gettext("Unsaved work might be lost."))
            .build();
            dialog.add_response("yes", &gettextrs::gettext("End Application"));
            dialog.set_response_appearance("yes", ResponseAppearance::Destructive);
            dialog.set_default_response(Some("no"));
            dialog.add_response("no", &gettextrs::gettext("Cancel"));
            dialog.set_close_response("no");
            dialog.connect_response(None, clone!(@strong app, @weak self as this => move |_, response| {
                if response == "yes" {
                    let imp = this.imp();
                    let res = app.term();
                    let processes_tried = res.len();
                    let processes_successful = res.iter().flatten().count();
                    let processes_unsuccessful = processes_tried - processes_successful;
                    if processes_tried == processes_successful {
                        imp.toast_overlay.add_toast(&Toast::new(&gettextrs::gettext!("Successfully ended {}", app.display_name())));
                    } else {
                        imp.toast_overlay.add_toast(&Toast::new(&gettextrs::ngettext!("There was a problem ending a process", "There were problems ending {} processes", processes_unsuccessful as u32, processes_unsuccessful)));
                    }
                }
            }));
            dialog.show();
        }
    }

    fn kill_selected_application(&self) {
        let imp = self.imp();
        let apps = imp.apps.borrow();
        let selection_option = self
            .get_selected_simple_item()
            .and_then(|simple_item| simple_item.id);
        if let Some(selection) = selection_option && let Some(app) = apps.get_app(selection) {
            let dialog = adw::MessageDialog::builder()
            .transient_for(&MainWindow::default())
            .modal(true)
            .heading(&gettextrs::gettext!("Kill {}?", app.display_name()))
            .body(&gettextrs::gettext("Killing an application can come with serious risks such as losing data and security implications. Use with caution."))
            .build();
            dialog.add_response("yes", &gettextrs::gettext("Kill Application"));
            dialog.set_response_appearance("yes", ResponseAppearance::Destructive);
            dialog.set_default_response(Some("no"));
            dialog.add_response("no", &gettextrs::gettext("Cancel"));
            dialog.set_close_response("no");
            dialog.connect_response(None, clone!(@strong app, @weak self as this => move |_, response| {
                if response == "yes" {
                    let imp = this.imp();
                    let res = app.kill();
                    let processes_tried = res.len();
                    let processes_successful = res.iter().flatten().count();
                    let processes_unsuccessful = processes_tried - processes_successful;
                    if processes_tried == processes_successful {
                        imp.toast_overlay.add_toast(&Toast::new(&gettextrs::gettext!("Successfully killed {}", app.display_name())));
                    } else {
                        imp.toast_overlay.add_toast(&Toast::new(&gettextrs::ngettext!("There was a problem killing a process", "There were problems killing {} processes", processes_unsuccessful as u32, processes_unsuccessful)));
                    }
                }
            }));
            dialog.show();
        }
    }

    fn halt_selected_application(&self) {
        let imp = self.imp();
        let apps = imp.apps.borrow();
        let selection_option = self
            .get_selected_simple_item()
            .and_then(|simple_item| simple_item.id);
        if let Some(selection) = selection_option && let Some(app) = apps.get_app(selection) {
            let dialog = adw::MessageDialog::builder()
            .transient_for(&MainWindow::default())
            .modal(true)
            .heading(&gettextrs::gettext!("Halt {}?", app.display_name()))
            .body(&gettextrs::gettext("Halting an application can come with serious risks such as losing data and security implications. Use with caution."))
            .build();
            dialog.add_response("yes", &gettextrs::gettext("Halt Application"));
            dialog.set_response_appearance("yes", ResponseAppearance::Destructive);
            dialog.set_default_response(Some("no"));
            dialog.add_response("no", &gettextrs::gettext("Cancel"));
            dialog.set_close_response("no");
            dialog.connect_response(None, clone!(@strong app, @weak self as this => move |_, response| {
                if response == "yes" {
                    let imp = this.imp();
                    let res = app.stop();
                    let processes_tried = res.len();
                    let processes_successful = res.iter().flatten().count();
                    let processes_unsuccessful = processes_tried - processes_successful;
                    if processes_tried == processes_successful {
                        imp.toast_overlay.add_toast(&Toast::new(&gettextrs::gettext!("Successfully halted {}", app.display_name())));
                    } else {
                        imp.toast_overlay.add_toast(&Toast::new(&gettextrs::ngettext!("There was a problem halting a process", "There were problems halting {} processes", processes_unsuccessful as u32, processes_unsuccessful)));
                    }
                }
            }));
            dialog.show();
        }
    }

    fn continue_selected_application(&self) {
        let imp = self.imp();
        let apps = imp.apps.borrow();
        let selection_option = self
            .get_selected_simple_item()
            .and_then(|simple_item| simple_item.id);
        if let Some(selection) = selection_option && let Some(app) = apps.get_app(selection) {
            let res = app.cont();
            let processes_tried = res.len();
            let processes_successful = res.iter().flatten().count();
            let processes_unsuccessful = processes_tried - processes_successful;
            if processes_tried == processes_successful {
                imp.toast_overlay.add_toast(&Toast::new(&gettextrs::gettext!("Successfully continued {}", app.display_name())));
            } else {
                imp.toast_overlay.add_toast(&Toast::new(&gettextrs::ngettext!("There was a problem continuing a process", "There were problems continuing {} processes", processes_unsuccessful as u32, processes_unsuccessful)));
            }
        }
    }
}