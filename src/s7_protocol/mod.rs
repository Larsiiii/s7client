pub(crate) mod header;
pub(crate) mod negotiate;
pub(crate) mod read_area;
pub(crate) mod types;
pub(crate) mod write_area;

pub(crate) trait S7Protocol {
    fn build() -> Self;
}
