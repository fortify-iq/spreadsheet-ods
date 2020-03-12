use std::collections::HashMap;
use string_cache::DefaultAtom;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};


use std::io::{self, Write};
use std::fmt;
use std::marker::PhantomData;
use std::fmt::{Display, Formatter};

pub type Result = io::Result<()>;

#[derive(PartialEq)]
enum Open {
    None,
    Elem,
    Empty,
}

impl Display for Open {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Open::None => f.write_str("None")?,
            Open::Elem => f.write_str("Elem")?,
            Open::Empty => f.write_str("Empty")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Stack<'a> {
    #[cfg(feature = "check_xml")]
    stack: Vec<&'a str>,
    #[cfg(not(feature = "check_xml"))]
    stack: PhantomData<&'a str>,
}

#[cfg(feature = "check_xml")]
impl<'a> Stack<'a> {
    fn new() -> Self {
        Self {
            stack: Vec::new()
        }
    }

    fn len(&self) -> usize {
        self.stack.len()
    }

    fn push(&mut self, name: &'a str) {
        self.stack.push(name);
    }

    fn pop(&mut self) -> Option<&'a str> {
        self.stack.pop()
    }

    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(not(feature = "check_xml"))]
impl<'a> Stack<'a> {
    fn new() -> Self {
        Self {
            stack: PhantomData {}
        }
    }

    fn len(&self) -> usize {
        0
    }

    fn push(&mut self, _name: &'a str) {}

    fn pop(&mut self) -> Option<&'a str> {
        None
    }

    fn is_empty(&self) -> bool {
        true
    }
}

/// The XmlWriter himself
pub struct XmlWriter<'a, W: Write> {
    writer: Box<W>,
    stack: Stack<'a>,
    open: Open,
    /// if `true` it will indent all opening elements
    pub indent: bool,
}

impl<'a, W: Write> fmt::Debug for XmlWriter<'a, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Ok(write!(f, "XmlWriter {{ stack: {:?}, opened: {} }}", self.stack, self.open)?)
    }
}

impl<'a, W: Write> XmlWriter<'a, W> {
    /// Create a new writer, by passing an `io::Write`
    pub fn new(writer: W) -> XmlWriter<'a, W> {
        XmlWriter {
            stack: Stack::new(),
            writer: Box::new(writer),
            open: Open::None,
            indent: true,
        }
    }

    /// Write the DTD. You have to take care of the encoding
    /// on the underlying Write yourself.
    pub fn dtd(&mut self, encoding: &str) -> Result {
        self.writer.write("<?xml version=\"1.0\" encoding=\"".as_bytes())?;
        self.writer.write(encoding.as_bytes())?;
        self.writer.write("\" ?>\n".as_bytes())?;
        Ok(())
    }

    fn indent(&mut self) -> Result {
        if cfg!(feature = "indent_xml") && self.indent && !self.stack.is_empty() {
            self.write("\n")?;
            let indent = self.stack.len() * 2;
            for _ in 0..indent {
                self.write(" ")?;
            };
        }
        Ok(())
    }

    /// Write an element with inlined text (not escaped)
    pub fn elem_text(&mut self, name: &str, text: &str) -> Result {
        self.close_elem()?;
        self.indent()?;

        self.write("<")?;
        self.write(name)?;
        self.write(">")?;

        self.write(text)?;

        self.write("</")?;
        self.write(name)?;
        self.write(">")
    }

    /// Write an element with inlined text (escaped)
    pub fn elem_text_esc(&mut self, name: &str, text: &str) -> Result {
        self.close_elem()?;
        self.indent()?;

        self.write("<")?;
        self.write(name)?;
        self.write(">")?;

        self.escape(text, false)?;

        self.write("</")?;
        self.write(name)?;
        self.write(">")
    }

    /// Begin an elem, make sure name contains only allowed chars
    pub fn elem(&mut self, name: &'a str) -> Result {
        self.close_elem()?;
        self.indent()?;
        self.stack.push(name);
        self.write("<")?;
        self.open = Open::Elem;
        self.write(name)
    }

    /// Begin an empty elem
    pub fn empty(&mut self, name: &'a str) -> Result {
        self.close_elem()?;
        self.indent()?;
        self.write("<")?;
        self.open = Open::Empty;
        self.write(name)?;
        Ok(())
    }

    /// Close an elem if open, do nothing otherwise
    fn close_elem(&mut self) -> Result {
        match self.open {
            Open::None => {}
            Open::Elem => self.write(">")?,
            Open::Empty => self.write("/>")?,
        }
        self.open = Open::None;
        Ok(())
    }

    /// Write an attr, make sure name and value contain only allowed chars.
    /// For an escaping version use `attr_esc`
    pub fn attr(&mut self, name: &str, value: &str) -> Result {
        if cfg!(feature = "check_xml") && self.open == Open::None {
            panic!("Attempted to write attr to elem, when no elem was opened, stack {:?}", self.stack);
        }
        self.write(" ")?;
        self.write(name)?;
        self.write("=\"")?;
        self.write(value)?;
        self.write("\"")
    }

    /// Write an attr, make sure name contains only allowed chars
    pub fn attr_esc(&mut self, name: &str, value: &str) -> Result {
        if cfg!(feature = "check_xml") && self.open == Open::None {
            panic!("Attempted to write attr to elem, when no elem was opened, stack {:?}", self.stack);
        }
        self.write(" ")?;
        self.escape(name, true)?;
        self.write("=\"")?;
        self.escape(value, false)?;
        self.write("\"")
    }

    /// Escape identifiers or text
    fn escape(&mut self, text: &str, ident: bool) -> Result {
        for c in text.chars() {
            match c {
                '"' => self.write("&quot;")?,
                '\'' => self.write("&apos;")?,
                '&' => self.write("&amp;")?,
                '<' => self.write("&lt;")?,
                '>' => self.write("&gt;")?,
                '\\' if ident => self.write("\\\\")?,
                _ => self.write_slice(c.encode_utf8(&mut [0; 4]).as_bytes())?
            };
        }
        Ok(())
    }

    /// Write a text, doesn't escape the text.
    pub fn text(&mut self, text: &str) -> Result {
        self.close_elem()?;
        self.write(text)?;
        Ok(())
    }

    /// Write a text, escapes the text automatically
    pub fn text_esc(&mut self, text: &str) -> Result {
        self.close_elem()?;
        self.escape(text, false)?;
        Ok(())
    }

    /// End and elem
    pub fn end_elem(&mut self, name: &str) -> Result {
        self.close_elem()?;

        if cfg!(feature = "check_xml") {
            match self.stack.pop() {
                Some(test) => {
                    if name != test {
                        panic!("Attempted to close elem {} but the open was {}, stack {:?}", name, test, self.stack)
                    }
                }
                None => panic!("Attempted to close an elem, when none was open, stack {:?}", self.stack)
            }
        }

        self.write("</")?;
        self.write(name)?;
        self.write(">")?;
        Ok(())
    }

    /// Fails if there are any open elements.
    pub fn close(&mut self) -> Result {
        if cfg!(feature = "check_xml") && !self.stack.is_empty() {
            panic!("Attempted to close the xml, but there are open elements on the stack {:?}", self.stack)
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------
// -----------------------------------------------------------------------
// -----------------------------------------------------------------------
// -----------------------------------------------------------------------


pub fn start(tag: &str) -> Event {
    let b = BytesStart::borrowed_name(tag.as_bytes());
    Event::Start(b)
}

pub fn start_a<'a>(tag: &'a str, attr: &[(&'a str, &'a str)]) -> Event::<'a> {
    let mut b = BytesStart::borrowed_name(tag.as_bytes());
    for av in attr {
        b.push_attribute(*av);
    }
    Event::Start(b)
}

