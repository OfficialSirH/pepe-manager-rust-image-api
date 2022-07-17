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
use actix_web::{error, post, web, HttpResponse, Result};
use derive_more::{Display, Error};
use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::image_manipulation::{ImageManipulationFunctionOptions, IMAGE_MANIPULATION};

#[derive(Deserialize, Debug)]
pub struct ImageQuery {
	url: String,
	large: Option<bool>,
	flip: Option<bool>,
}

#[derive(Serialize)]
pub struct MessageResponse {
	pub message: String,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "{}", name)]
pub struct APIResponseError {
	name: String,
}

impl APIResponseError {
	pub fn new(name: String) -> Self {
		APIResponseError { name }
	}
}

impl error::ResponseError for APIResponseError {}

#[post("/images/{type}")]
pub async fn create(
	image_type: web::Path<String>,
	query: web::Query<ImageQuery>,
) -> Result<HttpResponse, APIResponseError> {
	let start_time = std::time::Instant::now();

	let result = IMAGE_MANIPULATION[image_type.as_str()](
		&query
			.url
			.replace(".gif", ".png")
			.replace(".jpg", ".png")
			.replace(".jpeg", ".png")
			.replace(".webp", ".png"),
		ImageManipulationFunctionOptions {
			large: query.large.unwrap_or(false),
			flip: query.flip.unwrap_or(false),
		},
	)
	.await;

	let end_time = start_time.elapsed();

	match result {
		Ok(mut value) => {
			value.headers_mut().insert(
				HeaderName::from_static("time-taken"),
				HeaderValue::from_str(format!("{}", end_time.as_millis()).as_str()).unwrap(),
			);
			Ok(value)
		}
		Err(error) => Err(APIResponseError {
			name: error.to_string(),
		}),
	}
}
