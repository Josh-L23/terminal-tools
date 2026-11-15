use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Markdown,
    Pdf,
    Docx,
}

pub const ALL_FORMATS: &[Format] = &[Format::Markdown, Format::Pdf, Format::Docx];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conversion {
    pub from: Format,
    pub to: Format,
    pub backend: &'static str,
}

pub const CONVERSIONS: &[Conversion] = &[
    Conversion {
        from: Format::Markdown,
        to: Format::Pdf,
        backend: "WeasyPrint",
    },
    Conversion {
        from: Format::Docx,
        to: Format::Pdf,
        backend: "LibreOffice",
    },
    Conversion {
        from: Format::Docx,
        to: Format::Markdown,
        backend: "Mammoth",
    },
    Conversion {
        from: Format::Pdf,
        to: Format::Markdown,
        backend: "PyMuPDF",
    },
];

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Format::Markdown => "md",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
        }
    }

    pub fn extension(self) -> &'static str {
        self.name()
    }

    pub fn label(self) -> &'static str {
        match self {
            Format::Markdown => "Markdown (.md)",
            Format::Pdf => "PDF (.pdf)",
            Format::Docx => "Word document (.docx)",
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()?.to_str()?.parse().ok()
    }

    pub fn targets(self) -> Vec<Format> {
        CONVERSIONS
            .iter()
            .filter(|conversion| conversion.from == self)
            .map(|conversion| conversion.to)
            .collect()
    }

    pub fn can_convert_to(self, target: Format) -> bool {
        CONVERSIONS
            .iter()
            .any(|conversion| conversion.from == self && conversion.to == target)
    }
}

impl FromStr for Format {
    type Err = FormatParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(Format::Markdown),
            "pdf" => Ok(Format::Pdf),
            "docx" | "word" => Ok(Format::Docx),
            _ => Err(FormatParseError(value.to_string())),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[derive(Debug, Clone)]
pub struct FormatParseError(pub String);

impl fmt::Display for FormatParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported format '{}'; expected one of: md, pdf, docx",
            self.0
        )
    }
}

impl std::error::Error for FormatParseError {}

pub fn parse_format(value: &str) -> Result<Format, FormatParseError> {
    value.parse()
}

pub fn default_output_path(input: &Path, target: Format) -> PathBuf {
    let mut output = input.to_path_buf();
    output.set_extension(target.extension());
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_extensions_and_aliases() {
        assert_eq!(
            Format::from_path(Path::new("notes.MD")),
            Some(Format::Markdown)
        );
        assert_eq!(
            Format::from_path(Path::new("report.docx")),
            Some(Format::Docx)
        );
        assert_eq!(Format::from_path(Path::new("paper.pdf")), Some(Format::Pdf));
        assert_eq!(Format::from_path(Path::new("archive.zip")), None);
        assert_eq!("markdown".parse::<Format>().unwrap(), Format::Markdown);
        assert_eq!("WORD".parse::<Format>().unwrap(), Format::Docx);
        assert!("doc".parse::<Format>().is_err());
    }

    #[test]
    fn exposes_supported_targets() {
        assert_eq!(Format::Markdown.targets(), vec![Format::Pdf]);
        assert_eq!(Format::Docx.targets(), vec![Format::Pdf, Format::Markdown]);
        assert_eq!(Format::Pdf.targets(), vec![Format::Markdown]);
        assert!(!Format::Pdf.can_convert_to(Format::Docx));
    }

    #[test]
    fn builds_default_output_paths() {
        assert_eq!(
            default_output_path(Path::new("/tmp/notes.md"), Format::Pdf),
            PathBuf::from("/tmp/notes.pdf")
        );
    }

    #[test]
    fn conversions_are_unique() {
        for (index, conversion) in CONVERSIONS.iter().enumerate() {
            assert!(
                !CONVERSIONS[index + 1..]
                    .iter()
                    .any(|other| { other.from == conversion.from && other.to == conversion.to }),
                "duplicate conversion: {:?} -> {:?}",
                conversion.from,
                conversion.to
            );
        }
    }
}
