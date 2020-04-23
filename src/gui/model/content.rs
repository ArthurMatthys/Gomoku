extern crate gtk;
use super::*;
use gtk::*;

pub struct Content {
    pub gamemode: Box,
    pub nbr_player: Label,
    pub gamerule: Label,
    pub player_less: Button,
    pub player_add: Button,
    pub gamerule1: Button,
    pub gamerule2: Button,
    pub gamerule3: Button,
    pub gamerule4: Button,
}

impl Content {
    //Take the game object to initialise nbr_player and gamerule
    pub fn new(stats: &game_class::Gamerule) -> Content {
        let gamemode: Box = Box::new(Orientation::Horizontal, 0);

        let choice: Box = Box::new(Orientation::Vertical, 100);
        let display: Box = Box::new(Orientation::Vertical, 100);

        let player_info: Box = Box::new(Orientation::Horizontal, 0);
        let message_player: Label = Label::new("Number of player: ");
        let nbr_player: Label = Label::new(stats.get_player().to_string().as_str());

        player_info.set_halign(Align::Center);
        message_player.set_halign(Align::Start);
        nbr_player.set_halign(Align::Start);

        player_info.pack_start(&message_player, false, false, 5);
        player_info.pack_start(&nbr_player, true, true, 5);

        let gamerule_info: Box = Box::new(Orientation::Horizontal, 0);
        let message_gamerule: Label = Label::new("Gamerule chosen: ");
        let gamerule: Label = Label::new(game_class::TYPEOFPARTY[stats.get_gamerule()]);

        gamerule_info.set_halign(Align::Center);
        message_gamerule.set_halign(Align::Start);
        gamerule.set_halign(Align::Start);

        gamerule_info.pack_start(&message_gamerule, false, false, 5);
        gamerule_info.pack_start(&gamerule, true, true, 5);

        display.pack_start(&player_info, true, false, 0);
        display.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        display.pack_start(&gamerule_info, true, false, 0);

        let player_choice_info: Box = Box::new(Orientation::Horizontal, 0);
        let player_less: Button = Button::new_with_label("\t\t\t-\t\t\t");
        let player_add: Button = Button::new_with_label("\t\t\t+\t\t\t");

        player_choice_info.set_halign(Align::Center);
        player_less.set_halign(Align::Start);
        player_add.set_halign(Align::Start);

        player_choice_info.pack_start(&player_less, true, true, 10);
        player_choice_info.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);
        player_choice_info.pack_start(&player_add, true, true, 0);

        let gamerule_choice_info: Box = Box::new(Orientation::Horizontal, 0);
        let gamerule1: Button = Button::new_with_label(game_class::TYPEOFPARTY[0]);
        let gamerule2: Button = Button::new_with_label(game_class::TYPEOFPARTY[1]);
        let gamerule3: Button = Button::new_with_label(game_class::TYPEOFPARTY[2]);
        let gamerule4: Button = Button::new_with_label(game_class::TYPEOFPARTY[3]);

        // Create the alignement
        gamerule_choice_info.set_halign(Align::Center);
        gamerule1.set_halign(Align::Start);
        gamerule2.set_halign(Align::Start);
        gamerule3.set_halign(Align::Start);
        gamerule4.set_halign(Align::Start);

        // Plan the gamerule choice box
        gamerule_choice_info.pack_start(&gamerule1, true, true, 0);
        gamerule_choice_info.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);
        gamerule_choice_info.pack_start(&gamerule2, true, true, 0);
        gamerule_choice_info.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);
        gamerule_choice_info.pack_start(&gamerule3, true, true, 0);
        gamerule_choice_info.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);
        gamerule_choice_info.pack_start(&gamerule4, true, true, 0);

        // Plan the choice box
        choice.pack_start(&player_choice_info, true, true, 0);
        choice.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        choice.pack_start(&gamerule_choice_info, true, true, 0);

        // Plan the gamemode
        gamemode.pack_start(&choice, true, false, 0);
        gamemode.pack_start(&Separator::new(Orientation::Vertical), false, false, 0);
        gamemode.pack_start(&display, true, false, 0);

        Content {
            gamemode,
            nbr_player,
            gamerule,
            player_less,
            player_add,
            gamerule1,
            gamerule2,
            gamerule3,
            gamerule4,
        }
    }
}

#[repr(u8)]
pub enum ImageType {
    NoPawn,
    BlackPawn,
    WhitePawn,
    ForbiddenPawn,
}

pub struct EventBoxCoord {
    pub eventbox: EventBox,
    image: Image,
    x: u8,
    y: u8,
}

impl EventBoxCoord {
    pub fn new(x: u8, y: u8) -> EventBoxCoord {
        let image = Image::new_from_file("src/gui/content/No_pawn.png");
        let eventbox = EventBox::new();
        eventbox.add(&image);
        EventBoxCoord {
            eventbox,
            image,
            x,
            y,
        }
    }

    pub fn change_image(&mut self, imagetype: ImageType) {
        self.eventbox.remove(&self.image);
        let mut path_to_images: String = "src/gui/content/".to_owned();
        match imagetype {
            ImageType::NoPawn => {
                path_to_images.push_str("No_pawn.png");
                self.image = Image::new_from_file(path_to_images);
            }
            ImageType::WhitePawn => {
                path_to_images.push_str("White_pawn.png");
                self.image = Image::new_from_file(path_to_images);
            }
            ImageType::BlackPawn => {
                path_to_images.push_str("Black_pawn.png");
                self.image = Image::new_from_file(path_to_images);
            }
            ImageType::ForbiddenPawn => {
                path_to_images.push_str("No_pawn_gray.png");
                self.image = Image::new_from_file(path_to_images);
            }
        };
        self.eventbox.add(&self.image);
    }
}

pub struct ContentGameplay {
    pub grid: Grid,
}

impl ContentGameplay {
    pub fn new() -> ContentGameplay {
        let grid = Grid::new();
        let size_grid: i32 = 20;

        for x in 0..19 {
            for y in 0..19 {
                let mut eventbox = EventBoxCoord::new(x, y);
                grid.attach(
                    &eventbox.eventbox,
                    x as i32 * size_grid,
                    y as i32 * size_grid,
                    size_grid,
                    size_grid,
                );
                if x == y {
                    eventbox.change_image(ImageType::ForbiddenPawn);
                }
            }
        }

        ContentGameplay { grid }
    }
}
