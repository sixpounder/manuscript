/* window.rs
 *
 * Copyright 2023 Andrea
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::widgets::ManuscriptTextEditor;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/window.ui")]
    pub struct ManuscriptWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub editor: TemplateChild<ManuscriptTextEditor>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptWindow {
        const NAME: &'static str = "ManuscriptWindow";
        type Type = super::ManuscriptWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptWindow {}
    impl WidgetImpl for ManuscriptWindow {}
    impl WindowImpl for ManuscriptWindow {}
    impl ApplicationWindowImpl for ManuscriptWindow {}
    impl AdwApplicationWindowImpl for ManuscriptWindow {}
}

glib::wrapper! {
    pub struct ManuscriptWindow(ObjectSubclass<imp::ManuscriptWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,        @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::new(&[("application", application)])
    }
}
