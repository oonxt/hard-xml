//! Strong typed xml, based on xmlparser.
//!
//! ## Quick Start
//!
//! ```rust
//! use std::borrow::Cow;
//! use hard_xml::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(attr = "attr1")]
//!     attr1: Cow<'a, str>,
//!     #[xml(attr = "attr2")]
//!     attr2: Option<Cow<'a, str>>,
//!     #[xml(child = "child")]
//!     child: Vec<Child<'a>>,
//! }
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "child")]
//! struct Child<'a> {
//!     #[xml(text)]
//!     text: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { attr1: "val".into(), attr2: None, child: vec![] }).to_string().unwrap(),
//!     r#"<parent attr1="val"/>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent attr1="val" attr2="val"><child></child></parent>"#).unwrap(),
//!     Parent { attr1: "val".into(), attr2: Some("val".into()), child: vec![Child { text: "".into() }] }
//! );
//! ```
//!
//! ## Attributes
//!
//! ### `#[xml(strict(...))]`
//!
//! Opt-in to stricter input handling.
//!
//! ```rust
//! #[derive(hard_xml::XmlWrite, hard_xml::XmlRead, PartialEq, Debug)]
//! #[xml(tag = "ex")]
//! #[xml(strict(unknown_attribute, unknown_element))]
//! struct Ex {
//!     // ...
//! }
//! ```
//!
//! #### Strict Options
//!
//! ##### `strict(unknown_attribute)`
//!
//! Fail to parse if an unknown attribute is encountered.
//!
//! ```rust
//! # use hard_xml::XmlRead;
//! #[derive(Debug, hard_xml::XmlRead)]
//! #[xml(tag = "ex")]
//! #[xml(strict(unknown_attribute))]
//! struct Ex {
//! }
//!
//! assert_eq!(
//!     Ex::from_str("<ex foo='bar'/>").unwrap_err().to_string(),
//!     r#"unknown field "foo" in element "Ex""#);
//! ```
//!
//! ##### `strict(unknown_element)`
//!
//!  Fail to parse if an unknown child element is encountered.
//!
//! ```rust
//! # use hard_xml::XmlRead;
//! #[derive(Debug, hard_xml::XmlRead)]
//! #[xml(tag = "ex")]
//! #[xml(strict(unknown_element))]
//! struct Ex {
//! }
//!
//! assert_eq!(
//!     Ex::from_str("<ex><unknown/></ex>").unwrap_err().to_string(),
//!     r#"unknown field "unknown" in element "Ex""#);
//! ```
//!
//! ### `#[xml(tag = "")]`
//!
//! Specifies the xml tag of a struct or an enum variant.
//!
//! ```rust
//! # use hard_xml::{XmlRead, XmlWrite};
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent {}
//!
//! assert_eq!(
//!     (Parent {}).to_string().unwrap(),
//!     r#"<parent/>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent/>"#).unwrap(),
//!     Parent {}
//! );
//! ```
//!
//! ```rust
//! # use hard_xml::{XmlRead, XmlWrite};
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag1")]
//! struct Tag1 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag2")]
//! struct Tag2 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! enum Tag {
//!     #[xml(tag = "tag1")]
//!     Tag1(Tag1),
//!     #[xml(tag = "tag2")]
//!     Tag2(Tag2),
//! }
//!
//! assert_eq!(
//!     (Tag::Tag1(Tag1 {})).to_string().unwrap(),
//!     r#"<tag1/>"#
//! );
//!
//! assert_eq!(
//!     Tag::from_str(r#"<tag2></tag2>"#).unwrap(),
//!     Tag::Tag2(Tag2 {})
//! );
//! ```
//!
//! ### `#[xml(attr = "")]`
//!
//! Specifies that a struct field is attribute. Support
//! `Cow<str>`, `Option<Cow<str>>`, `T` and `Option<T>`
//! where `T: FromStr + Display`.
//!
//! ```rust
//! use hard_xml::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent {
//!     #[xml(attr = "attr")]
//!     attr: usize
//! }
//!
//! assert_eq!(
//!     (Parent { attr: 42 }).to_string().unwrap(),
//!     r#"<parent attr="42"/>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent attr="48"></parent>"#).unwrap(),
//!     Parent { attr: 48 }
//! );
//! ```
//!
//! ### `#[xml(attr = "" with = "")]`
//!
//! Parse and serialize the field with the module specified as an argument. That module must provide the following interface given that the fields is of type `T`.
//!
//! ```ignore
//! mod some_module_name {
//!     fn from_xml(mode: &str) -> hard_xml::XmlResult<T>;
//!     fn to_xml(mode: &T) -> hard_xml::XmlResult<impl std::convert::AsRef<str>>;
//! }
//! ```
//!
//! #### Example
//!
//! ```rust
//! # use hard_xml::{XmlRead, XmlWrite};
//! #
//! mod parse_drive_mode {
//!     pub fn from_xml(mode: &str) -> hard_xml::XmlResult<bool> {
//!         match mode {
//!             "agile" => Ok(false),
//!             "fast" => Ok(true),
//!             _ => Err(hard_xml::XmlError::FromStr(
//!                 format!("Expected mode to be agile or fast, got {mode:?}").into())),
//!         }
//!     }
//!
//!     pub fn to_xml(mode: &bool) -> hard_xml::XmlResult<&'static str> {
//!         match mode {
//!             false => Ok("agile"),
//!             true => Ok("fast"),
//!         }
//!     }
//! }
//!
//! #[derive(XmlRead, XmlWrite, PartialEq, Debug)]
//! #[xml(tag = "spaceship")]
//! struct Spaceship {
//!     #[xml(attr = "drive_mode", with = "parse_drive_mode")]
//!     drive_mode: bool,
//! }
//!
//! assert_eq!(
//!     Spaceship::from_str(r#"<spaceship drive_mode="agile"/>"#).unwrap(),
//!     Spaceship {
//!         drive_mode: false,
//!     }
//! );
//!
//! // match with a reversed string in from_xml of withmod.
//! assert_eq!(
//!     Spaceship {
//!         drive_mode: true,
//!     }.to_string().unwrap(),
//!     r#"<spaceship drive_mode="fast"/>"#,
//! );
//! ```
//!
//! ### `#[xml(child = "")]`
//!
//! Specifies that a struct field is a child element. Support
//! `T`, `Option<T>`, `Vec<T>` where `T: XmlRead + XmlWrite`.
//!
//! ```rust
//! use hard_xml::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag1")]
//! struct Tag1 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag2")]
//! struct Tag2 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "tag3")]
//! struct Tag3 {}
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! enum Tag12 {
//!     #[xml(tag = "tag1")]
//!     Tag1(Tag1),
//!     #[xml(tag = "tag2")]
//!     Tag2(Tag2),
//! }
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent {
//!     #[xml(child = "tag3")]
//!     tag3: Vec<Tag3>,
//!     #[xml(child = "tag1", child = "tag2")]
//!     tag12: Option<Tag12>
//! }
//!
//! assert_eq!(
//!     (Parent { tag3: vec![Tag3 {}], tag12: None }).to_string().unwrap(),
//!     r#"<parent><tag3/></parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent><tag2></tag2></parent>"#).unwrap(),
//!     Parent { tag3: vec![], tag12: Some(Tag12::Tag2(Tag2 {})) }
//! );
//! ```
//!
//! ### `#[xml(text)]`
//!
//! Specifies that a struct field is text content.
//! Support `Cow<str>`, `Vec<Cow<str>>`, `Option<Cow<str>>`,
//! `T`, `Vec<T>`, `Option<T>` where `T: FromStr + Display`.
//!
//! ```rust
//! use std::borrow::Cow;
//! use hard_xml::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(text)]
//!     content: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { content: "content".into() }).to_string().unwrap(),
//!     r#"<parent>content</parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent></parent>"#).unwrap(),
//!     Parent { content: "".into() }
//! );
//! ```
//!
//! ### `#[xml(flatten_text = "")]`
//!
//! Specifies that a struct field is child text element.
//! Support `Cow<str>`, `Vec<Cow<str>>`, `Option<Cow<str>>`,
//! `T`, `Vec<T>`, `Option<T>` where `T: FromStr + Display`.
//!
//! ```rust
//! use std::borrow::Cow;
//! use hard_xml::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(flatten_text = "child")]
//!     content: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { content: "content".into() }).to_string().unwrap(),
//!     r#"<parent><child>content</child></parent>"#
//! );
//!
//! assert_eq!(
//!     Parent::from_str(r#"<parent><child></child></parent>"#).unwrap(),
//!     Parent { content: "".into() }
//! );
//! ```
//!
//! ### `#[xml(cdata)]`
//!
//! Specifies a CDATA text. Should be used together with `text` or `flatten_text`.
//!
//! > `#[xml(cdata)]` only changes the behavior of writing,
//! > text field without `#[xml(cdata)]` can still works with cdata tag.
//!
//! ```rust
//! use std::borrow::Cow;
//! use hard_xml::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(text, cdata)]
//!     content: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { content: "content".into() }).to_string().unwrap(),
//!     r#"<parent><![CDATA[content]]></parent>"#
//! );
//! ```
//!
//! ```rust
//! use std::borrow::Cow;
//! use hard_xml::{XmlRead, XmlWrite};
//!
//! #[derive(XmlWrite, XmlRead, PartialEq, Debug)]
//! #[xml(tag = "parent")]
//! struct Parent<'a> {
//!     #[xml(flatten_text = "code", cdata)]
//!     content: Cow<'a, str>,
//! }
//!
//! assert_eq!(
//!     (Parent { content: r#"hello("deities!");"#.into() }).to_string().unwrap(),
//!     r#"<parent><code><![CDATA[hello("deities!");]]></code></parent>"#
//! );
//! ```
//!
//! ### `#[xml(default)]`
//!
//! Use `Default::default()` if the value is not present when reading.
//!
//! ```rust
//! use std::borrow::Cow;
//! use hard_xml::XmlRead;
//!
//! #[derive(XmlRead, PartialEq, Debug)]
//! #[xml(tag = "root")]
//! struct Root {
//!     #[xml(default, attr = "attr")]
//!     attr: bool,
//! }
//!
//! assert_eq!(
//!     Root::from_str(r#"<root/>"#).unwrap(),
//!     Root { attr: false }
//! );
//!
//! assert_eq!(
//!     Root::from_str(r#"<root attr="1"/>"#).unwrap(),
//!     Root { attr: true }
//! );
//! ```

#[cfg(feature = "log")]
mod log;
#[cfg(not(feature = "log"))]
mod noop_log;

#[cfg(feature = "log")]
#[doc(hidden)]
pub mod lib {
    pub use log;
}

mod xml_error;
mod xml_escape;
mod xml_read;
mod xml_reader;
mod xml_unescape;
mod xml_write;
mod xml_writer;

pub use self::xml_error::{XmlError, XmlResult};
pub use self::xml_read::{XmlRead, XmlReadOwned};
pub use self::xml_reader::XmlReader;
pub use self::xml_write::XmlWrite;
pub use self::xml_writer::XmlWriter;

pub use hard_xml_derive::{XmlRead, XmlWrite};

pub use xmlparser;

pub mod utils {
    pub use super::xml_escape::xml_escape;
    pub use super::xml_unescape::xml_unescape;
}
