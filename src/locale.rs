//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! Localization support for calendar and date-related widgets.

/// Supported locales for calendar widgets.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CalendarLocale {
    #[default]
    English,
    German,
    French,
    Spanish,
    Dutch,
    Arabic,
}

impl CalendarLocale {
    /// Returns true if this locale uses right-to-left layout.
    pub fn is_rtl(&self) -> bool {
        matches!(self, CalendarLocale::Arabic)
    }

    /// Returns 2-letter weekday abbreviations starting with Monday (or Sunday for RTL locales).
    /// Example: "Mo", "Di", "Mi" for German.
    /// For Arabic the order is Sun–Sat (right-to-left reading: the rightmost column is Sunday).
    pub fn weekdays_short(&self) -> [&'static str; 7] {
        match self {
            CalendarLocale::English => ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
            CalendarLocale::German => ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"],
            CalendarLocale::French => ["Lu", "Ma", "Me", "Je", "Ve", "Sa", "Di"],
            CalendarLocale::Spanish => ["Lu", "Ma", "Mi", "Ju", "Vi", "Sa", "Do"],
            CalendarLocale::Dutch => ["Ma", "Di", "Wo", "Do", "Vr", "Za", "Zo"],
            // Mon Tue Wed Thu Fri Sat Sun — same week order as other locales, mirrored RTL
            // so Monday appears on the right and Sunday on the left.
            CalendarLocale::Arabic => ["ان", "ثل", "رب", "خم", "جم", "سب", "أح"],
        }
    }

    /// Returns full month names.
    pub fn months(&self) -> [&'static str; 12] {
        match self {
            CalendarLocale::English => [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            CalendarLocale::German => [
                "Januar",
                "Februar",
                "März",
                "April",
                "Mai",
                "Juni",
                "Juli",
                "August",
                "September",
                "Oktober",
                "November",
                "Dezember",
            ],
            CalendarLocale::French => [
                "Janvier",
                "Février",
                "Mars",
                "Avril",
                "Mai",
                "Juin",
                "Juillet",
                "Août",
                "Septembre",
                "Octobre",
                "Novembre",
                "Décembre",
            ],
            CalendarLocale::Spanish => [
                "Enero",
                "Febrero",
                "Marzo",
                "Abril",
                "Mayo",
                "Junio",
                "Julio",
                "Agosto",
                "Septiembre",
                "Octubre",
                "Noviembre",
                "Diciembre",
            ],
            CalendarLocale::Dutch => [
                "Januari",
                "Februari",
                "Maart",
                "April",
                "Mei",
                "Juni",
                "Juli",
                "Augustus",
                "September",
                "Oktober",
                "November",
                "December",
            ],
            CalendarLocale::Arabic => [
                "يناير",
                "فبراير",
                "مارس",
                "أبريل",
                "مايو",
                "يونيو",
                "يوليو",
                "أغسطس",
                "سبتمبر",
                "أكتوبر",
                "نوفمبر",
                "ديسمبر",
            ],
        }
    }

    /// Returns the label for "calendar week" (e.g., "Week", "KW", "Sem").
    pub fn week_label(&self) -> &'static str {
        match self {
            CalendarLocale::English => "Week",
            CalendarLocale::German => "KW",
            CalendarLocale::French => "Sem",
            CalendarLocale::Spanish => "Sem",
            CalendarLocale::Dutch => "Week",
            CalendarLocale::Arabic => "أسبوع",
        }
    }

    /// Returns the strftime format string for short dates in this locale.
    pub fn date_format(&self) -> &'static str {
        match self {
            CalendarLocale::English => "%m/%d/%Y",
            CalendarLocale::German => "%d.%m.%Y",
            CalendarLocale::French => "%d/%m/%Y",
            CalendarLocale::Spanish => "%d/%m/%Y",
            CalendarLocale::Dutch => "%d-%m-%Y",
            CalendarLocale::Arabic => "%d/%m/%Y",
        }
    }

    /// Returns abbreviated month names (3-4 letters).
    pub fn months_short(&self) -> [&'static str; 12] {
        match self {
            CalendarLocale::English => [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ],
            CalendarLocale::German => [
                "Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
            ],
            CalendarLocale::French => [
                "Jan", "Fév", "Mar", "Avr", "Mai", "Jui", "Jul", "Aoû", "Sep", "Oct", "Nov", "Déc",
            ],
            CalendarLocale::Spanish => [
                "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
            ],
            CalendarLocale::Dutch => [
                "Jan", "Feb", "Mrt", "Apr", "Mei", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dec",
            ],
            CalendarLocale::Arabic => [
                "يناير",
                "فبراير",
                "مارس",
                "أبريل",
                "مايو",
                "يونيو",
                "يوليو",
                "أغسطس",
                "سبتمبر",
                "أكتوبر",
                "نوفمبر",
                "ديسمبر",
            ],
        }
    }
}
