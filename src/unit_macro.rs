/// deg angles. 360°
#[macro_export]
macro_rules! deg {
    ($l:expr) => {
        Angle::Deg($l as f64).into()
    };
}

/// grad angles. 400°
#[macro_export]
macro_rules! grad {
    ($l:expr) => {
        Angle::Grad($l as f64).into()
    };
}

/// radians angle.
#[macro_export]
macro_rules! rad {
    ($l:expr) => {
        Angle::Rad($l as f64).into()
    };
}

/// Centimeters.
#[macro_export]
macro_rules! cm {
    ($l:expr) => {
        Length::Cm($l as f64).into()
    };
}

/// Millimeters.
#[macro_export]
macro_rules! mm {
    ($l:expr) => {
        Length::Mm($l as f64).into()
    };
}

/// Inches.
#[macro_export]
macro_rules! inch {
    ($l:expr) => {
        Length::In($l as f64).into()
    };
}

/// Point. 1/72"
#[macro_export]
macro_rules! pt {
    ($l:expr) => {
        Length::Pt($l as f64).into()
    };
}

/// Pica. 12/72"
#[macro_export]
macro_rules! pc {
    ($l:expr) => {
        Length::Pc($l as f64).into()
    };
}

/// Length depending on font size.
#[macro_export]
macro_rules! em {
    ($l:expr) => {
        Length::Em($l as f64).into()
    };
}
