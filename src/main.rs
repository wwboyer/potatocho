use potatocho::ChipEight;
use rfd::FileDialog;

fn main() {
    let mut chip_eight_state = ChipEight::new();
    let file = loop {
        let file = match FileDialog::new()
            .set_title("Select a valid Chip-8 program")
            .pick_file()
        {
            Some(file) => break file,
            None => println!("bruh")
        };
    };
    let program = loop {
        let program = match std::fs::read(file) {
            Ok(bytes) => break bytes,
            Err(err) => panic!("{:#?}", err)
        };
    };
    chip_eight_state.load_program(program);
    chip_eight_state.run();
}
