//! File-type detection from magic bytes (port of upstream `file_type`).
//!
//! Ref: https://en.wikipedia.org/wiki/List_of_file_signatures

/// A detected file type (upstream `FileType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileType {
    Jpeg,
    Png,
    Gif,
    Bmp,
    Qoi,
    Webp,
    Unknown,
}

struct TypeDetails {
    typ: FileType,
    /// Magic-byte signatures; `None` is a wildcard (matches any byte).
    sigs: &'static [&'static [Option<u8>]],
    exts: &'static [&'static str],
}

// Signature order matters: the first matching type wins.
#[rustfmt::skip]
const TYPE_DETAILS: &[TypeDetails] = &[
    TypeDetails {
        typ: FileType::Jpeg,
        sigs: &[
            &[Some(0xFF), Some(0xD8), Some(0xFF), Some(0xDB)],
            &[
                Some(0xFF), Some(0xD8), Some(0xFF), Some(0xE0), Some(0x00), Some(0x10),
                Some(0x4A), Some(0x46), Some(0x49), Some(0x46), Some(0x00), Some(0x01),
            ],
            &[Some(0xFF), Some(0xD8), Some(0xFF), Some(0xEE)],
            &[
                Some(0xFF), Some(0xD8), Some(0xFF), Some(0xE1), None, None,
                Some(0x45), Some(0x78), Some(0x69), Some(0x66), Some(0x00), Some(0x00),
            ],
            &[Some(0xFF), Some(0xD8), Some(0xFF), Some(0xE0)],
        ],
        exts: &[".jpg", ".jpeg", ".jfif"],
    },
    TypeDetails {
        typ: FileType::Png,
        sigs: &[&[
            Some(0x89), Some(0x50), Some(0x4E), Some(0x47),
            Some(0x0D), Some(0x0A), Some(0x1A), Some(0x0A),
        ]],
        exts: &[".png"],
    },
    TypeDetails {
        typ: FileType::Gif,
        sigs: &[
            &[Some(b'G'), Some(b'I'), Some(b'F'), Some(b'8'), Some(b'7'), Some(b'a')],
            &[Some(b'G'), Some(b'I'), Some(b'F'), Some(b'8'), Some(b'9'), Some(b'a')],
        ],
        exts: &[".gif"],
    },
    TypeDetails {
        typ: FileType::Bmp,
        sigs: &[&[Some(b'B'), Some(b'M')]],
        exts: &[".bmp"],
    },
    TypeDetails {
        typ: FileType::Qoi,
        sigs: &[&[Some(b'q'), Some(b'o'), Some(b'i'), Some(b'f')]],
        exts: &[".qoi"],
    },
    TypeDetails {
        typ: FileType::Webp,
        sigs: &[&[
            Some(0x52), Some(0x49), Some(0x46), Some(0x46), None, None, None, None,
            Some(0x57), Some(0x45), Some(0x42), Some(0x50),
        ]],
        exts: &[".webp"],
    },
];

impl FileType {
    /// Detect the file type from the leading magic bytes (upstream `detect`).
    pub(crate) fn detect(contents: &[u8]) -> FileType {
        for td in TYPE_DETAILS {
            for sig in td.sigs {
                if contents.len() >= sig.len()
                    && contents.iter().zip(sig.iter()).all(|(&f, s)| match s {
                        Some(b) => f == *b,
                        None => true, // wildcard
                    })
                {
                    return td.typ;
                }
            }
        }
        FileType::Unknown
    }

    /// Guess the file type from an extension, case-insensitively (upstream `guessFromExtension`).
    ///
    /// Byte-oriented to match upstream's `[]const u8`: a non-UTF-8 extension naturally returns
    /// `Unknown` rather than being unrepresentable.
    pub(crate) fn guess_from_extension(extension: &[u8]) -> FileType {
        for td in TYPE_DETAILS {
            for ext in td.exts {
                if extension.eq_ignore_ascii_case(ext.as_bytes()) {
                    return td.typ;
                }
            }
        }
        FileType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_each_format() {
        assert_eq!(FileType::detect(&[0xFF, 0xD8, 0xFF, 0xDB]), FileType::Jpeg);
        assert_eq!(FileType::detect(&[0xFF, 0xD8, 0xFF, 0xEE]), FileType::Jpeg);
        assert_eq!(
            FileType::detect(&[
                0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01
            ]),
            FileType::Jpeg
        );
        assert_eq!(
            FileType::detect(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            FileType::Png
        );
        assert_eq!(FileType::detect(b"GIF87a"), FileType::Gif);
        assert_eq!(FileType::detect(b"GIF89a"), FileType::Gif);
        assert_eq!(FileType::detect(b"BM......"), FileType::Bmp);
        assert_eq!(FileType::detect(b"qoif"), FileType::Qoi);
    }

    #[test]
    fn honors_wildcards() {
        // EXIF JPEG: bytes 4-5 are wildcards.
        assert_eq!(
            FileType::detect(&[
                0xFF, 0xD8, 0xFF, 0xE1, 0xAB, 0xCD, 0x45, 0x78, 0x69, 0x66, 0x00, 0x00
            ]),
            FileType::Jpeg
        );
        // WebP: RIFF + 4 wildcard size bytes + WEBP.
        assert_eq!(
            FileType::detect(&[
                0x52, 0x49, 0x46, 0x46, 0x12, 0x34, 0x56, 0x78, 0x57, 0x45, 0x42, 0x50
            ]),
            FileType::Webp
        );
        // The wildcard bytes truly don't matter: a different size still matches.
        assert_eq!(
            FileType::detect(&[
                0x52, 0x49, 0x46, 0x46, 0xFF, 0xFF, 0xFF, 0xFF, 0x57, 0x45, 0x42, 0x50
            ]),
            FileType::Webp
        );
    }

    #[test]
    fn matches_short_jpeg_signature_via_ordering() {
        // A bare `FF D8 FF E0` (4 bytes) matches the short trailing JPEG signature, not requiring
        // the longer 12-byte JFIF variant listed before it.
        assert_eq!(FileType::detect(&[0xFF, 0xD8, 0xFF, 0xE0]), FileType::Jpeg);
    }

    #[test]
    fn too_short_or_unrelated_is_unknown() {
        // Shorter than any signature (min JPEG signature is 4 bytes).
        assert_eq!(FileType::detect(&[0xFF, 0xD8]), FileType::Unknown);
        assert_eq!(FileType::detect(&[]), FileType::Unknown);
        // Unrelated bytes.
        assert_eq!(FileType::detect(b"not an image"), FileType::Unknown);
    }

    #[test]
    fn guesses_from_extension_case_insensitively() {
        assert_eq!(FileType::guess_from_extension(b".png"), FileType::Png);
        assert_eq!(FileType::guess_from_extension(b".JPG"), FileType::Jpeg);
        assert_eq!(FileType::guess_from_extension(b".Jpeg"), FileType::Jpeg);
        assert_eq!(FileType::guess_from_extension(b".jfif"), FileType::Jpeg);
        assert_eq!(FileType::guess_from_extension(b".WEBP"), FileType::Webp);
        assert_eq!(FileType::guess_from_extension(b".qoi"), FileType::Qoi);
        assert_eq!(FileType::guess_from_extension(b".txt"), FileType::Unknown);
        // A non-UTF-8 extension simply doesn't match.
        assert_eq!(
            FileType::guess_from_extension(&[0xFF, 0xFE]),
            FileType::Unknown
        );
    }
}
