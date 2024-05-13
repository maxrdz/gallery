// This file is part of Albums.
//
// Copyright (c) 2024 Max Rodriguez
// All rights reserved.
//
// Albums is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Albums is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Albums.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::config::APP_ID;
use crate::library::library_list_model::LibraryListModel;
use crate::window::AlbumsApplicationWindow;
use adw::gtk;
use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::{gio, glib};
use gettextrs::gettext;
use glib::g_debug;
use libadwaita as adw;
use std::cell::OnceCell;

#[derive(Debug, glib::Properties)]
#[properties(wrapper_type = super::AlbumsApplication)]
pub struct AlbumsApplication {
    pub(super) gsettings: gio::Settings,
    /// Core GListModel for enumerating photo and video album files.
    /// Initialized after the application window is presented.
    #[property(get, set)]
    pub library_list_model: OnceCell<LibraryListModel>,
}

impl Default for AlbumsApplication {
    fn default() -> Self {
        Self {
            gsettings: gio::Settings::new(APP_ID),
            library_list_model: OnceCell::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for AlbumsApplication {
    const NAME: &'static str = "AlbumsApplication";
    type Type = super::AlbumsApplication;
    type ParentType = adw::Application;
}

#[glib::derived_properties]
impl ObjectImpl for AlbumsApplication {
    fn constructed(&self) {
        g_debug!("AlbumsApplication", "Reached constructed()");
        self.parent_constructed();
        let obj = self.obj();

        obj.setup_gactions();
        obj.set_accels_for_action("win.settings", &["<primary>comma"]);

        obj.set_accels_for_action("app.system-theme", &["<primary><shift>s"]);
        obj.set_accels_for_action("app.light-theme", &["<primary><shift>l"]);
        obj.set_accels_for_action("app.dark-theme", &["<primary><shift>d"]);

        obj.set_accels_for_action("app.about", &["<primary>a"]);
        obj.set_accels_for_action("app.quit", &["<primary>q"]);
    }
}

impl ApplicationImpl for AlbumsApplication {
    fn activate(&self) {
        let application = self.obj();

        // The activate() callback also notifies us when the user tries
        // to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        let window = if let Some(window) = application.active_window() {
            g_debug!("AlbumsApplication", "Application has an active window present!");
            window
        } else {
            g_debug!(
                "AlbumsApplication",
                "No active window found; Creating a new window."
            );
            let window = AlbumsApplicationWindow::new(&application);
            window.upcast()
        };

        window.set_title(Some(&gettext("Albums")));
        window.present();

        // Setup our own CSS provider from gresource
        let gdk_screen: gdk::Display = gdk::Display::default().unwrap();
        let new_css_provider: gtk::CssProvider = gtk::CssProvider::new();
        new_css_provider.load_from_resource("/com/maxrdz/Albums/style.css");

        gtk::style_context_add_provider_for_display(
            &gdk_screen,
            &new_css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

impl GtkApplicationImpl for AlbumsApplication {}
impl AdwApplicationImpl for AlbumsApplication {}
