fn init() {
    STORAGE.set_int("inventory.itemcount", 0);
    STORAGE.set_string("levels.level0", "");
    STORAGE.set_string("levels.level1", "");
    STORAGE.set_string("levels.level2", "");
    STORAGE.set_string("levels.level3", "");
    STORAGE.set_string("levels.level3", "");
}

fn update() {
    let count = STORAGE.get_int("inventory.itemcount");
    // print(count);
    STORAGE.set_int("inventory.itemcount", count + 1);
    let levels = STORAGE.list_keys("levels");
    // print("-----------");
    for level in levels {
        // print(level);
    }
    if INPUT.action_just_pressed("common", "up") {
        // WORLD.create_entity
    }
}