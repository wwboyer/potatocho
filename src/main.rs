use potatocho::ChipEight;
use rfd::FileDialog;

fn find_sdl_gl_driver() -> Option<u32> {
    for (i, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(i as u32);
        }
    }
    None
}
fn main() {
    let mut chip_eight_state = ChipEight::new();

    let sdl_context = match sdl2::init() {
        Ok(sdl) => {
            println!("Created sdl context!");
            sdl
        }
        Err(e) => panic!("Error creating sdl context: {:?}", e),
    };

    let video_subsystem = match sdl_context.video() {
        Ok(video) => {
            println!("Created sdl videocontext!");
            video
        }
        Err(e) => panic!("Error creating sdl videocontext: {:?}", e),
    };

    let window = match video_subsystem
        .window("PotatOcho", 1280, 640)
        .opengl()
        .position_centered()
        .build()
    {
        Ok(window) => {
            println!("Created sdl window!");
            window
        }
        Err(e) => panic!("Error creating sdl window: {:?}", e.to_string()),
    };

    let canvas = match window
        .into_canvas()
        .index(match find_sdl_gl_driver() {
            Some(i) => i,
            None => panic!("Unable to find compatible OpenGL driver!"),
        })
        .present_vsync()
        .build()
    {
        Ok(canvas) => {
            println!("Created sdl canvas!");
            canvas
        }
        Err(e) => panic!("Error creating sdl canvas: {:?}", e.to_string()),
    };

    let file = loop {
        match FileDialog::new()
            .set_title("Select a valid Chip-8 program")
            .pick_file()
        {
            Some(file) => break file,
            None => println!("bruh"),
        };
    };
    let program = loop {
        match std::fs::read(file) {
            Ok(bytes) => break bytes,
            Err(err) => panic!("{:#?}", err),
        };
    };
    chip_eight_state.load_program(program);
    chip_eight_state.run(canvas, sdl_context);
}
