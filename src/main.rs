#[macro_use]
extern crate lazy_static;

extern crate gdk;
extern crate gio;
extern crate gtk;

mod parser;
mod window;

use gio::prelude::*;

fn main() {
    let application = gtk::Application::new(
        "com.ethanmcdonough.calculator",
        gio::ApplicationFlags::empty(),
    ).expect("Application initialization failure");

    application.connect_startup(|app| {
        use self::window::Calculator;
        let calculator = Calculator::new(app);
        calculator.show();
    });

    application.connect_activate(|_| ());

    application.run(&std::env::args().collect::<Vec<_>>());
}
