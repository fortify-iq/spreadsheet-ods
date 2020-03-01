pub use crate::ods::error::OdsError;
pub use crate::ods::read_ods::read_ods;
pub use crate::ods::write_ods::write_ods;
pub use crate::ods::write_ods::write_ods_clean;

mod xml_util;
mod temp_zip;

mod error {
    use std::fmt::{Display, Formatter};

    #[derive(Debug)]
    pub enum OdsError {
        Ods(String),
        Io(std::io::Error),
        Zip(zip::result::ZipError),
        Xml(quick_xml::Error),
        ParseInt(std::num::ParseIntError),
        ParseFloat(std::num::ParseFloatError),
        Chrono(chrono::format::ParseError),
        Duration(time::OutOfRangeError),
        SystemTime(std::time::SystemTimeError),
    }

    impl Display for OdsError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            match self {
                OdsError::Ods(e) => write!(f, "Ods {}", e)?,
                OdsError::Io(e) => write!(f, "IO {}", e)?,
                OdsError::Zip(e) => write!(f, "Zip {}", e)?,
                OdsError::Xml(e) => write!(f, "Xml {}", e)?,
                OdsError::ParseInt(e) => write!(f, "ParseInt {}", e)?,
                OdsError::ParseFloat(e) => write!(f, "ParseFloat {}", e)?,
                OdsError::Chrono(e) => write!(f, "Chrono {}", e)?,
                OdsError::Duration(e) => write!(f, "Duration {}", e)?,
                OdsError::SystemTime(e) => write!(f, "SystemTime {}", e)?,
            }

            Ok(())
        }
    }

    impl std::error::Error for OdsError {
        fn cause(&self) -> Option<&dyn std::error::Error> {
            match self {
                OdsError::Ods(_) => None,
                OdsError::Io(e) => Some(e),
                OdsError::Zip(e) => Some(e),
                OdsError::Xml(e) => Some(e),
                OdsError::ParseInt(e) => Some(e),
                OdsError::ParseFloat(e) => Some(e),
                OdsError::Chrono(e) => Some(e),
                OdsError::Duration(e) => Some(e),
                OdsError::SystemTime(e) => Some(e),
            }
        }
    }

    impl From<time::OutOfRangeError> for OdsError {
        fn from(err: time::OutOfRangeError) -> OdsError {
            OdsError::Duration(err)
        }
    }

    impl From<std::io::Error> for OdsError {
        fn from(err: std::io::Error) -> OdsError {
            OdsError::Io(err)
        }
    }

    impl From<zip::result::ZipError> for OdsError {
        fn from(err: zip::result::ZipError) -> OdsError {
            OdsError::Zip(err)
        }
    }

    impl From<quick_xml::Error> for OdsError {
        fn from(err: quick_xml::Error) -> OdsError {
            OdsError::Xml(err)
        }
    }

    impl From<std::num::ParseIntError> for OdsError {
        fn from(err: std::num::ParseIntError) -> OdsError {
            OdsError::ParseInt(err)
        }
    }

    impl From<std::num::ParseFloatError> for OdsError {
        fn from(err: std::num::ParseFloatError) -> OdsError {
            OdsError::ParseFloat(err)
        }
    }

    impl From<chrono::format::ParseError> for OdsError {
        fn from(err: chrono::format::ParseError) -> OdsError {
            OdsError::Chrono(err)
        }
    }

    impl From<std::time::SystemTimeError> for OdsError {
        fn from(err: std::time::SystemTimeError) -> OdsError {
            OdsError::SystemTime(err)
        }
    }
}


mod read_ods {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    use chrono::{Duration, NaiveDate, NaiveDateTime};
    use quick_xml::events::{BytesStart, Event};
    use quick_xml::events::attributes::Attribute;
    use zip::read::ZipFile;

    use crate::{Family, Origin, FormatPart, FormatType, SCell, SColumn, Sheet, Style, Value, ValueFormat, ValueType, WorkBook, FontDecl};
    use crate::ods::error::OdsError;

    //const DUMP_XML: bool = false;

    // Reads an ODS-file.
    pub fn read_ods<P: AsRef<Path>>(path: P) -> Result<WorkBook, OdsError> {
        let file = File::open(path.as_ref())?;
        // ods is a zip-archive, we read content.xml
        let mut zip = zip::ZipArchive::new(file)?;
        let mut zip_file = zip.by_name("content.xml")?;

        let mut book = read_content(&mut zip_file)?;

        book.file = Some(path.as_ref().to_path_buf());

        Ok(book)
    }

    fn read_content(zip_file: &mut ZipFile) -> Result<WorkBook, OdsError> {
        // xml parser
        let mut xml
            = quick_xml::Reader::from_reader(BufReader::new(zip_file));
        xml.trim_text(true);

        let mut buf = Vec::new();

        let mut book = WorkBook::new();
        let mut sheet = Sheet::new();

        // Separate counter for table-columns
        let mut tcol: usize = 0;

        // Cell position
        let mut row: usize = 0;
        let mut col: usize = 0;

        // Rows can be repeated. In reality only empty ones ever are.
        let mut row_repeat: usize = 1;
        // Row style.
        let mut row_style: Option<String> = None;

        loop {
            let event = xml.read_event(&mut buf)?;
            //if DUMP_XML { log::debug!("{:?}", event); }
            match event {
                Event::Start(xml_tag)
                if xml_tag.name() == b"table:table" => {
                    read_table(&xml, xml_tag, &mut sheet)?;
                }
                Event::End(xml_tag)
                if xml_tag.name() == b"table:table" => {
                    row = 0;
                    col = 0;
                    book.push_sheet(sheet);
                    sheet = Sheet::new();
                }

                Event::Empty(xml_tag)
                if xml_tag.name() == b"table:table-column" => {
                    tcol = read_table_column(&mut xml, &xml_tag, tcol, &mut sheet)?;
                }

                Event::Start(xml_tag)
                if xml_tag.name() == b"table:table-row" => {
                    row_repeat = read_table_row(&mut xml, xml_tag, &mut row_style)?;
                }
                Event::End(xml_tag)
                if xml_tag.name() == b"table:table-row" => {
                    if let Some(style) = row_style {
                        for r in row..row + row_repeat {
                            sheet.set_row_style(r, style.clone());
                        }
                    }
                    row_style = None;

                    row += row_repeat;
                    col = 0;
                    row_repeat = 1;
                }

                Event::Start(xml_tag)
                if xml_tag.name() == b"office:font-face-decls" =>
                    read_fonts(&mut book, &mut xml, b"office:font-face-decls")?,

                Event::Start(xml_tag)
                if xml_tag.name() == b"office:automatic-styles" =>
                    read_styles(&mut book, &mut xml, b"office:automatic-styles")?,

                Event::Empty(xml_tag)
                if xml_tag.name() == b"table:table-cell" =>
                    col = read_empty_table_cell(&mut xml, xml_tag, col)?,

                Event::Start(xml_tag)
                if xml_tag.name() == b"table:table-cell" =>
                    col = read_table_cell(&mut xml, xml_tag, row, col, &mut sheet)?,

                Event::Eof => {
                    break;
                }
                _ => {}
            }

            buf.clear();
        }

        Ok(book)
    }

    fn read_table(xml: &quick_xml::Reader<BufReader<&mut ZipFile>>,
                  xml_tag: BytesStart,
                  sheet: &mut Sheet) -> Result<(), OdsError> {
        for attr in xml_tag.attributes().with_checks(false) {
            match attr? {
                attr if attr.key == b"table:name" => {
                    let v = attr.unescape_and_decode_value(xml)?;
                    sheet.set_name(v);
                }
                attr if attr.key == b"table:style-name" => {
                    let v = attr.unescape_and_decode_value(xml)?;
                    sheet.set_style(v);
                }
                _ => { /* ignore other attr */ }
            }
        }

        Ok(())
    }

