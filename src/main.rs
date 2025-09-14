mod audio;
use audio::decoder::Decoder;

fn main() {
    let dec = Decoder::open("file.mp3");
}