pub fn start_opt<'a, S: ::std::hash::BuildHasher>(tag: &'a str,
                                                  attr: Option<&'a HashMap<DefaultAtom, String, S>>)
                                                  -> Event::<'a> {
    if let Some(attr) = attr {
        start_m(tag, attr)
    } else {
        start(tag)
    }
}

pub fn start_m<'a, S: ::std::hash::BuildHasher>(tag: &'a str,
                                                attr: &'a HashMap<DefaultAtom, String, S>)
                                                -> Event::<'a> {
    let mut b = BytesStart::borrowed_name(tag.as_bytes());
    for (a, v) in attr {
        b.push_attribute((a.as_ref(), v.as_str()));
    }
    Event::Start(b)
}

pub fn start_am<'a, S: ::std::hash::BuildHasher>(tag: &'a str,
                                                 attr0: &[(&'a str, &'a str)],
                                                 attr1: Option<&'a HashMap<DefaultAtom, String, S>>)
                                                 -> Event::<'a> {
    let mut b = BytesStart::borrowed_name(tag.as_bytes());
    for av in attr0 {
        b.push_attribute(*av);
    }
    if let Some(attr1) = attr1 {
        for (a, v) in attr1 {
            b.push_attribute((a.as_ref(), v.as_str()));
        }
    }
    Event::Start(b)
}

pub fn text(text: &str) -> Event {
    Event::Text(BytesText::from_plain_str(text))
}

pub fn end(tag: &str) -> Event {
    let b = BytesEnd::borrowed(tag.as_bytes());
    Event::End(b)
}

pub fn empty(tag: &str) -> Event {
    let b = BytesStart::borrowed_name(tag.as_bytes());
    Event::Empty(b)
}

pub fn empty_a<'a>(tag: &'a str,
                   attr: &[(&'a str, &'a str)])
                   -> Event::<'a> {
    let mut b = BytesStart::borrowed_name(tag.as_bytes());
    for av in attr {
        b.push_attribute(*av);
    }
    Event::Empty(b)
}

pub fn empty_opt<'a, S: ::std::hash::BuildHasher>(tag: &'a str,
                                                  attr: Option<&'a HashMap<DefaultAtom, String, S>>)
                                                  -> Event::<'a> {
    if let Some(attr) = attr {
        empty_m(tag, attr)
    } else {
        empty(tag)
    }
}

pub fn empty_am<'a, S: ::std::hash::BuildHasher>(tag: &'a str,
                                                 attr0: &[(&'a str, &'a str)],
                                                 attr1: Option<&'a HashMap<DefaultAtom, String, S>>)
                                                 -> Event::<'a> {
    let mut b = BytesStart::borrowed_name(tag.as_bytes());
    for av in attr0 {
        b.push_attribute(*av);
    }
    if let Some(attr1) = attr1 {
        for (a, v) in attr1.iter() {
            b.push_attribute((a.as_ref(), v.as_str()));
        }
    }
    Event::Empty(b)
}

pub fn empty_m<'a, S: ::std::hash::BuildHasher>(tag: &'a str,
                                                attr: &'a HashMap<DefaultAtom, String, S>)
                                                -> Event::<'a> {
    let mut b = BytesStart::borrowed_name(tag.as_bytes());
    for (a, v) in attr.iter() {
        b.push_attribute((a.as_ref(), v.as_str()));
    }
    Event::Empty(b)
}
