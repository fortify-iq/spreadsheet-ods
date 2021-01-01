use std::fmt::{Display, Formatter};

/// Value type for angles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
    Deg(f64),
    Grad(f64),
    Rad(f64),
}

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Angle::Deg(v) => write!(f, "{}deg", v),
            Angle::Grad(v) => write!(f, "{}grad", v),
            Angle::Rad(v) => write!(f, "{}rad", v),
        }
    }
}

/// deg angles. 360°
#[macro_export]
macro_rules! deg {
    ($l:expr) => {
        Angle::Deg($l as f64)
    };
}

/// grad angles. 400°
#[macro_export]
macro_rules! grad {
    ($l:expr) => {
        Angle::Grad($l as f64)
    };
}

/// radians angle.
#[macro_export]
macro_rules! rad {
    ($l:expr) => {
        Angle::Rad($l as f64)
    };
}

/// Value type for lengths.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Cm(f64),
    Mm(f64),
    In(f64),
    Pt(f64),
    Pc(f64),
    Em(f64),
}

impl Display for Length {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Length::Cm(v) => write!(f, "{}cm", v),
            Length::Mm(v) => write!(f, "{}mm", v),
            Length::In(v) => write!(f, "{}in", v),
            Length::Pt(v) => write!(f, "{}pt", v),
            Length::Pc(v) => write!(f, "{}pc", v),
            Length::Em(v) => write!(f, "{}em", v),
        }
    }
}

/// Centimeters.
#[macro_export]
macro_rules! cm {
    ($l:expr) => {
        Length::Cm($l as f64)
    };
}

/// Millimeters.
#[macro_export]
macro_rules! mm {
    ($l:expr) => {
        Length::Mm($l as f64)
    };
}

/// Inches.
#[macro_export]
macro_rules! inch {
    ($l:expr) => {
        Length::In($l as f64)
    };
}

/// Point. 1/72"
#[macro_export]
macro_rules! pt {
    ($l:expr) => {
        Length::Pt($l as f64)
    };
}

/// Pica. 12/72"
#[macro_export]
macro_rules! pc {
    ($l:expr) => {
        Length::Pc($l as f64)
    };
}

/// Length depending on font size.
#[macro_export]
macro_rules! em {
    ($l:expr) => {
        Length::Em($l as f64)
    };
}

/// Font pitch.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FontPitch {
    Variable,
    Fixed,
}

impl Display for FontPitch {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            FontPitch::Variable => write!(f, "variable"),
            FontPitch::Fixed => write!(f, "fixed"),
        }
    }
}

/// Various border styles.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Border {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

impl Display for Border {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Border::None => write!(f, "none"),
            Border::Hidden => write!(f, "hidden"),
            Border::Dotted => write!(f, "dotted"),
            Border::Dashed => write!(f, "dashed"),
            Border::Solid => write!(f, "solid"),
            Border::Double => write!(f, "double"),
            Border::Groove => write!(f, "groove"),
            Border::Ridge => write!(f, "ridge"),
            Border::Inset => write!(f, "inset"),
            Border::Outset => write!(f, "outset"),
        }
    }
}

/// Page breaks.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PageBreak {
    Auto,
    Column,
    Page,
}

impl Display for PageBreak {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PageBreak::Auto => write!(f, "auto")?,
            PageBreak::Column => write!(f, "column")?,
            PageBreak::Page => write!(f, "page")?,
        }
        Ok(())
    }
}

/// Text keep together.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextKeep {
    Auto,
    Always,
}

impl Display for TextKeep {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextKeep::Auto => write!(f, "auto")?,
            TextKeep::Always => write!(f, "always")?,
        }
        Ok(())
    }
}

/// Writing modes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WritingMode {
    LrTb,
    RlTb,
    TbRl,
    TbLr,
    Lr,
    Rl,
    Tb,
    Page,
}

impl Display for WritingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            WritingMode::LrTb => write!(f, "lr-tb"),
            WritingMode::RlTb => write!(f, "rl-tb"),
            WritingMode::TbRl => write!(f, "tb-rl"),
            WritingMode::TbLr => write!(f, "tb-lr"),
            WritingMode::Lr => write!(f, "lr"),
            WritingMode::Rl => write!(f, "rl"),
            WritingMode::Tb => write!(f, "tb"),
            WritingMode::Page => write!(f, "page"),
        }
    }
}

/// Text wrapping.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WrapOption {
    NoWrap,
    Wrap,
}

impl Display for WrapOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            WrapOption::NoWrap => write!(f, "no-wrap"),
            WrapOption::Wrap => write!(f, "wrap"),
        }
    }
}

/// Rotation.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RotationAlign {
    None,
    Bottom,
    Top,
    Center,
}

impl Display for RotationAlign {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            RotationAlign::None => write!(f, "none"),
            RotationAlign::Bottom => write!(f, "bottom"),
            RotationAlign::Top => write!(f, "top"),
            RotationAlign::Center => write!(f, "center"),
        }
    }
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CellAlignVertical {
    Top,
    Middle,
    Bottom,
    Automatic,
}

impl Display for CellAlignVertical {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            CellAlignVertical::Top => write!(f, "top"),
            CellAlignVertical::Middle => write!(f, "middle"),
            CellAlignVertical::Bottom => write!(f, "bottom"),
            CellAlignVertical::Automatic => write!(f, "automatic"),
        }
    }
}