    fn read_table_row(xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                      xml_tag: BytesStart,
                      row_style: &mut Option<String>) -> Result<usize, OdsError>
    {
        let mut row_repeat: usize = 1;

        for attr in xml_tag.attributes().with_checks(false) {
            match attr? {
                attr if attr.key == b"table:number-rows-repeated" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    row_repeat = v.parse::<usize>()?;
                }
                attr if attr.key == b"table:style-name" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    *row_style = Some(v);
                }
                _ => { /* ignore other */ }
            }
        }

        Ok(row_repeat)
    }

    fn read_table_column(xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                         xml_tag: &BytesStart,
                         mut tcol: usize,
                         sheet: &mut Sheet) -> Result<usize, OdsError> {
        let mut column = SColumn::new();
        let mut repeat: usize = 1;

        for attr in xml_tag.attributes().with_checks(false) {
            match attr? {
                attr if attr.key == b"table:style-name" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    column.style = Some(v.to_string());
                }
                attr if attr.key == b"table:number-columns-repeated" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    repeat = v.parse()?;
                }
                attr if attr.key == b"table:default-cell-style-name" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    column.def_cell_style = Some(v.to_string());
                }
                _ => {}
            }
        }

        while repeat > 0 {
            sheet.columns.insert(tcol, column.clone());
            tcol += 1;
            repeat -= 1;
        }

        Ok(tcol)
    }

    fn read_table_cell(xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                       xml_tag: BytesStart,
                       row: usize,
                       mut col: usize,
                       sheet: &mut Sheet) -> Result<usize, OdsError> {

        // The current cell.
        let mut cell: SCell = SCell::new();
        // Columns can be repeated, not only empty ones.
        let mut cell_repeat: usize = 1;
        // Decoded type.
        let mut value_type: Option<ValueType> = None;
        // Basic cell value here.
        let mut cell_value: Option<String> = None;
        // Content of the table-cell tag.
        let mut cell_content: Option<String> = None;
        // Currency
        let mut cell_currency: Option<String> = None;

        for attr in xml_tag.attributes().with_checks(false) {
            match attr? {
                attr if attr.key == b"table:number-columns-repeated" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    cell_repeat = v.parse::<usize>()?;
                }

                attr if attr.key == b"office:value-type" =>
                    value_type = Some(decode_value_type(&xml, attr)?),

                attr if attr.key == b"office:date-value" =>
                    cell_value = Some(attr.unescape_and_decode_value(&xml)?),
                attr if attr.key == b"office:time-value" =>
                    cell_value = Some(attr.unescape_and_decode_value(&xml)?),
                attr if attr.key == b"office:value" =>
                    cell_value = Some(attr.unescape_and_decode_value(&xml)?),
                attr if attr.key == b"office:boolean-value" =>
                    cell_value = Some(attr.unescape_and_decode_value(&xml)?),

                attr if attr.key == b"office:currency" =>
                    cell_currency = Some(attr.unescape_and_decode_value(&xml)?),

                attr if attr.key == b"table:formula" =>
                    cell.formula = Some(attr.unescape_and_decode_value(&xml)?),
                attr if attr.key == b"table:style-name" =>
                    cell.style = Some(attr.unescape_and_decode_value(&xml)?),

                _ => {}
            }
        }

        let mut buf = Vec::new();
        loop {
            let evt = xml.read_event(&mut buf)?;
            //if DUMP_XML { log::debug!(" style {:?}", evt); }
            match evt {
                Event::Text(xml_tag) => {
                    // Not every cell type has a value attribute, some take
                    // their value from the string representation.
                    cell_content = Some(xml_tag.unescape_and_decode(&xml)?);
                }

                Event::End(xml_tag)
                if xml_tag.name() == b"table:table-cell" => {
                    cell.value = parse_value(value_type,
                                             cell_value,
                                             cell_content,
                                             cell_currency)?;

                    while cell_repeat > 1 {
                        sheet.add_cell(row, col, cell.clone());
                        col += 1;
                        cell_repeat -= 1;
                    }
                    sheet.add_cell(row, col, cell);
                    col += 1;

                    break;
                }

                Event::Eof => {
                    break;
                }

                _ => {}
            }

            buf.clear();
        }

        Ok(col)
    }

    fn read_empty_table_cell(xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                             xml_tag: BytesStart,
                             mut col: usize) -> Result<usize, OdsError> {
        // Simple empty cell. Advance and don't store anything.
        if xml_tag.attributes().count() == 0 {
            col += 1;
        }
        for attr in xml_tag.attributes().with_checks(false) {
            match attr? {
                attr if attr.key == b"table:number-columns-repeated" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    col += v.parse::<usize>()?;
                }
                _ => { /* should be nothing else of interest here */ }
            }
        }

        Ok(col)
    }

    fn parse_value(value_type: Option<ValueType>,
                   cell_value: Option<String>,
                   cell_content: Option<String>,
                   cell_currency: Option<String>) -> Result<Option<Value>, OdsError> {
        if let Some(value_type) = value_type {
            match value_type {
                ValueType::Text => {
                    Ok(cell_content.map(Value::Text))
                }
                ValueType::Number => {
                    if let Some(cell_value) = cell_value {
                        let f = cell_value.parse::<f64>()?;
                        Ok(Some(Value::Number(f)))
                    } else {
                        Err(OdsError::Ods(String::from("Cell of type number, but no value!")))
                    }
                }
                ValueType::DateTime => {
                    if let Some(cell_value) = cell_value {
                        let dt =
                            if cell_value.len() == 10 {
                                NaiveDate::parse_from_str(cell_value.as_str(), "%Y-%m-%d")?.and_hms(0, 0, 0)
                            } else {
                                NaiveDateTime::parse_from_str(cell_value.as_str(), "%Y-%m-%dT%H:%M:%S%.f")?
                            };

                        Ok(Some(Value::DateTime(dt)))
                    } else {
                        Err(OdsError::Ods(String::from("Cell of type datetime, but no value!")))
                    }
                }
                ValueType::TimeDuration => {
                    if let Some(mut cell_value) = cell_value {
                        let mut hour: u32 = 0;
                        let mut have_hour = false;
                        let mut min: u32 = 0;
                        let mut have_min = false;
                        let mut sec: u32 = 0;
                        let mut have_sec = false;
                        let mut nanos: u32 = 0;
                        let mut nanos_digits: u8 = 0;

                        for c in cell_value.drain(..) {
                            match c {
                                'P' | 'T' => {}
                                '0'..='9' => {
                                    if !have_hour {
                                        hour = hour * 10 + (c as u32 - '0' as u32);
                                    } else if !have_min {
                                        min = min * 10 + (c as u32 - '0' as u32);
                                    } else if !have_sec {
                                        sec = sec * 10 + (c as u32 - '0' as u32);
                                    } else {
                                        nanos = nanos * 10 + (c as u32 - '0' as u32);
                                        nanos_digits += 1;
                                    }
                                }
                                'H' => have_hour = true,
                                'M' => have_min = true,
                                '.' => have_sec = true,
                                'S' => {}
                                _ => {}
                            }
                        }
                        // unseen nano digits
                        while nanos_digits < 9 {
                            nanos *= 10;
                            nanos_digits += 1;
                        }

                        let secs: u64 = hour as u64 * 3600 + min as u64 * 60 + sec as u64;
                        let dur = Duration::from_std(std::time::Duration::new(secs, nanos))?;

                        Ok(Some(Value::TimeDuration(dur)))
                    } else {
                        Err(OdsError::Ods(String::from("Cell of type time-duration, but no value!")))
                    }
                }
                ValueType::Boolean => {
                    if let Some(cell_value) = cell_value {
                        Ok(Some(Value::Boolean(&cell_value == "true")))
                    } else {
                        Err(OdsError::Ods(String::from("Cell of type boolean, but no value!")))
                    }
                }
                ValueType::Currency => {
                    if let Some(cell_value) = cell_value {
                        let f = cell_value.parse::<f64>()?;
                        if let Some(cell_currency) = cell_currency {
                            Ok(Some(Value::Currency(cell_currency, f)))
                        } else {
                            Err(OdsError::Ods(String::from("Cell of type currency, but no currency name!")))
                        }
                    } else {
                        Err(OdsError::Ods(String::from("Cell of type currency, but no value!")))
                    }
                }
                ValueType::Percentage => {
                    if let Some(cell_value) = cell_value {
                        let f = cell_value.parse::<f64>()?;
                        Ok(Some(Value::Percentage(f)))
                    } else {
                        Err(OdsError::Ods(String::from("Cell of type percentage, but no value!")))
                    }
                }
            }
        } else {
            Err(OdsError::Ods(String::from("Cell with no value-type!")))
        }
    }

    fn decode_value_type(xml: &quick_xml::Reader<BufReader<&mut ZipFile>>,
                         attr: Attribute) -> Result<ValueType, OdsError> {
        match attr.unescape_and_decode_value(&xml)?.as_ref() {
            "string" => Ok(ValueType::Text),
            "float" => Ok(ValueType::Number),
            "percentage" => Ok(ValueType::Percentage),
            "date" => Ok(ValueType::DateTime),
            "time" => Ok(ValueType::TimeDuration),
            "boolean" => Ok(ValueType::Boolean),
            "currency" => Ok(ValueType::Currency),
            other => Err(OdsError::Ods(format!("Unknown cell-type {}", other)))
        }
    }

    fn read_fonts(book: &mut WorkBook,
                  xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                  end_tag: &[u8]) -> Result<(), OdsError> {
        let mut buf = Vec::new();

        let mut font: FontDecl = FontDecl::new_origin(Origin::Content);

        loop {
            let evt = xml.read_event(&mut buf)?;
            //if DUMP_XML { log::debug!(" style {:?}", evt); }
            match evt {
                Event::Start(ref xml_tag)
                | Event::Empty(ref xml_tag) => {
                    match xml_tag.name() {
                        b"style:font-face" => {
                            for attr in xml_tag.attributes().with_checks(false) {
                                match attr? {
                                    attr if attr.key == b"style:name" => {
                                        let v = attr.unescape_and_decode_value(&xml)?;
                                        font.set_name(v);
                                    }
                                    attr => {
                                        let k = xml.decode(&attr.key)?;
                                        let v = attr.unescape_and_decode_value(&xml)?;
                                        font.set_prp(k, v);
                                    }
                                }
                            }

                            book.add_font(font);
                            font = FontDecl::new_origin(Origin::Content);
                        }
                        _ => {}
                    }
                }

                Event::End(ref e) => {
                    if e.name() == end_tag {
                        break;
                    }
                }

                Event::Eof => {
                    break;
                }
                _ => {}
            }

            buf.clear();
        }

        Ok(())
    }

    fn read_styles(book: &mut WorkBook,
                   xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                   end_tag: &[u8]) -> Result<(), OdsError> {
        let mut buf = Vec::new();

        let mut style: Style = Style::new_origin(Origin::Content);
        let mut value_style = ValueFormat::new_origin(Origin::Content);
        // Styles with content information are stored before completion.
        let mut value_style_part = None;

        loop {
            let evt = xml.read_event(&mut buf)?;
            //if DUMP_XML { log::debug!(" style {:?}", evt); }
            match evt {
                Event::Start(ref xml_tag)
                | Event::Empty(ref xml_tag) => {
                    match xml_tag.name() {
                        b"style:style" => {
                            read_style(xml, xml_tag, &mut style)?;

                            // In case of an empty xml-tag we are done here.
                            if let Event::Empty(_) = evt {
                                book.add_style(style);
                                style = Style::new_origin(Origin::Content);
                            }
                        }

                        b"style:table-properties" =>
                            copy_style_properties(&mut style, &Style::set_table_prp, xml, xml_tag)?,
                        b"style:table-column-properties" =>
                            copy_style_properties(&mut style, &Style::set_table_col_prp, xml, xml_tag)?,
                        b"style:table-row-properties" =>
                            copy_style_properties(&mut style, &Style::set_table_row_prp, xml, xml_tag)?,
                        b"style:table-cell-properties" =>
                            copy_style_properties(&mut style, &Style::set_table_cell_prp, xml, xml_tag)?,
                        b"style:text-properties" =>
                            copy_style_properties(&mut style, &Style::set_text_prp, xml, xml_tag)?,
                        b"style:paragraph-properties" =>
                            copy_style_properties(&mut style, &Style::set_paragraph_prp, xml, xml_tag)?,

                        b"number:boolean-style" =>
                            read_value_style(ValueType::Boolean, &mut value_style, xml, xml_tag)?,
                        b"number:date-style" =>
                            read_value_style(ValueType::DateTime, &mut value_style, xml, xml_tag)?,
                        b"number:time-style" =>
                            read_value_style(ValueType::TimeDuration, &mut value_style, xml, xml_tag)?,
                        b"number:number-style" =>
                            read_value_style(ValueType::Number, &mut value_style, xml, xml_tag)?,
                        b"number:currency-style" =>
                            read_value_style(ValueType::Currency, &mut value_style, xml, xml_tag)?,
                        b"number:percentage-style" =>
                            read_value_style(ValueType::Percentage, &mut value_style, xml, xml_tag)?,
                        b"number:text-style" =>
                            read_value_style(ValueType::Text, &mut value_style, xml, xml_tag)?,

                        b"number:boolean" =>
                            push_value_style_part(&mut value_style, FormatType::Boolean, xml, xml_tag)?,
                        b"number:number" =>
                            push_value_style_part(&mut value_style, FormatType::Number, xml, xml_tag)?,
                        b"number:scientific-number" =>
                            push_value_style_part(&mut value_style, FormatType::Scientific, xml, xml_tag)?,
                        b"number:day" =>
                            push_value_style_part(&mut value_style, FormatType::Day, xml, xml_tag)?,
                        b"number:month" =>
                            push_value_style_part(&mut value_style, FormatType::Month, xml, xml_tag)?,
                        b"number:year" =>
                            push_value_style_part(&mut value_style, FormatType::Year, xml, xml_tag)?,
                        b"number:era" =>
                            push_value_style_part(&mut value_style, FormatType::Era, xml, xml_tag)?,
                        b"number:day-of-week" =>
                            push_value_style_part(&mut value_style, FormatType::DayOfWeek, xml, xml_tag)?,
                        b"number:week-of-year" =>
                            push_value_style_part(&mut value_style, FormatType::WeekOfYear, xml, xml_tag)?,
                        b"number:quarter" =>
                            push_value_style_part(&mut value_style, FormatType::Quarter, xml, xml_tag)?,
                        b"number:hours" =>
                            push_value_style_part(&mut value_style, FormatType::Hours, xml, xml_tag)?,
                        b"number:minutes" =>
                            push_value_style_part(&mut value_style, FormatType::Minutes, xml, xml_tag)?,
                        b"number:seconds" =>
                            push_value_style_part(&mut value_style, FormatType::Seconds, xml, xml_tag)?,
                        b"number:fraction" =>
                            push_value_style_part(&mut value_style, FormatType::Fraction, xml, xml_tag)?,
                        b"number:am-pm" =>
                            push_value_style_part(&mut value_style, FormatType::AmPm, xml, xml_tag)?,
                        b"number:embedded-text" =>
                            push_value_style_part(&mut value_style, FormatType::EmbeddedText, xml, xml_tag)?,
                        b"number:text-content" =>
                            push_value_style_part(&mut value_style, FormatType::TextContent, xml, xml_tag)?,
                        b"style:text" =>
                            push_value_style_part(&mut value_style, FormatType::Day, xml, xml_tag)?,
                        b"style:map" =>
                            push_value_style_part(&mut value_style, FormatType::StyleMap, xml, xml_tag)?,
                        b"number:currency-symbol" => {
                            value_style_part = Some(read_part(xml, xml_tag, FormatType::CurrencySymbol)?);

                            // Empty-Tag. Finish here.
                            if let Event::Empty(_) = evt {
                                if let Some(part) = value_style_part {
                                    value_style.push_part(part);
                                }
                                value_style_part = None;
                            }
                        }
                        b"number:text" => {
                            value_style_part = Some(read_part(xml, xml_tag, FormatType::Text)?);

                            // Empty-Tag. Finish here.
                            if let Event::Empty(_) = evt {
                                if let Some(part) = value_style_part {
                                    value_style.push_part(part);
                                }
                                value_style_part = None;
                            }
                        }

                        _ => {}
                    }
                }

                Event::Text(ref e) => {
                    if let Some(part) = &mut value_style_part {
                        part.content = Some(e.unescape_and_decode(&xml)?);
                    }
                }

                Event::End(ref e) => {
                    if e.name() == end_tag {
                        break;
                    }

                    match e.name() {
                        b"style:style" => {
                            book.add_style(style);
                            style = Style::new_origin(Origin::Content);
                        }
                        b"number:boolean-style" |
                        b"number:date-style" |
                        b"number:time-style" |
                        b"number:number-style" |
                        b"number:currency-style" |
                        b"number:percentage-style" |
                        b"number:text-style" => {
                            book.add_format(value_style);
                            value_style = ValueFormat::new_origin(Origin::Content);
                        }
                        b"number:currency-symbol" | b"number:text" => {
                            if let Some(part) = value_style_part {
                                value_style.push_part(part);
                            }
                            value_style_part = None;
                        }

                        _ => {}
                    }
                }
                Event::Eof => {
                    break;
                }
                _ => {}
            }

            buf.clear();
        }

        Ok(())
    }

    fn read_value_style(value_type: ValueType,
                        value_style: &mut ValueFormat,
                        xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                        xml_tag: &BytesStart) -> Result<(), OdsError> {
        value_style.v_type = value_type;

        for attr in xml_tag.attributes().with_checks(false) {
            match attr? {
                attr if attr.key == b"style:name" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    value_style.set_name(v);
                }
                attr => {
                    let k = xml.decode(&attr.key)?;
                    let v = attr.unescape_and_decode_value(&xml)?;
                    value_style.set_prp(k, v);
                }
            }
        }

        Ok(())
    }

    fn push_value_style_part(value_style: &mut ValueFormat,
                             part_type: FormatType,
                             xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                             xml_tag: &BytesStart) -> Result<(), OdsError> {
        value_style.push_part(read_part(xml, xml_tag, part_type)?);

        Ok(())
    }

    fn read_part(xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                 xml_tag: &BytesStart,
                 part_type: FormatType) -> Result<FormatPart, OdsError> {
        let mut part = FormatPart::new(part_type);

        for a in xml_tag.attributes().with_checks(false) {
            if let Ok(attr) = a {
                let k = xml.decode(&attr.key)?;
                let v = attr.unescape_and_decode_value(&xml)?;

                part.set_prp(k, v);
            }
        }

        Ok(part)
    }

    fn read_style(xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                  xml_tag: &BytesStart,
                  style: &mut Style) -> Result<(), OdsError> {
        for attr in xml_tag.attributes().with_checks(false) {
            match attr? {
                attr if attr.key == b"style:name" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    style.set_name(v);
                }
                attr if attr.key == b"style:family" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    match v.as_ref() {
                        "table" => style.family = Family::Table,
                        "table-column" => style.family = Family::TableColumn,
                        "table-row" => style.family = Family::TableRow,
                        "table-cell" => style.family = Family::TableCell,
                        _ => {}
                    }
                }
                attr if attr.key == b"style:parent-style-name" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    style.parent = Some(v);
                }
                attr if attr.key == b"style:data-style-name" => {
                    let v = attr.unescape_and_decode_value(&xml)?;
                    style.value_style = Some(v);
                }
                _ => { /* noop */ }
            }
        }

        Ok(())
    }

    fn copy_style_properties(style: &mut Style,
                             add_fn: &dyn Fn(&mut Style, &str, String),
                             xml: &mut quick_xml::Reader<BufReader<&mut ZipFile>>,
                             xml_tag: &BytesStart) -> Result<(), OdsError> {
        for attr in xml_tag.attributes().with_checks(false) {
            if let Ok(attr) = attr {
                let k = xml.decode(&attr.key)?;
                let v = attr.unescape_and_decode_value(&xml)?;
                add_fn(style, k, v);
            }
        }

        Ok(())
    }
}

