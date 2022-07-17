use dotenv::vars;

#[derive(Debug)]
pub struct Config {
	pub in_production: bool,
	pub server_addr: String,
}
impl Config {
	pub fn new() -> Self {
		let environment_vars: Vec<(String, String)> = vars().collect();

		let in_production = match find_key(&environment_vars, "NODE_ENV").as_str() {
			"development" => false,
			"production" => true,
			&_ => panic!("NODE_ENV isn't using the correct values"),
		};
		let port = find_key(&environment_vars, "IMAGE_API_PORT");

		Config {
			in_production,
			server_addr: match in_production {
				true => format!("0.0.0.0:{}", &port),
				false => format!("127.0.0.1:{}", &port),
			},
		}
	}
}

pub fn find_key(iteration: &[(String, String)], key_search: &'static str) -> String {
	match iteration.iter().find(|(key, _)| key == key_search) {
		Some((_, value)) => value.to_string(),
		None => panic!(
			"couldn't find '{}' in the environment variables",
			key_search
		),
	}
}
