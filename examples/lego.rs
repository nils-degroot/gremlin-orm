use evil_orm::Entity;

#[derive(Debug, Entity)]
#[orm(table = "public.lego_colors")]
struct LegoColors {
    #[orm(pk)]
    id: i32,
    name: String,
    rgb: String,
    is_trans: String,
}

fn main() {
    let name = std::env::args().nth(1);
    println!("Hello, {}!", name.unwrap_or("world".into()));
}
