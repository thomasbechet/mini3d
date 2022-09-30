#[macro_export]
macro_rules! button_pressed {
    ($manager:expr, $name:tt) => {
        $manager.find_button($name).map_or(false, |b| b.is_pressed())
    };
}

#[macro_export]
macro_rules! button_released {
    ($manager:expr, $name:tt) => {
        $manager.find_button($name).map_or(false, |b| b.is_released())
    };
}

#[macro_export]
macro_rules! button_just_pressed {
    ($manager:expr, $name:tt) => {
        $manager.find_button($name).map_or(false, |b| b.is_just_pressed())
    };
}

#[macro_export]
macro_rules! button_just_released {
    ($manager:expr, $name:tt) => {
        $manager.find_button($name).map_or(false, |b| b.is_just_released())
    };
}