/// Represents the xml version num: 1.0 or 1.1
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum XmlVersion {
    Ver10,
    Ver11,
}
