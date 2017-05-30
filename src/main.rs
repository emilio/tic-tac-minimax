/*
 * Copyright (C) 2017 Emilio Cobos Álvarez <emilio@crisal.io>
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
 */

extern crate gtk;

mod minimax;
mod state;

use minimax::MiniMaxTree;
use state::CheckBox;

use gtk::{BoxExt, Cast, EntryExt, WidgetExt, WindowExt, ContainerExt, ButtonExt};

use std::cell::RefCell;
use std::rc::Rc;

struct App {
    tree: RefCell<MiniMaxTree>,

    window: gtk::Window,
    restart_button: gtk::Button,
    grid: gtk::Grid,
    depth_input: gtk::Entry,
}

impl App {
    fn init(app: Rc<Self>) {
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 10 /* px */);
        box_.pack_start(&app.grid, /* expand = */ true, /* fill = */ true, 0);
        box_.pack_start(&app.restart_button, /* expand = */ true, /* fill = */ true, 0);
        box_.pack_start(&app.depth_input, /* expand = */ true, /* fill = */ true, 0);
        app.window.add(&box_);

        app.depth_input.set_placeholder_text("Max depth");

        app.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            gtk::Inhibit(false)
        });

        {
            let app_clone = app.clone();
            app.restart_button.connect_clicked(move |_| {
                // TODO(randomize?).
                *app_clone.tree.borrow_mut() = MiniMaxTree::new(CheckBox::X);
                app_clone.update_grid();
            });
        }

        for x in 0..3 {
            for y in 0..3 {
                let app = app.clone();
                let button = app.grid.get_child_at(x, y)
                    .expect("Grid should be 3x3")
                    .downcast::<gtk::Button>()
                    .expect("No button? Pshaw!");

                let x = x as usize;
                let y = y as usize;

                button.connect_clicked(move |_| {
                    app.handle_click(x, y);
                });
            }
        }

        app.window.show_all();
    }

    fn handle_click(&self, x: usize, y: usize) {
        use std::cmp;

        {
            let mut tree = self.tree.borrow_mut();
            if tree.choose(x, y).is_err() {
                // TODO(emilio): Suggest an error? meh.
                return;
            }

            let max_depth = self.depth_input.get_text().and_then(|s| {
                s.parse::<usize>().ok()
            }).unwrap_or(4);

            let max_depth = cmp::max(max_depth, 1);

            // Now play as the opponent.
            if let Some(index) = tree.find_move_index(max_depth) {
                tree.choose_with_index(index);
            }
        }

        self.update_grid();
    }

    fn update_grid(&self) {
        let tree = self.tree.borrow();
        let state = tree.state();

        for x in 0..3 {
            for y in 0..3 {
                let button = self.grid.get_child_at(x, y)
                    .expect("Grid should be 3x3")
                    .downcast::<gtk::Button>()
                    .expect("No button? Pshaw!");
                let label = match state.get(x as usize, y as usize) {
                    CheckBox::Empty => " ",
                    CheckBox::X => "X",
                    CheckBox::O => "O",
                };
                button.set_label(label);
            }
        }
    }

    fn build_grid() -> gtk::Grid {
        let grid = gtk::Grid::new();
        for x in 0..3 {
            for y in 0..3 {
                let button = gtk::Button::new();
                grid.attach(&button, x, y, 1, 1);
            }
        }
        grid
    }
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Tic tac toe");
    window.set_default_size(350, 70);

    let button = gtk::Button::new_with_label("Restart");

    let app = Rc::new(App {
        tree: RefCell::new(minimax::MiniMaxTree::new(CheckBox::X)),

        window: window,
        restart_button: button,
        grid: App::build_grid(),
        depth_input: gtk::Entry::new(),
    });

    App::init(app);

    gtk::main();
}
