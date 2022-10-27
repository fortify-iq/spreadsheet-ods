use crate::defaultstyles::DefaultFormat;
use crate::format::FormatNumberStyle;
use crate::locale::LocalizedValueFormat;
use crate::{ValueFormat, ValueType};
use icu_locid::{locale, Locale};

pub(crate) struct LocaleDeAt {}

pub(crate) static LOCALE_DE_AT: LocaleDeAt = LocaleDeAt {};

impl LocaleDeAt {
    const LOCALE: Locale = locale!("de_AT");
}

impl LocalizedValueFormat for LocaleDeAt {
    fn locale(&self) -> Locale {
        LocaleDeAt::LOCALE
    }

    fn boolean_format(&self) -> ValueFormat {
        let mut v =
            ValueFormat::new_localized(DefaultFormat::bool(), Self::LOCALE, ValueType::Boolean);
        v.part_boolean().build();
        v
    }

    fn number_format(&self) -> ValueFormat {
        let mut v =
            ValueFormat::new_localized(DefaultFormat::number(), Self::LOCALE, ValueType::Number);
        v.part_number().decimal_places(2).build();
        v
    }

    fn percentage_format(&self) -> ValueFormat {
        let mut v = ValueFormat::new_localized(
            DefaultFormat::percent(),
            Self::LOCALE,
            ValueType::Percentage,
        );
        v.part_number().decimal_places(2).build();
        v.part_text("%").build();
        v
    }

    fn currency_format(&self) -> ValueFormat {
        let mut v = ValueFormat::new_localized(
            DefaultFormat::currency(),
            Self::LOCALE,
            ValueType::Currency,
        );
        v.part_currency().locale(Self::LOCALE).symbol("€").build();
        v.part_text(" ").build();
        v.part_number()
            .decimal_places(2)
            .min_decimal_places(2)
            .grouping()
            .build();
        v.part_number()
            .decimal_places(2)
            .min_decimal_places(2)
            .grouping()
            .build();
        v
    }

    fn date_format(&self) -> ValueFormat {
        let mut v =
            ValueFormat::new_localized(DefaultFormat::date(), Self::LOCALE, ValueType::DateTime);
        v.part_day().style(FormatNumberStyle::Long).build();
        v.part_text(".").build();
        v.part_month().style(FormatNumberStyle::Long).build();
        v.part_text(".").build();
        v.part_year().style(FormatNumberStyle::Long).build();
        v
    }

    fn datetime_format(&self) -> ValueFormat {
        let mut v = ValueFormat::new_localized(
            DefaultFormat::datetime(),
            Self::LOCALE,
            ValueType::DateTime,
        );
        v.part_day().style(FormatNumberStyle::Long).build();
        v.part_text(".").build();
        v.part_month().style(FormatNumberStyle::Long).build();
        v.part_text(".").build();
        v.part_year().style(FormatNumberStyle::Long).build();
        v.part_text(" ").build();
        v.part_hours().style(FormatNumberStyle::Long).build();
        v.part_text(":").build();
        v.part_minutes().style(FormatNumberStyle::Long).build();
        v.part_text(":").build();
        v.part_seconds().style(FormatNumberStyle::Long).build();
        v
    }

    fn time_of_day_format(&self) -> ValueFormat {
        let mut v = ValueFormat::new_localized(
            DefaultFormat::datetime(),
            Self::LOCALE,
            ValueType::TimeDuration,
        );
        v.part_hours().style(FormatNumberStyle::Long).build();
        v.part_text(":").build();
        v.part_minutes().style(FormatNumberStyle::Long).build();
        v.part_text(":").build();
        v.part_seconds().style(FormatNumberStyle::Long).build();
        v
    }

    fn time_interval_format(&self) -> ValueFormat {
        let mut v = ValueFormat::new_localized(
            DefaultFormat::datetime(),
            Self::LOCALE,
            ValueType::DateTime,
        );
        v.set_truncate_on_overflow(false);

        v.part_hours().style(FormatNumberStyle::Long).build();
        v.part_text(":").build();
        v.part_minutes().style(FormatNumberStyle::Long).build();
        v.part_text(":").build();
        v.part_seconds().style(FormatNumberStyle::Long).build();
        v
    }
}
