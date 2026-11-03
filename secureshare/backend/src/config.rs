#[derive(Debug,Clone)]

pub struct Config {
    pub database_url: String,
    pub jwt_secret : String,
    pub jwt_max_age: i64,
    pub port: u16
}
impl Config {
    pub fn init() -> Config {
        let database_url: String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret: String = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set before usage");
        let jwt_max_age: String = std::env::var("JWT_MAX_AGE").expect("JWT MAX_AGE must be set before usage");
        Config {
            database_url,
            jwt_secret,
            jwt_max_age: jwt_max_age.parse::<i64>().unwrap(),
            port:5000
        }
    }
}