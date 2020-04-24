extern crate gtk;
use super::*;
use gdk_pixbuf::*;
use gtk::*;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize};
//use std::sync::

pub struct ContentGamerule {
    pub gamemode: Box,
}

impl ContentGamerule {
    //Take the game object to initialise nbr_player and gamerule
    pub fn new(stats: std::sync::Arc<game_class::Gamerule>) -> ContentGamerule {
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
        {
            let stats = stats.clone();
            let nbr_player = nbr_player.clone();
            let button_minus = player_less.clone();
            button_minus.connect_clicked(move |_| {
                stats.set_nb_player(-1);
                nbr_player.set_label(stats.get_player().to_string().as_str());
            });
        }
        {
            let stats = stats.clone();
            let nbr_player = nbr_player.clone();
            let button_add = player_add.clone();
            button_add.connect_clicked(move |_| {
                stats.set_nb_player(1);
                nbr_player.set_label(stats.get_player().to_string().as_str());
            });
        }

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

        {
            let stats = stats.clone();
            let gamerule = gamerule.clone();
            let cpy_button1 = gamerule1.clone();
            cpy_button1.connect_clicked(move |_| {
                stats.set_gamerule(0);
                gamerule.set_label(stats.get_gamerule_name());
            });
        }
        {
            let stats = stats.clone();
            let gamerule = gamerule.clone();
            let cpy_button2 = gamerule2.clone();
            cpy_button2.connect_clicked(move |_| {
                stats.set_gamerule(1);
                gamerule.set_label(stats.get_gamerule_name());
            });
        }
        {
            let stats = stats.clone();
            let gamerule = gamerule.clone();
            let cpy_button3 = gamerule3.clone();
            cpy_button3.connect_clicked(move |_| {
                stats.set_gamerule(2);
                gamerule.set_label(stats.get_gamerule_name());
            });
        }
        {
            let stats = stats.clone();
            let gamerule = gamerule.clone();
            let cpy_button4 = gamerule4.clone();
            cpy_button4.connect_clicked(move |_| {
                stats.set_gamerule(3);
                gamerule.set_label(stats.get_gamerule_name());
            });
        }

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

        ContentGamerule { gamemode }
    }
}

pub struct MyImage {
    no_pawn: Image,
    black_pawn: Image,
    white_pawn: Image,
    forbidden_pawn: Image,
}

//impl MyImage {
//    pub fn new() -> MyImage {
//        MyImage {}
//    }
//}

#[repr(u8)]
pub enum ImageType {
    NoPawn,
    BlackPawn,
    WhitePawn,
    ForbiddenPawn,
}

pub struct EventBoxCoord {
    eventbox: AtomicPtr<EventBox>,
    image: AtomicUsize,
    x: AtomicUsize,
    y: AtomicUsize,
}

pub enum Images {
    BlackPawn = Pixbuf::get_pixbuf("src/gui/content/Black_pawn.png"),
    NoPawn = get_pixbuf("src/gui/content/No_pawn.png"),
    NoPawnPrey = get_pixbuf("src/gui/content/No_pawn_grey.png"),
    NoPawnGreen = get_pixbuf("src/gui/content/No_pawn_green.png"),
    NoPawnRed = get_pixbuf("src/gui/content/No_pawn_red.jpg"),
    NoPawnOrange = get_pixbuf("src/gui/content/No_pawn_orange.png"),
    WhitePawn = get_pixbuf("src/gui/content/White_pawn.png"),
}

impl EventBoxCoord {
    
   pub fn new(x: u8, y: u8) -> EventBoxCoord {
       let image = Image::new_from_pixbuf(Some(&content::IMAGES[0]));
       //let image = match content::IMAGES[0] {
       //    Ok(pixbuf) => Image::new_from_pixbuf(Some(&pixbuf)),
       //    Err(e) => {
       //        eprintln!("Failed to load images");
       //        process::exit(1)
       //    }
       //};
       let eventbox = EventBox::new();
       eventbox.add(&image);
       EventBoxCoord {
           eventbox,
           image,
           x,
           y,
       }
   }

