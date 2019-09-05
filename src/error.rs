//! Provides an error type for this crate.

use std::convert::From;

/// Our main error type.
#[derive(Debug)]
pub enum Error {
    /// An error from quick_xml.
    XmlError(::quick_xml::Error),

    /// An UTF-8 conversion error.
    Utf8Error(::std::str::Utf8Error),

    /// An I/O error, from std::io.
    IoError(::std::io::Error),

    /// An error which is returned when the end of the document was reached prematurely.
    EndOfDocument,

    /// An error which is returned when an element is closed when it shouldn't be
    InvalidElementClosed,

    /// An error which is returned when an elemet's name contains more than one colon
    InvalidElement,

    /// An error which is returned when a comment is to be parsed by minidom
    #[cfg(not(comments))]
    CommentsDisabled,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::XmlError(e) => write!(fmt, "XML error: {}", e),
            Error::Utf8Error(e) => write!(fmt, "UTF-8 error: {}", e),
            Error::IoError(e) => write!(fmt, "IO error: {}", e),
            Error::EndOfDocument => write!(fmt, "the end of the document has been reached prematurely"),
            Error::InvalidElementClosed => write!(fmt, "the XML is invalid, an element was wrongly closed"),
            Error::InvalidElement => write!(fmt, "the XML element is invalid"),
            #[cfg(not(comments))]
            Error::CommentsDisabled => write!(fmt, "a comment has been found even though comments are disabled by feature"),
        }
    }
}

impl From<::quick_xml::Error> for Error {
    fn from(err: ::quick_xml::Error) -> Error {
        Error::XmlError(err)
    }
}

impl From<::std::str::Utf8Error> for Error {
    fn from(err: ::std::str::Utf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error::IoError(err)
    }
}

/// Our simplified Result type.
pub type Result<T> = ::std::result::Result<T, Error>;
