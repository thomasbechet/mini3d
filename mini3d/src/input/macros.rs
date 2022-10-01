#[macro_export]
macro_rules! action_pressed {
    ($manager:expr, $name:tt) => {
        $manager.find_action($name).map_or(false, |b| b.is_pressed())
    };
}

#[macro_export]
macro_rules! action_released {
    ($manager:expr, $name:tt) => {
        $manager.find_action($name).map_or(false, |b| b.is_released())
    };
}

#[macro_export]
macro_rules! action_just_pressed {
    ($manager:expr, $name:tt) => {
        $manager.find_action($name).map_or(false, |b| b.is_just_pressed())
    };
}

#[macro_export]
macro_rules! action_just_released {
    ($manager:expr, $name:tt) => {
        $manager.find_action($name).map_or(false, |b| b.is_just_released())
    };
}