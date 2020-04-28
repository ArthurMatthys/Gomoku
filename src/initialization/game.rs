use  sdl2::render::Canvas;

// TYPE OF PARTY
pub enum TypeOfParty {
    Unset,
    Standard,
    Pro,
    Swap,
    Swap2
}

pub struct Game {
    // DESIGN
    events: sdl2::EventPump,
    canvas: Canvas<sdl2::video::Window>,
    
    // GAME
    pub nb_of_player: usize,
    pub board: [Option<bool>; 361],

    type_of_party: TypeOfParty,
}

impl Game {

    pub fn new(title: &'static str, width: u32, height: u32, nb_of_player: usize, type_of_party: TypeOfParty) -> Result<Game, String> {
        // Initialize SDL2
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;

        // Create the window
        let window = video.window(title, width, height)
            .position_centered().opengl()
            .build()
            .map_err(|e| e.to_string())?;
        
        let events = sdl_context.event_pump()?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Game {
            events: events,
            canvas: canvas,
            nb_of_player: nb_of_player,
            board: [None; 361],
            type_of_party: type_of_party,
        })
    }

    pub fn get_canvas(&self) -> &Canvas<sdl2::video::Window> {
        &self.canvas
    }
}