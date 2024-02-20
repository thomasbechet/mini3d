use mini3d_db::database::Database;

pub struct API<'a> {
    db: &'a mut Database,
}
