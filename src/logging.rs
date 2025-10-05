use env_logger::{self, Builder, Env, Target};
use std::fs::File;
use std::io::BufWriter;

pub fn init() {
    // only log for debug builds
    if cfg!(debug_assertions) {
        let file = File::create("./dutar.log").unwrap();
        let writer = BufWriter::new(file);

        let mut builder = Builder::from_env(Env::default().default_filter_or("error,dutar=debug"));
        builder.target(Target::Pipe(Box::new(writer)));
        builder.init();
    }
}