   pub fn get_coord_safe(&self) -> (usize, usize) {
       (self.x.load(Ordering::SeqCst), self.y.load(Ordering::SeqCst))
   }

   pub fn delete_image(&self) {
       let eventbox = self.eventbox.load(Ordering::SeqCst);
       let image = self.image
       eventbox.remove(&);
   }
//
//    pub fn change_image(&mut self, imagetype: ImageType) {
//        self.eventbox.remove(&self.image);
//        let mut path_to_images: String = "src/gui/content/".to_owned();
//        match imagetype {
//            ImageType::NoPawn => {
//                path_to_images.push_str("No_pawn.png");
//                self.image = Image::new_from_file(path_to_images);
//            }
//            ImageType::WhitePawn => {
//                path_to_images.push_str("White_pawn.png");
//                self.image = Image::new_from_file(path_to_images);
//            }
//            ImageType::BlackPawn => {
//                path_to_images.push_str("Black_pawn.png");
//                self.image = Image::new_from_file(path_to_images);
//            }
//            ImageType::ForbiddenPawn => {
//                path_to_images.push_str("No_pawn_gray.png");
//                self.image = Image::new_from_file(path_to_images);
//            }
//        };
//        self.eventbox.add(&self.image);
//    }
//}

pub struct ContentGameplay {
    pub grid: Grid,
}

pub fn get_pixbuf(filename: &str) -> Pixbuf {
    match Pixbuf::new_from_file(filename) {
        Err(e) => {
            eprintln!("Erreur ici hihiihhihi {}", e);
            process::exit(1)
        }
        Ok(pixbuf) => pixbuf,
    }
}


impl ContentGameplay {
    pub fn new() -> ContentGameplay {
        let images: [Pixbuf; 7] = [
            get_pixbuf("src/gui/content/Black_pawn.png"),
            get_pixbuf("src/gui/content/No_pawn.png"),
            get_pixbuf("src/gui/content/No_pawn_grey.png"),
            get_pixbuf("src/gui/content/No_pawn_green.png"),
            get_pixbuf("src/gui/content/No_pawn_red.jpg"),
            get_pixbuf("src/gui/content/No_pawn_orange.png"),
            get_pixbuf("src/gui/content/White_pawn.png"),
        ];

        let grid = Grid::new();
        let size_grid: i32 = 20;

        for x in 0..19 {
            for y in 0..19 {
                let safe_eventbox = Arc::new(EventBox::new());
                
                let image = Image::new_from_pixbuf(Some(&images[0]));
                
                // let image2 = Arc::new(AtomicPtr::new(&mut image));
                // let safe_image = Arc::new(AtomicPtr::new(&mut image));

                // let safer = safe_image.clone();
                // let safe_image2 = safe_image.clone();
                
                // eventbox.add(&image);

                {
                    eventbox.connect("button_press_event", true, |_| {
                        eventbox.add(&image);
                        // eventbox.add(&(safer.load(Ordering::SeqCst)));
                        println!("test");
                        None
                    });
                }
                //                gtk_container_add(Container::from(eventbox), image);
                //                g_signal_connect(
                //                    G_OBJECT(eventbox),
                //                    "button_press_event",
                //                    G_CALLBACK(|_| {println!("bite")}),
                //                    image,
                //                );

                grid.attach(
                    &eventbox,
                    x as i32 * size_grid,
                    y as i32 * size_grid,
                    size_grid,
                    size_grid,
                );
                //                {
                //                    let eventbox = eventbox;
                                //    eventbox.eventbox.connect_button_press_event(move |_, _| {
                                //        eventbox.change_image(ImageType::ForbiddenPawn);
                                //        Inhibit(false)
                                //    });
                //                }
                //                if x == y {
                //                    eventbox.change_image(ImageType::ForbiddenPawn);
                //                }
            }
        }

        ContentGameplay { grid }
    }
}
