pub mod dyna_psr {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "dyna.pest"]
    pub struct TryParser;
}
