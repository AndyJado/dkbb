// FIXME: removed pest: use pest::{error::LineColLocation, Span};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

pub(crate) trait IntoRange {
    fn into_lsp_range(self) -> Range;
}

pub trait IntoLocation {
    fn into_lsp_location(self, uri: &Url) -> Location;
}
