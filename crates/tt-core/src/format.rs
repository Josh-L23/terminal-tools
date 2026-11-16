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
    Png,
    Jpg,
    Webp,
    Bmp,
    Gif,
    Tiff,
}

pub const ALL_FORMATS: &[Format] = &[
    Format::Markdown,
    Format::Pdf,
    Format::Docx,
    Format::Png,
    Format::Jpg,
    Format::Webp,
    Format::Bmp,
    Format::Gif,
    Format::Tiff,
];

pub const IMAGE_FORMATS: &[Format] = &[
    Format::Png,
    Format::Jpg,
    Format::Webp,
    Format::Bmp,
    Format::Gif,
    Format::Tiff,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conversion {
    pub from: Format,
    pub to: Format,
    pub backend: &'static str,
}

pub const CONVERSIONS: &[Conversion] = &[
    // Documents
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
    // Images
    Conversion {
        from: Format::Png,
        to: Format::Jpg,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Png,
        to: Format::Webp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Png,
        to: Format::Bmp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Png,
        to: Format::Gif,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Png,
        to: Format::Tiff,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Jpg,
        to: Format::Png,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Jpg,
        to: Format::Webp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Jpg,
        to: Format::Bmp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Jpg,
        to: Format::Gif,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Jpg,
        to: Format::Tiff,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Webp,
        to: Format::Png,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Webp,
        to: Format::Jpg,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Webp,
        to: Format::Bmp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Webp,
        to: Format::Gif,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Webp,
        to: Format::Tiff,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Bmp,
        to: Format::Png,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Bmp,
        to: Format::Jpg,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Bmp,
        to: Format::Webp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Bmp,
        to: Format::Gif,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Bmp,
        to: Format::Tiff,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Gif,
        to: Format::Png,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Gif,
        to: Format::Jpg,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Gif,
        to: Format::Webp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Gif,
        to: Format::Bmp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Gif,
        to: Format::Tiff,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Tiff,
        to: Format::Png,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Tiff,
        to: Format::Jpg,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Tiff,
        to: Format::Webp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Tiff,
        to: Format::Bmp,
        backend: "Pillow",
    },
    Conversion {
        from: Format::Tiff,
        to: Format::Gif,
        backend: "Pillow",
    },
];

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Format::Markdown => "md",
            Format::Pdf => "pdf",
            Format::Docx => "docx",
            Format::Png => "png",
            Format::Jpg => "jpg",
            Format::Webp => "webp",
            Format::Bmp => "bmp",
            Format::Gif => "gif",
            Format::Tiff => "tiff",
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
            Format::Png => "PNG image (.png)",
            Format::Jpg => "JPEG image (.jpg)",
            Format::Webp => "WebP image (.webp)",
            Format::Bmp => "BMP image (.bmp)",
            Format::Gif => "GIF image (.gif)",
            Format::Tiff => "TIFF image (.tiff)",
        }
    }

    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            Format::Markdown => &["markdown"],
            Format::Docx => &["word"],
            Format::Jpg => &["jpeg"],
            Format::Tiff => &["tif"],
            _ => &[],
        }
    }

    pub fn is_image(self) -> bool {
        IMAGE_FORMATS.contains(&self)
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

    pub fn supported_names() -> String {
        ALL_FORMATS
            .iter()
            .map(|format| format.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn supported_extension_list() -> String {
        let mut extensions: Vec<String> = ALL_FORMATS
            .iter()
            .map(|format| format!(".{}", format.name()))
            .collect();
        // Include common alternate extensions users may type.
        extensions.push(".jpeg".to_string());
        extensions.push(".tif".to_string());
        extensions.sort();
        extensions.dedup();
        extensions.join(", ")
    }
}

impl FromStr for Format {
    type Err = FormatParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(Format::Markdown),
            "pdf" => Ok(Format::Pdf),
            "docx" | "word" => Ok(Format::Docx),
            "png" => Ok(Format::Png),
            "jpg" | "jpeg" => Ok(Format::Jpg),
            "webp" => Ok(Format::Webp),
            "bmp" => Ok(Format::Bmp),
            "gif" => Ok(Format::Gif),
            "tiff" | "tif" => Ok(Format::Tiff),
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
            "unsupported format '{}'; expected one of: {}",
            self.0,
            Format::supported_names()
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
        assert_eq!(
            Format::from_path(Path::new("photo.JPEG")),
            Some(Format::Jpg)
        );
        assert_eq!(Format::from_path(Path::new("scan.TIF")), Some(Format::Tiff));
        assert_eq!(Format::from_path(Path::new("archive.zip")), None);
        assert_eq!("markdown".parse::<Format>().unwrap(), Format::Markdown);
        assert_eq!("WORD".parse::<Format>().unwrap(), Format::Docx);
        assert_eq!("jpeg".parse::<Format>().unwrap(), Format::Jpg);
        assert_eq!("tif".parse::<Format>().unwrap(), Format::Tiff);
        assert!("doc".parse::<Format>().is_err());
    }

    #[test]
    fn exposes_supported_targets() {
        assert_eq!(Format::Markdown.targets(), vec![Format::Pdf]);
        assert_eq!(Format::Docx.targets(), vec![Format::Pdf, Format::Markdown]);
        assert_eq!(Format::Pdf.targets(), vec![Format::Markdown]);
        assert!(!Format::Pdf.can_convert_to(Format::Docx));

        let png_targets = Format::Png.targets();
        assert_eq!(png_targets.len(), IMAGE_FORMATS.len() - 1);
        assert!(png_targets.contains(&Format::Jpg));
        assert!(png_targets.contains(&Format::Webp));
        assert!(!png_targets.contains(&Format::Png));
        assert!(Format::Jpg.can_convert_to(Format::Png));
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