mod write_ods {
    use std::collections::{BTreeMap, HashMap, HashSet};
    use std::fs::{File, rename};
    use std::io;
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};

    use chrono::NaiveDateTime;
    use quick_xml::events::{BytesDecl, Event};
    use zip::write::FileOptions;

    use crate::{Family, Origin, FormatType, SCell, Sheet, Style, Value, ValueFormat, ValueType, WorkBook, FontDecl};
    use crate::ods::error::OdsError;
    use crate::ods::temp_zip::TempZip;
    use crate::ods::xml_util;

// this did not work out as expected ...
    // TODO: find out why this breaks content.xml
    // type OdsWriter = zip::ZipWriter<BufWriter<File>>;
    // type XmlOdsWriter<'a> = quick_xml::Writer<&'a mut zip::ZipWriter<BufWriter<File>>>;

    type OdsWriter = TempZip;
    type XmlOdsWriter<'a> = quick_xml::Writer<&'a mut OdsWriter>;

    /// Writes the ODS file.
    pub fn write_ods<P: AsRef<Path>>(book: &WorkBook, ods_path: P) -> Result<(), OdsError> {
        write_ods_clean(book, ods_path, true)?;
        Ok(())
    }

    /// Writes the ODS file. The parameter clean indicates the cleanup of the
    /// temp files at the end.
    pub fn write_ods_clean<P: AsRef<Path>>(book: &WorkBook, ods_path: P, clean: bool) -> Result<(), OdsError> {
        let orig_bak = if let Some(ods_orig) = &book.file {
            let mut orig_bak = ods_orig.clone();
            orig_bak.set_extension("bak");
            rename(&ods_orig, &orig_bak)?;
            Some(orig_bak)
        } else {
            None
        };

        // let zip_file = File::create(ods_path)?;
        // let mut zip_writer = zip::ZipWriter::new(io::BufWriter::new(zip_file));
        let mut zip_writer = TempZip::new(ods_path.as_ref());

        let mut file_set = HashSet::<String>::new();
        //
        if let Some(orig_bak) = orig_bak {
            copy_workbook(&orig_bak, &mut file_set, &mut zip_writer)?;
        }

        write_mimetype(&mut zip_writer, &mut file_set)?;
        write_manifest(&mut zip_writer, &mut file_set)?;
        write_manifest_rdf(&mut zip_writer, &mut file_set)?;
        write_meta(&mut zip_writer, &mut file_set)?;
        //write_settings(&mut zip_writer, &mut file_set)?;
        //write_configurations(&mut zip_writer, &mut file_set)?;

        write_ods_styles(&mut zip_writer, &mut file_set)?;
        write_ods_content(&book, &mut zip_writer, &mut file_set)?;

        zip_writer.zip()?;
        if clean {
            zip_writer.clean()?;
        }

        Ok(())
    }

    fn copy_workbook(ods_orig_name: &PathBuf, file_set: &mut HashSet<String>, zip_writer: &mut OdsWriter) -> Result<(), OdsError> {
        let ods_orig = File::open(ods_orig_name)?;
        let mut zip_orig = zip::ZipArchive::new(ods_orig)?;

        for i in 0..zip_orig.len() {
            let mut zip_entry = zip_orig.by_index(i)?;

            if zip_entry.is_dir() {
                if !file_set.contains(zip_entry.name()) {
                    file_set.insert(zip_entry.name().to_string());
                    zip_writer.add_directory(zip_entry.name(), FileOptions::default())?;
                }
            } else if !file_set.contains(zip_entry.name()) {
                file_set.insert(zip_entry.name().to_string());
                zip_writer.start_file(zip_entry.name(), FileOptions::default())?;
                let mut buf: [u8; 1024] = [0; 1024];
                loop {
                    let n = zip_entry.read(&mut buf)?;
                    if n == 0 {
                        break;
                    } else {
                        zip_writer.write_all(&buf[0..n])?;
                    }
                }
            }
        }

        Ok(())
    }

    fn write_mimetype(zip_out: &mut OdsWriter, file_set: &mut HashSet<String>) -> Result<(), io::Error> {
        if !file_set.contains("mimetype") {
            file_set.insert(String::from("mimetype"));

            zip_out.start_file("mimetype", FileOptions::default().compression_method(zip::CompressionMethod::Stored))?;

            let mime = "application/vnd.oasis.opendocument.spreadsheet";
            zip_out.write_all(mime.as_bytes())?;
        }

        Ok(())
    }

    fn write_manifest(zip_out: &mut OdsWriter, file_set: &mut HashSet<String>) -> Result<(), OdsError> {
        if !file_set.contains("META-INF/manifest.xml") {
            file_set.insert(String::from("META-INF/manifest.xml"));

            zip_out.add_directory("META-INF", FileOptions::default())?;
            zip_out.start_file("META-INF/manifest.xml", FileOptions::default())?;

            let mut xml_out = quick_xml::Writer::new_with_indent(zip_out, 32, 1);

            xml_out.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;

            xml_out.write_event(xml_util::start_a("manifest:manifest", vec![
                ("xmlns:manifest", String::from("urn:oasis:names:tc:opendocument:xmlns:manifest:1.0")),
                ("manifest:version", String::from("1.2")),
            ]))?;

            xml_out.write_event(xml_util::empty_a("manifest:file-entry", vec![
                ("manifest:full-path", String::from("/")),
                ("manifest:version", String::from("1.2")),
                ("manifest:media-type", String::from("application/vnd.oasis.opendocument.spreadsheet")),
            ]))?;
//        xml_out.write_event(xml_empty_a("manifest:file-entry", vec![
//            ("manifest:full-path", String::from("Configurations2/")),
//            ("manifest:media-type", String::from("application/vnd.sun.xml.ui.configuration")),
//        ]))?;
            xml_out.write_event(xml_util::empty_a("manifest:file-entry", vec![
                ("manifest:full-path", String::from("manifest.rdf")),
                ("manifest:media-type", String::from("application/rdf+xml")),
            ]))?;
            xml_out.write_event(xml_util::empty_a("manifest:file-entry", vec![
                ("manifest:full-path", String::from("styles.xml")),
                ("manifest:media-type", String::from("text/xml")),
            ]))?;
            xml_out.write_event(xml_util::empty_a("manifest:file-entry", vec![
                ("manifest:full-path", String::from("meta.xml")),
                ("manifest:media-type", String::from("text/xml")),
            ]))?;
            xml_out.write_event(xml_util::empty_a("manifest:file-entry", vec![
                ("manifest:full-path", String::from("content.xml")),
                ("manifest:media-type", String::from("text/xml")),
            ]))?;
//        xml_out.write_event(xml::xml_empty_a("manifest:file-entry", vec![
//            ("manifest:full-path", String::from("settings.xml")),
//            ("manifest:media-type", String::from("text/xml")),
//        ]))?;
            xml_out.write_event(xml_util::end("manifest:manifest"))?;
        }

        Ok(())
    }

    fn write_manifest_rdf(zip_out: &mut OdsWriter, file_set: &mut HashSet<String>) -> Result<(), OdsError> {
        if !file_set.contains("manifest.rdf") {
            file_set.insert(String::from("manifest.rdf"));

            zip_out.start_file("manifest.rdf", FileOptions::default())?;

            let mut xml_out = quick_xml::Writer::new(zip_out);

            xml_out.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
            xml_out.write(b"\n")?;

            xml_out.write_event(xml_util::start_a("rdf:RDF", vec![
                ("xmlns:rdf", String::from("http://www.w3.org/1999/02/22-rdf-syntax-ns#")),
            ]))?;

            xml_out.write_event(xml_util::start_a("rdf:Description", vec![
                ("rdf:about", String::from("content.xml")),
            ]))?;
            xml_out.write_event(xml_util::empty_a("rdf:type", vec![
                ("rdf:resource", String::from("http://docs.oasis-open.org/ns/office/1.2/meta/odf#ContentFile")),
            ]))?;
            xml_out.write_event(xml_util::end("rdf:Description"))?;

            xml_out.write_event(xml_util::start_a("rdf:Description", vec![
                ("rdf:about", String::from("")),
            ]))?;
            xml_out.write_event(xml_util::empty_a("ns0:hasPart", vec![
                ("xmlns:ns0", String::from("http://docs.oasis-open.org/ns/office/1.2/meta/pkg#")),
                ("rdf:resource", String::from("content.xml")),
            ]))?;
            xml_out.write_event(xml_util::end("rdf:Description"))?;

            xml_out.write_event(xml_util::start_a("rdf:Description", vec![
                ("rdf:about", String::from("")),
            ]))?;
            xml_out.write_event(xml_util::empty_a("rdf:type", vec![
                ("rdf:resource", String::from("http://docs.oasis-open.org/ns/office/1.2/meta/pkg#Document")),
            ]))?;
            xml_out.write_event(xml_util::end("rdf:Description"))?;

            xml_out.write_event(xml_util::end("rdf:RDF"))?;
        }

        Ok(())
    }

    fn write_meta(zip_out: &mut OdsWriter, file_set: &mut HashSet<String>) -> Result<(), OdsError> {
        if !file_set.contains("meta.xml") {
            file_set.insert(String::from("meta.xml"));

            zip_out.start_file("meta.xml", FileOptions::default())?;

            let mut xml_out = quick_xml::Writer::new(zip_out);

            xml_out.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
            xml_out.write(b"\n")?;

            xml_out.write_event(xml_util::start_a("office:document-meta", vec![
                ("xmlns:meta", String::from("urn:oasis:names:tc:opendocument:xmlns:meta:1.0")),
                ("xmlns:office", String::from("urn:oasis:names:tc:opendocument:xmlns:office:1.0")),
                ("office:version", String::from("1.2")),
            ]))?;

            xml_out.write_event(xml_util::start("office:meta"))?;

            xml_out.write_event(xml_util::start("meta:generator"))?;
            xml_out.write_event(xml_util::text("spreadsheet-ods 0.1.0"))?;
            xml_out.write_event(xml_util::end("meta:generator"))?;

            xml_out.write_event(xml_util::start("meta:creation-date"))?;
            let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
            let d = NaiveDateTime::from_timestamp(s.as_secs() as i64, 0);
            xml_out.write_event(xml_util::text(&d.format("%Y-%m-%dT%H:%M:%S%.f").to_string()))?;
            xml_out.write_event(xml_util::end("meta:creation-date"))?;

            xml_out.write_event(xml_util::start("meta:editing-duration"))?;
            xml_out.write_event(xml_util::text("P0D"))?;
            xml_out.write_event(xml_util::end("meta:editing-duration"))?;

            xml_out.write_event(xml_util::start("meta:editing-cycles"))?;
            xml_out.write_event(xml_util::text("1"))?;
            xml_out.write_event(xml_util::end("meta:editing-cycles"))?;

            xml_out.write_event(xml_util::start("meta:initial-creator"))?;
            xml_out.write_event(xml_util::text(&username::get_user_name().unwrap()))?;
            xml_out.write_event(xml_util::end("meta:initial-creator"))?;

            xml_out.write_event(xml_util::end("office:meta"))?;

            xml_out.write_event(xml_util::end("office:document-meta"))?;
        }

        Ok(())
    }

