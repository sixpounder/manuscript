use crate::widgets::ManuscriptThemeSwitcher;
use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

mod imp {
    use super::*;
    // use glib::ParamSpec;
    // use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/primary_menu_button.ui")]
    pub struct ManuscriptPrimaryMenuButton {
        #[template_child]
        pub(super) primary_menu_button: TemplateChild<gtk::MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptPrimaryMenuButton {
        const NAME: &'static str = "ManuscriptPrimaryMenuButton";
        type Type = super::ManuscriptPrimaryMenuButton;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptPrimaryMenuButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }

    impl WidgetImpl for ManuscriptPrimaryMenuButton {}
}

glib::wrapper! {
    pub struct ManuscriptPrimaryMenuButton(ObjectSubclass<imp::ManuscriptPrimaryMenuButton>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptPrimaryMenuButton {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptPrimaryMenuButton {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup(&self) {
        if let Some(popover_menu) = self
            .imp()
            .primary_menu_button
            .popover()
            .and_downcast_ref::<gtk::PopoverMenu>()
        {
            popover_menu.add_child(&ManuscriptThemeSwitcher::new(), "themeswitcher");
        }
    }
}
