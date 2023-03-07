use pest::{error::LineColLocation, Span};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

pub(crate) trait IntoRange {
    fn into_lsp_range(self) -> Range;
}

impl IntoRange for LineColLocation {
    fn into_lsp_range(self) -> Range {
        match self {
            LineColLocation::Pos((line, col)) => {
                let pos = Position::new(line as u32, col as u32);
                Range::new(pos, pos)
            }
            LineColLocation::Span((start_line, start_col), (end_line, end_col)) => Range::new(
                Position::new(start_line as u32, start_col as u32),
                Position::new(end_line as u32, end_col as u32),
            ),
        }
    }
}

impl IntoRange for Span<'_> {
    fn into_lsp_range(self) -> Range {
        let start = self.start_pos().line_col();
        let end = self.end_pos().line_col();
        LineColLocation::Span((start.0 - 1, start.1 - 1), (end.0 - 1, end.1 - 1)).into_lsp_range()
    }
}

pub trait IntoLocation {
    fn into_lsp_location(self, uri: &Url) -> Location;
}

impl IntoLocation for Span<'_> {
    fn into_lsp_location(self, uri: &Url) -> Location {
        Location::new(uri.clone(), self.into_lsp_range())
    }
}