//fn write_settings(zip_out: &mut ZipWriter<BufWriter<File>>, file_set: &mut HashSet<String>) -> Result<(), OdsError> {
//    if !file_set.contains("settings.xml") {
//        file_set.insert(String::from("settings.xml"));
//
//        zip_out.start_file("settings.xml", FileOptions::default())?;
//
//        let mut xml_out = quick_xml::Writer::new(zip_out);
//
//        xml_out.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
//        xml_out.write(b"\n")?;
//
//        xml_out.write_event(xml_start_a("office:document-settings", vec![
//            ("xmlns:office", String::from("urn:oasis:names:tc:opendocument:xmlns:office:1.0")),
//            ("xmlns:ooo", String::from("http://openoffice.org/2004/office")),
//            ("xmlns:config", String::from("urn:oasis:names:tc:opendocument:xmlns:config:1.0")),
//            ("office:version", String::from("1.2")),
//        ]))?;
//
//        xml_out.write_event(xml_start("office:settings"))?;
//        xml_out.write_event(xml::xml_end("office:settings"))?;
//
//        xml_out.write_event(xml::xml_end("office:document-settings"))?;
//    }
//
//    Ok(())
//}

//fn write_configurations(zip_out: &mut ZipWriter<BufWriter<File>>, file_set: &mut HashSet<String>) -> Result<(), OdsError> {
//    if !file_set.contains("Configurations2") {
//        file_set.insert(String::from("Configurations2"));
//        file_set.insert(String::from("Configurations2/accelerator"));
//        file_set.insert(String::from("Configurations2/floater"));
//        file_set.insert(String::from("Configurations2/images"));
//        file_set.insert(String::from("Configurations2/images/Bitmaps"));
//        file_set.insert(String::from("Configurations2/menubar"));
//        file_set.insert(String::from("Configurations2/popupmenu"));
//        file_set.insert(String::from("Configurations2/progressbar"));
//        file_set.insert(String::from("Configurations2/statusbar"));
//        file_set.insert(String::from("Configurations2/toolbar"));
//        file_set.insert(String::from("Configurations2/toolpanel"));
//
//        zip_out.add_directory("Configurations2", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/accelerator", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/floater", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/images", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/images/Bitmaps", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/menubar", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/popupmenu", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/progressbar", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/statusbar", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/toolbar", FileOptions::default())?;
//        zip_out.add_directory("Configurations2/toolpanel", FileOptions::default())?;
//    }
//
//    Ok(())
//}

    fn write_ods_styles(zip_out: &mut OdsWriter, file_set: &mut HashSet<String>) -> Result<(), OdsError> {
        if !file_set.contains("styles.xml") {
            file_set.insert(String::from("styles.xml"));

            zip_out.start_file("styles.xml", FileOptions::default())?;

            let mut xml_out = quick_xml::Writer::new(zip_out);

            xml_out.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
            xml_out.write(b"\n")?;

            xml_out.write_event(xml_util::start_a("office:document-styles", vec![
                ("xmlns:meta", String::from("urn:oasis:names:tc:opendocument:xmlns:meta:1.0")),
                ("xmlns:office", String::from("urn:oasis:names:tc:opendocument:xmlns:office:1.0")),
                ("xmlns:fo", String::from("urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0")),
                ("xmlns:style", String::from("urn:oasis:names:tc:opendocument:xmlns:style:1.0")),
                ("xmlns:text", String::from("urn:oasis:names:tc:opendocument:xmlns:text:1.0")),
                ("xmlns:dr3d", String::from("urn:oasis:names:tc:opendocument:xmlns:dr3d:1.0")),
                ("xmlns:svg", String::from("urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0")),
                ("xmlns:chart", String::from("urn:oasis:names:tc:opendocument:xmlns:chart:1.0")),
                ("xmlns:table", String::from("urn:oasis:names:tc:opendocument:xmlns:table:1.0")),
                ("xmlns:number", String::from("urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0")),
                ("xmlns:of", String::from("urn:oasis:names:tc:opendocument:xmlns:of:1.2")),
                ("xmlns:calcext", String::from("urn:org:documentfoundation:names:experimental:calc:xmlns:calcext:1.0")),
                ("xmlns:loext", String::from("urn:org:documentfoundation:names:experimental:office:xmlns:loext:1.0")),
                ("xmlns:field", String::from("urn:openoffice:names:experimental:ooo-ms-interop:xmlns:field:1.0")),
                ("xmlns:form", String::from("urn:oasis:names:tc:opendocument:xmlns:form:1.0")),
                ("xmlns:script", String::from("urn:oasis:names:tc:opendocument:xmlns:script:1.0")),
                ("xmlns:presentation", String::from("urn:oasis:names:tc:opendocument:xmlns:presentation:1.0")),
                ("office:version", String::from("1.2")),
            ]))?;

            // TODO: read and write global styles

            xml_out.write_event(xml_util::end("office:document-styles"))?;
        }

        Ok(())
    }

    fn write_ods_content(book: &WorkBook, zip_out: &mut OdsWriter, file_set: &mut HashSet<String>) -> Result<(), OdsError> {
        file_set.insert(String::from("content.xml"));

        zip_out.start_file("content.xml", FileOptions::default())?;

        let mut xml_out = quick_xml::Writer::new_with_indent(zip_out, b' ', 1);

        xml_out.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
        xml_out.write(b"\n")?;
        xml_out.write_event(xml_util::start_a("office:document-content", vec![
            ("xmlns:presentation", String::from("urn:oasis:names:tc:opendocument:xmlns:presentation:1.0")),
            ("xmlns:grddl", String::from("http://www.w3.org/2003/g/data-view#")),
            ("xmlns:xhtml", String::from("http://www.w3.org/1999/xhtml")),
            ("xmlns:xsi", String::from("http://www.w3.org/2001/XMLSchema-instance")),
            ("xmlns:xsd", String::from("http://www.w3.org/2001/XMLSchema")),
            ("xmlns:xforms", String::from("http://www.w3.org/2002/xforms")),
            ("xmlns:dom", String::from("http://www.w3.org/2001/xml-events")),
            ("xmlns:script", String::from("urn:oasis:names:tc:opendocument:xmlns:script:1.0")),
            ("xmlns:form", String::from("urn:oasis:names:tc:opendocument:xmlns:form:1.0")),
            ("xmlns:math", String::from("http://www.w3.org/1998/Math/MathML")),
            ("xmlns:draw", String::from("urn:oasis:names:tc:opendocument:xmlns:drawing:1.0")),
            ("xmlns:dr3d", String::from("urn:oasis:names:tc:opendocument:xmlns:dr3d:1.0")),
            ("xmlns:text", String::from("urn:oasis:names:tc:opendocument:xmlns:text:1.0")),
            ("xmlns:style", String::from("urn:oasis:names:tc:opendocument:xmlns:style:1.0")),
            ("xmlns:meta", String::from("urn:oasis:names:tc:opendocument:xmlns:meta:1.0")),
            ("xmlns:ooo", String::from("http://openoffice.org/2004/office")),
            ("xmlns:loext", String::from("urn:org:documentfoundation:names:experimental:office:xmlns:loext:1.0")),
            ("xmlns:svg", String::from("urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0")),
            ("xmlns:of", String::from("urn:oasis:names:tc:opendocument:xmlns:of:1.2")),
            ("xmlns:office", String::from("urn:oasis:names:tc:opendocument:xmlns:office:1.0")),
            ("xmlns:fo", String::from("urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0")),
            ("xmlns:field", String::from("urn:openoffice:names:experimental:ooo-ms-interop:xmlns:field:1.0")),
            ("xmlns:xlink", String::from("http://www.w3.org/1999/xlink")),
            ("xmlns:formx", String::from("urn:openoffice:names:experimental:ooxml-odf-interop:xmlns:form:1.0")),
            ("xmlns:dc", String::from("http://purl.org/dc/elements/1.1/")),
            ("xmlns:chart", String::from("urn:oasis:names:tc:opendocument:xmlns:chart:1.0")),
            ("xmlns:rpt", String::from("http://openoffice.org/2005/report")),
            ("xmlns:table", String::from("urn:oasis:names:tc:opendocument:xmlns:table:1.0")),
            ("xmlns:css3t", String::from("http://www.w3.org/TR/css3-text/")),
            ("xmlns:number", String::from("urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0")),
            ("xmlns:ooow", String::from("http://openoffice.org/2004/writer")),
            ("xmlns:oooc", String::from("http://openoffice.org/2004/calc")),
            ("xmlns:tableooo", String::from("http://openoffice.org/2009/table")),
            ("xmlns:calcext", String::from("urn:org:documentfoundation:names:experimental:calc:xmlns:calcext:1.0")),
            ("xmlns:drawooo", String::from("http://openoffice.org/2010/draw")),
            ("office:version", String::from("1.2")),
        ]))?;
        xml_out.write_event(xml_util::empty("office:scripts"))?;

        xml_out.write_event(xml_util::start("office:font-face-decls"))?;
        write_font_decl(&book.fonts, Origin::Content, &mut xml_out)?;
        xml_out.write_event(xml_util::end("office:font-face-decls")) ? ;

        xml_out.write_event(xml_util::start("office:automatic-styles"))?;
        write_styles(&book.styles, Origin::Content, &mut xml_out)?;
        write_value_styles(&book.formats, Origin::Content, &mut xml_out)?;
        xml_out.write_event(xml_util::end("office:automatic-styles"))?;

        xml_out.write_event(xml_util::start("office:body"))?;
        xml_out.write_event(xml_util::start("office:spreadsheet"))?;

        for sheet in &book.sheets {
            write_sheet(&book, &sheet, &mut xml_out)?;
        }

        xml_out.write_event(xml_util::end("office:spreadsheet"))?;
        xml_out.write_event(xml_util::end("office:body"))?;
        xml_out.write_event(xml_util::end("office:document-content"))?;

        Ok(())
    }

    fn write_sheet(book: &WorkBook, sheet: &Sheet, xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        let mut attr = Vec::new();
        attr.push(("table:name", sheet.name.to_string()));
        if let Some(style) = &sheet.style {
            attr.push(("table:style-name", style.to_string()));
        }
        xml_out.write_event(xml_util::start_a("table:table", attr))?;

        let max_cell = sheet.used_grid_size();

        write_table_columns(&sheet, &max_cell, xml_out)?;

        // table-row + table-cell
        let mut first_cell = true;
        let mut last_r: usize = 0;
        let mut last_c: usize = 0;

        for ((cur_row, cur_col), cell) in sheet.data.iter() {

            // There may be a lot of gaps of any kind in our data.
            // In the XML format there is no cell identification, every gap
            // must be filled with empty rows/columns. For this we need some
            // calculations.

            // For the repeat-counter we need to look forward.
            // Works nicely with the range operator :-)
            let (next_r, next_c) =
                if let Some(((next_r, next_c), _)) = sheet.data.range((*cur_row, cur_col + 1)..).next() {
                    (*next_r, *next_c)
                } else {
                    (max_cell.0, max_cell.1)
                };

            // Looking forward row-wise.
            let forward_dr = next_r as i32 - *cur_row as i32;

            // Column deltas are only relevant in the same row. Except we need to
            // fill up to max used columns.
            let forward_dc = if forward_dr >= 1 {
                max_cell.1 as i32 - *cur_col as i32
            } else {
                next_c as i32 - *cur_col as i32
            };

            // Looking backward row-wise.
            let backward_dr = *cur_row as i32 - last_r as i32;
            // When a row changes our delta is from zero to cur_col.
            let backward_dc = if backward_dr >= 1 {
                *cur_col as i32
            } else {
                *cur_col as i32 - last_c as i32
            };

            // After the first cell there is always an open row tag.
            if backward_dr > 0 && !first_cell {
                xml_out.write_event(xml_util::end("table:table-row"))?;
            }

            // Any empty rows before this one?
            if backward_dr > 1 {
                write_empty_rows_before(first_cell, backward_dr, &max_cell, xml_out)?;
            }

            // Start a new row if there is a delta or we are at the start
            if backward_dr > 0 || first_cell {
                write_start_current_row(sheet, *cur_row, backward_dc, xml_out)?;
            }

            // And now to something completely different ...
            write_cell(book, cell, xml_out)?;

            // There may be some blank cells until the next one, but only one less the forward.
            if forward_dc > 1 {
                write_empty_cells(forward_dc, xml_out)?;
            }

            first_cell = false;
            last_r = *cur_row;
            last_c = *cur_col;
        }

        if !first_cell {
            xml_out.write_event(xml_util::end("table:table-row"))?;
        }

        xml_out.write_event(xml_util::end("table:table"))?;

        Ok(())
    }

    fn write_empty_cells(forward_dc: i32, xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        let mut attr = Vec::new();
        attr.push(("table:number-columns-repeated", (forward_dc - 1).to_string()));
        xml_out.write_event(xml_util::empty_a("table:table-cell", attr))?;

        Ok(())
    }

    fn write_start_current_row(sheet: &Sheet,
                               cur_row: usize,
                               backward_dc: i32,
                               xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        let mut attr = Vec::new();
        if let Some(row) = sheet.rows.get(&cur_row) {
            if let Some(style) = &row.style {
                attr.push(("table:style-name", style.to_string()));
            }
        }

        xml_out.write_event(xml_util::start_a("table:table-row", attr))?;

        // Might not be the first column in this row.
        if backward_dc > 0 {
            xml_out.write_event(xml_util::empty_a("table:table-cell", vec![
                ("table:number-columns-repeated", backward_dc.to_string()),
            ]))?;
        }

        Ok(())
    }

    fn write_empty_rows_before(first_cell: bool,
                               backward_dr: i32,
                               max_cell: &(usize, usize),
                               xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        // Empty rows in between are 1 less than the delta, except at the very start.
        let mut attr = Vec::new();

        let empty_count = if first_cell { backward_dr } else { backward_dr - 1 };
        if empty_count > 1 {
            attr.push(("table:number-rows-repeated", empty_count.to_string()));
        }

        xml_out.write_event(xml_util::start_a("table:table-row", attr))?;

        // We fill the empty spaces completely up to max columns.
        xml_out.write_event(xml_util::empty_a("table:table-cell", vec![
            ("table:number-columns-repeated", max_cell.1.to_string()),
        ]))?;

        xml_out.write_event(xml_util::end("table:table-row"))?;

        Ok(())
    }

    fn write_table_columns(sheet: &Sheet,
                           max_cell: &(usize, usize),
                           xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        // table:table-column
        for c in sheet.columns.keys() {
            // For the repeat-counter we need to look forward.
            // Works nicely with the range operator :-)
            let next_c = if let Some((next_c, _)) = sheet.columns.range(c + 1..).next() {
                *next_c
            } else {
                max_cell.1
            };

            let dc = next_c as i32 - *c as i32;

            let column = &sheet.columns[c];

            let mut attr = Vec::new();
            if dc > 1 {
                attr.push(("table:number-columns-repeated", dc.to_string()));
            }
            if let Some(style) = &column.style {
                attr.push(("table:style-name", style.to_string()));
            }
            if let Some(style) = &column.def_cell_style {
                attr.push(("table:default-cell-style-name", style.to_string()));
            }
            xml_out.write_event(xml_util::empty_a("table:table-column", attr))?;
        }

        Ok(())
    }

    fn write_cell(book: &WorkBook,
                  cell: &SCell,
                  xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        let mut attr = Vec::new();
        let mut content: String = String::from("");
        if let Some(formula) = &cell.formula {
            attr.push(("table:formula", formula.to_string()));
        }

        if let Some(style) = &cell.style {
            attr.push(("table:style-name", style.to_string()));
        } else if let Some(value) = &cell.value {
            if let Some(style) = book.def_style(value.value_type()) {
                attr.push(("table:style-name", style.to_string()));
            }
        }

        // Might not yield a useful result. Could not exist, or be in styles.xml
        // which I don't read. Defaulting to to_string() seems reasonable.
        let value_style = if let Some(style_name) = &cell.style {
            book.find_value_style(style_name)
        } else {
            None
        };

        let mut is_empty = false;
        match &cell.value {
            None => {
                is_empty = true;
            }
            Some(Value::Text(s)) => {
                attr.push(("office:value-type", String::from("string")));
                if let Some(value_style) = value_style {
                    content = value_style.format_str(s);
                } else {
                    content = s.to_string();
                }
            }
            Some(Value::DateTime(d)) => {
                attr.push(("office:value-type", String::from("date")));
                attr.push(("office:date-value", d.format("%Y-%m-%dT%H:%M:%S%.f").to_string()));
                if let Some(value_style) = value_style {
                    content = value_style.format_datetime(d);
                } else {
                    content = d.format("%d.%m.%Y").to_string();
                }
            }
            Some(Value::TimeDuration(d)) => {
                attr.push(("office:value-type", String::from("time")));

                let mut buf = String::from("PT");
                buf.push_str(&d.num_hours().to_string());
                buf.push_str("H");
                buf.push_str(&(d.num_minutes() % 60).to_string());
                buf.push_str("M");
                buf.push_str(&(d.num_seconds() % 60).to_string());
                buf.push_str(".");
                buf.push_str(&(d.num_milliseconds() % 1000).to_string());
                buf.push_str("S");

                attr.push(("office:time-value", buf));
                if let Some(value_style) = value_style {
                    content = value_style.format_time_duration(d);
                } else {
                    content.push_str(&d.num_hours().to_string());
                    content.push_str(":");
                    content.push_str(&(d.num_minutes() % 60).to_string());
                    content.push_str(":");
                    content.push_str(&(d.num_seconds() % 60).to_string());
                    content.push_str(".");
                    content.push_str(&(d.num_milliseconds() % 1000).to_string());
                }
            }
            Some(Value::Boolean(b)) => {
                attr.push(("office:value-type", String::from("boolean")));
                attr.push(("office:boolean-value", b.to_string()));
                if let Some(value_style) = value_style {
                    content = value_style.format_boolean(*b);
                } else {
                    content = b.to_string();
                }
            }
            Some(Value::Currency(c, v)) => {
                attr.push(("office:value-type", String::from("currency")));
                attr.push(("office:currency", c.to_string()));
                attr.push(("office:value", v.to_string()));
                if let Some(value_style) = value_style {
                    content = value_style.format_float(*v);
                } else {
                    content.push_str(c);
                    content.push_str(" ");
                    content.push_str(&v.to_string());
                }
            }
            Some(Value::Number(v)) => {
                attr.push(("office:value-type", String::from("float")));
                attr.push(("office:value", v.to_string()));
                if let Some(value_style) = value_style {
                    content = value_style.format_float(*v);
                } else {
                    content = v.to_string();
                }
            }
            Some(Value::Percentage(v)) => {
                attr.push(("office:value-type", String::from("percentage")));
                attr.push(("office:value", format!("{}%", v)));
                if let Some(value_style) = value_style {
                    content = value_style.format_float(*v * 100.0);
                } else {
                    content = (v * 100.0).to_string();
                }
            }
        }

        if !is_empty {
            xml_out.write_event(xml_util::start_a("table:table-cell", attr))?;
            xml_out.write_event(xml_util::start("text:p"))?;
            xml_out.write_event(xml_util::text(&content))?;
            xml_out.write_event(xml_util::end("text:p"))?;
            xml_out.write_event(xml_util::end("table:table-cell"))?;
        } else {
            xml_out.write_event(xml_util::empty_a("table:table-cell", attr))?;
        }

        Ok(())
    }

    fn write_font_decl(fonts: &BTreeMap<String, FontDecl>, origin: Origin, xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        for font in fonts.values().filter(|s| s.origin == origin) {
            let mut attr: HashMap<String, String> = HashMap::new();

            attr.insert("style:name".to_string(), font.name.to_string());

            if let Some(prp) = &font.prp {
                for (k, v) in prp {
                    attr.insert(k.to_string(), v.to_string());
                }
            }

            xml_out.write_event(xml_util::start_m("style:style", &attr))?;
        }
        Ok(())
    }

    fn write_styles(styles: &BTreeMap<String, Style>, origin: Origin, xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        for style in styles.values().filter(|s| s.origin == origin) {
            let mut attr: Vec<(&str, String)> = Vec::new();

            attr.push(("style:name", style.name.to_string()));
            let family = match style.family {
                Family::Table => "table",
                Family::TableColumn => "table-column",
                Family::TableRow => "table-row",
                Family::TableCell => "table-cell",
                Family::None => "",
            };
            attr.push(("style:family", family.to_string()));
            if let Some(display_name) = &style.display_name {
                attr.push(("style:display-name", display_name.to_string()));
            }
            if let Some(parent) = &style.parent {
                attr.push(("style:parent-style-name", parent.to_string()));
            }
            if let Some(value_style) = &style.value_style {
                attr.push(("style:data-style-name", value_style.to_string()));
            }
            xml_out.write_event(xml_util::start_a("style:style", attr))?;

            if let Some(table_cell_prp) = &style.table_cell_prp {
                xml_out.write_event(xml_util::empty_m("style:table-cell-properties", &table_cell_prp))?;
            }
            if let Some(table_col_prp) = &style.table_col_prp {
                xml_out.write_event(xml_util::empty_m("style:table-column-properties", &table_col_prp))?;
            }
            if let Some(table_row_prp) = &style.table_row_prp {
                xml_out.write_event(xml_util::empty_m("style:table-row-properties", &table_row_prp))?;
            }
            if let Some(table_prp) = &style.table_prp {
                xml_out.write_event(xml_util::empty_m("style:table-properties", &table_prp))?;
            }
            if let Some(paragraph_prp) = &style.paragraph_prp {
                xml_out.write_event(xml_util::empty_m("style:paragraph-properties", &paragraph_prp))?;
            }
            if let Some(text_prp) = &style.text_prp {
                xml_out.write_event(xml_util::empty_m("style:text-properties", &text_prp))?;
            }

            xml_out.write_event(xml_util::end("style:style"))?;
        }

        Ok(())
    }

    fn write_value_styles(styles: &BTreeMap<String, ValueFormat>, origin: Origin, xml_out: &mut XmlOdsWriter) -> Result<(), OdsError> {
        for style in styles.values().filter(|s| s.origin == origin) {
            let tag = match style.v_type {
                ValueType::Boolean => "number:boolean-style",
                ValueType::Number => "number:number-style",
                ValueType::Text => "number:text-style",
                ValueType::TimeDuration => "number:time-style",
                ValueType::Percentage => "number:percentage-style",
                ValueType::Currency => "number:currency-style",
                ValueType::DateTime => "number:date-style",
            };

            let mut attr: HashMap<String, String> = HashMap::new();
            attr.insert(String::from("style:name"), style.name.to_string());
            if let Some(m) = &style.prp {
                for (k, v) in m {
                    attr.insert(k.to_string(), v.to_string());
                }
            }
            xml_out.write_event(xml_util::start_m(tag, &attr))?;

            if let Some(parts) = style.parts() {
                for part in parts {
                    let part_tag = match part.ftype {
                        FormatType::Boolean => "number:boolean",
                        FormatType::Number => "number:number",
                        FormatType::Scientific => "number:scientific-number",
                        FormatType::CurrencySymbol => "number:currency-symbol",
                        FormatType::Day => "number:day",
                        FormatType::Month => "number:month",
                        FormatType::Year => "number:year",
                        FormatType::Era => "number:era",
                        FormatType::DayOfWeek => "number:day-of-week",
                        FormatType::WeekOfYear => "number:week-of-year",
                        FormatType::Quarter => "number:quarter",
                        FormatType::Hours => "number:hours",
                        FormatType::Minutes => "number:minutes",
                        FormatType::Seconds => "number:seconds",
                        FormatType::Fraction => "number:fraction",
                        FormatType::AmPm => "number:am-pm",
                        FormatType::EmbeddedText => "number:embedded-text",
                        FormatType::Text => "number:text",
                        FormatType::TextContent => "number:text-content",
                        FormatType::StyleText => "style:text",
                        FormatType::StyleMap => "style:map",
                    };

                    if part.ftype == FormatType::Text || part.ftype == FormatType::CurrencySymbol {
                        xml_out.write_event(xml_util::start_o(part_tag, part.prp.as_ref()))?;
                        if let Some(content) = &part.content {
                            xml_out.write_event(xml_util::text(content))?;
                        }
                        xml_out.write_event(xml_util::end(part_tag))?;
                    } else {
                        xml_out.write_event(xml_util::empty_o(part_tag, part.prp.as_ref()))?;
                    }
                }
            }
            xml_out.write_event(xml_util::end(tag))?;
        }

        Ok(())
    }
}