extern crate gtk;
use gtk::*;

pub struct HeaderGameplay {
    pub container: HeaderBar,
    pub hint: Button,
}

impl HeaderGameplay {
    pub fn new() -> HeaderGameplay {
        // Creates the main header bar container widget.
        let container = HeaderBar::new();

        // Sets the text to display in the title section of the header bar.
        container.set_title("Gomoku");
        // Enable the window controls within this headerbar.
        container.set_show_close_button(true);
        // Enable the window controls within this headerbar.

        // Create the hit and heal buttons.
        let hint = Button::new_with_label("Hint");

        // Add the corresponding style classes to those buttons.
        hint.get_style_context()
            .map(|c| c.add_class("suggested-action"));

        // THen add them to the header bar.
        container.pack_start(&hint);

        HeaderGameplay { container, hint }
    }
}

pub struct HeaderGamerule {
    pub container: HeaderBar,
    pub play: Button,
}

impl HeaderGamerule {
    pub fn new() -> HeaderGamerule {
        // Creates the main header bar container widget.
        let container = HeaderBar::new();

        // Sets the text to display in the title section of the header bar.
        container.set_title("Gomoku");
        // Enable the window controls within this headerbar.
        container.set_show_close_button(true);
        // Enable the window controls within this headerbar.

        // Create the hit and heal buttons.
        let play = Button::new_with_label("Play");

        // Add the corresponding style classes to those buttons.
        play.get_style_context()
            .map(|c| c.add_class("suggested-action"));

        // THen add them to the header bar.
        container.pack_start(&play);

        HeaderGamerule { container, play }
    }
}
