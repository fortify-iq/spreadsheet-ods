use crate::attrmap2::AttrMap2;
use crate::style::units::{
    Border, LengthPercent, Margin, MasterPageUsage, Percent, PrintCentering, PrintContent,
    PrintOrder, PrintOrientation, StyleNumFormat, WritingMode,
};
use crate::style::{
    border_line_width_string, border_string, color_string, shadow_string, ParseStyleAttr,
};
use crate::{Length, OdsError};
use color::Rgb;
use std::fmt::{Display, Formatter};

style_ref!(PageStyleRef);

/// The <style:page-layout> element represents the styles that specify the formatting properties
/// of a page.
///
/// For an example see [MasterPage].
///
#[derive(Debug, Clone)]
pub struct PageStyle {
    name: String,
    // TODO: reading and writing work on strings, get/set on an enum. is this nice?
    pub(crate) master_page_usage: Option<String>,
    style: AttrMap2,
    header: HeaderFooterStyle,
    footer: HeaderFooterStyle,
}

impl PageStyle {
    /// New pagestyle.
    pub(crate) fn new_empty() -> Self {
        Self {
            name: Default::default(),
            master_page_usage: None,
            style: Default::default(),
            header: Default::default(),
            footer: Default::default(),
        }
    }

    /// New pagestyle.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            master_page_usage: None,
            style: Default::default(),
            header: Default::default(),
            footer: Default::default(),
        }
    }

    /// Style reference.
    pub fn style_ref(&self) -> PageStyleRef {
        PageStyleRef::from(self.name())
    }

    /// Style name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Style name
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = name.into();
    }

    /// The style:page-usage attribute specifies the type of pages that a page master should
    /// generate.
    /// The defined values for the style:page-usage attribute are:
    /// * all: if there are no <style:header-left> or <style:footer-left> elements, the
    /// header and footer content is the same for left and right pages.
    /// * left: <style:header-left> or <style:footer-left> elements are ignored.
    /// * mirrored: if there are no <style:header-left> or <style:footer-left> elements,
    /// the header and footer content is the same for left and right pages.
    /// * right: <style:header-left> or <style:footer-left> elements are ignored.
    ///
    /// The default value for this attribute is all.
    pub fn set_page_usage(&mut self, usage: Option<MasterPageUsage>) {
        self.master_page_usage = usage.map(|m| m.to_string());
    }

    /// The style:page-usage attribute specifies the type of pages that a page master should
    /// generate.
    pub fn page_usage(&self) -> Result<Option<MasterPageUsage>, OdsError> {
        MasterPageUsage::parse_attr(self.master_page_usage.as_ref())
    }

    /// Attributes for header.
    pub fn headerstyle(&self) -> &HeaderFooterStyle {
        &self.header
    }

    /// Attributes for header.
    pub fn headerstyle_mut(&mut self) -> &mut HeaderFooterStyle {
        &mut self.header
    }

    /// Attributes for footer.
    pub fn footerstyle(&self) -> &HeaderFooterStyle {
        &self.footer
    }

    /// Attributes for footer.
    pub fn footerstyle_mut(&mut self) -> &mut HeaderFooterStyle {
        &mut self.footer
    }

    /// Access to all style attributes.
    pub(crate) fn style(&self) -> &AttrMap2 {
        &self.style
    }

    /// Access to all style attributes.
    pub(crate) fn style_mut(&mut self) -> &mut AttrMap2 {
        &mut self.style
    }

    fo_page_height!(style);
    fo_page_width!(style);
    style_first_page_number!(style);
    style_footnote_max_height!(style);
    style_num_format!(style);
    style_num_letter_sync!(style);
    style_num_prefix!(style);
    style_num_suffix!(style);
    style_paper_tray_name!(style);
    style_print!(style);
    style_print_orientation!(style);
    style_print_page_order!(style);
    style_scale_to!(style);
    style_scale_to_pages!(style);
    style_table_centering!(style);
    style_writing_mode!(style);
    fo_background_color!(style);
    fo_border!(style);
    fo_border_line_width!(style);
    fo_margin!(style);
    fo_padding!(style);
    style_dynamic_spacing!(style);
    style_shadow!(style);
}

/// Style attributes for header/footer.
#[derive(Clone, Debug, Default)]
pub struct HeaderFooterStyle {
    style: AttrMap2,
}

impl HeaderFooterStyle {
    /// General attributes.
    pub(crate) fn style(&self) -> &AttrMap2 {
        &self.style
    }

    /// General attributes.
    pub(crate) fn style_mut(&mut self) -> &mut AttrMap2 {
        &mut self.style
    }

    fo_background_color!(style);
    fo_border!(style);
    fo_margin!(style);
    fo_min_height!(style);
    fo_padding!(style);
    fo_border_line_width!(style);
    style_dynamic_spacing!(style);
    style_shadow!(style);
    svg_height!(style);
}