/// Fix uses the text-align attribute, value-type bases alignment on content.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextAlignSource {
    Fix,
    ValueType,
}

impl Display for TextAlignSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextAlignSource::Fix => write!(f, "fix"),
            TextAlignSource::ValueType => write!(f, "value-type"),
        }
    }
}

/// Horizontal alignment.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextAlign {
    Start,
    Center,
    End,
    Justify,
    Inside,
    Outside,
    Left,
    Right,
}

impl Display for TextAlign {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextAlign::Start => write!(f, "start"),
            TextAlign::Center => write!(f, "center"),
            TextAlign::End => write!(f, "end"),
            TextAlign::Justify => write!(f, "justify"),
            TextAlign::Inside => write!(f, "inside"),
            TextAlign::Outside => write!(f, "outside"),
            TextAlign::Left => write!(f, "left"),
            TextAlign::Right => write!(f, "right"),
        }
    }
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ParaAlignVertical {
    Top,
    Middle,
    Bottom,
    Auto,
    Baseline,
}

impl Display for ParaAlignVertical {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ParaAlignVertical::Top => write!(f, "top"),
            ParaAlignVertical::Middle => write!(f, "middle"),
            ParaAlignVertical::Bottom => write!(f, "bottom"),
            ParaAlignVertical::Auto => write!(f, "auto"),
            ParaAlignVertical::Baseline => write!(f, "baseline"),
        }
    }
}

/// Text style values.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextStyle {
    Normal,
    Italic,
    Oblique,
}

impl Display for TextStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextStyle::Normal => write!(f, "normal"),
            TextStyle::Italic => write!(f, "italic"),
            TextStyle::Oblique => write!(f, "oblique"),
        }
    }
}

/// Text weight values.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextWeight {
    Normal,
    Bold,
    W100,
    W200,
    W300,
    W400,
    W500,
    W600,
    W700,
    W800,
    W900,
}

impl Display for TextWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextWeight::Normal => write!(f, "normal"),
            TextWeight::Bold => write!(f, "bold"),
            TextWeight::W100 => write!(f, "100"),
            TextWeight::W200 => write!(f, "200"),
            TextWeight::W300 => write!(f, "300"),
            TextWeight::W400 => write!(f, "400"),
            TextWeight::W500 => write!(f, "500"),
            TextWeight::W600 => write!(f, "600"),
            TextWeight::W700 => write!(f, "700"),
            TextWeight::W800 => write!(f, "800"),
            TextWeight::W900 => write!(f, "900"),
        }
    }
}

/// Text case transformations.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextTransform {
    None,
    Lowercase,
    Uppercase,
    Capitalize,
}

impl Display for TextTransform {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextTransform::None => write!(f, "none"),
            TextTransform::Lowercase => write!(f, "lowercase"),
            TextTransform::Uppercase => write!(f, "uppercase"),
            TextTransform::Capitalize => write!(f, "capitalize"),
        }
    }
}

/// Text style engraved and embossed.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextRelief {
    None,
    Embossed,
    Engraved,
}

impl Display for TextRelief {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextRelief::None => write!(f, "none"),
            TextRelief::Embossed => write!(f, "embossed"),
            TextRelief::Engraved => write!(f, "engraved"),
        }
    }
}

/// Text style subscript or superscript.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextPosition {
    Sub,
    Super,
}

impl Display for TextPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TextPosition::Sub => write!(f, "sub"),
            TextPosition::Super => write!(f, "super"),
        }
    }
}

/// Line style for underline, overline, line-through.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineStyle {
    Dash,
    DotDash,
    DotDotDash,
    Dotted,
    LongDash,
    None,
    Solid,
    Wave,
}

impl Display for LineStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LineStyle::Dash => write!(f, "dash"),
            LineStyle::DotDash => write!(f, "dot-dash"),
            LineStyle::DotDotDash => write!(f, "dot-dot-dash"),
            LineStyle::Dotted => write!(f, "dotted"),
            LineStyle::LongDash => write!(f, "long-dash"),
            LineStyle::None => write!(f, "none"),
            LineStyle::Solid => write!(f, "solid"),
            LineStyle::Wave => write!(f, "wave"),
        }
    }
}

/// Line types for underline, overline, line-through.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineType {
    None,
    Single,
    Double,
}

impl Display for LineType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LineType::None => write!(f, "none"),
            LineType::Single => write!(f, "single"),
            LineType::Double => write!(f, "double"),
        }
    }
}

/// Line modes for underline, overline, line-through.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineMode {
    Continuous,
    SkipWhiteSpace,
}

impl Display for LineMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LineMode::Continuous => write!(f, "continuous"),
            LineMode::SkipWhiteSpace => write!(f, "skip-white-space"),
        }
    }
}

/// Line width for underline, overline, line-through.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineWidth {
    Auto,
    Normal,
    Bold,
    Thin,
    Medium,
    Thick,
}

impl Display for LineWidth {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LineWidth::Auto => write!(f, "auto"),
            LineWidth::Normal => write!(f, "normal"),
            LineWidth::Bold => write!(f, "bold"),
            LineWidth::Thin => write!(f, "thin"),
            LineWidth::Medium => write!(f, "medium"),
            LineWidth::Thick => write!(f, "thick"),
        }
    }
}
