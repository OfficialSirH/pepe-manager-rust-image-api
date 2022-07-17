/**
 * Copyright 2022, Micah Benac
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
mod config;
mod handler;
mod image_manipulation;
mod image_utilities;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use handler::create;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let config = crate::config::Config::new();

	HttpServer::new(move || {
		let cors = Cors::default()
			.allowed_origin_fn(|origin, _req_head| {
				let temp_config = crate::config::Config::new();
				match &temp_config.in_production {
					true => {
						origin.as_bytes().ends_with(b"pepe-is.life")
							|| origin.as_bytes().ends_with(b"pepemanager.com")
					}
					false => true,
				}
			})
			.allowed_methods(vec!["POST"]);

		App::new().wrap(cors).service(create)
	})
	.bind(&config.server_addr)?
	.run()
	.await?;
	println!("Server running at http://{}/", config.server_addr);

	Ok(())
}
