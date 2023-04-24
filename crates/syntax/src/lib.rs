pub mod ast;
pub mod dyna_nodes;
pub mod parse;
pub mod reparsing;
pub mod syntax_error;
pub mod syntax_node;

pub mod helpers {
    pub fn env_read() -> String {
        let p = std::env::args().skip(1).next().unwrap();
        std::fs::read_to_string(p).unwrap()
    }

    pub struct Now(std::time::Instant);

    impl Now {
        pub fn new() -> Self {
            use std::time::Instant;
            Now(Instant::now())
        }
        pub fn now(&mut self, is: &str) {
            let dur = self.0.elapsed();
            self.0 = std::time::Instant::now();
            eprintln!("{is} takes {:#?}", dur);
        }
    }
}
