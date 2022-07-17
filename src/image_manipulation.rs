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
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Result};
use image::{
	codecs::gif, imageops::FilterType, io::Reader as ImageReader, Delay, DynamicImage, Frame,
};
use image::{GenericImage, RgbaImage};
use std::{future, ops::Index, pin::Pin};

use crate::handler::{APIResponseError, MessageResponse};
use crate::image_utilities::{
	load_avatar_from_url, out_of_bounds_crop, resolve_asset_path, round_image,
	smallify_large_number, AlphaImplementations, CustomRotation, GifAssistant, PngAssistant,
	ProperResultConversion,
};

#[derive(Debug)]
pub struct ImageManipulationFunctionOptions {
	pub large: bool,
	pub flip: bool,
}

type ImageManipulationFunctionReturn =
	Pin<Box<dyn future::Future<Output = Result<HttpResponse, APIResponseError>>>>;
type ImageManipulationFunction =
	fn(&str, ImageManipulationFunctionOptions) -> ImageManipulationFunctionReturn;

pub struct ImageManipulationFunctions {
	out_of_bounds_index: ImageManipulationFunction,
	// note: manually implement new image function into struct
	enter: ImageManipulationFunction,
	exit: ImageManipulationFunction,
}

impl Index<&'_ str> for ImageManipulationFunctions {
	type Output = ImageManipulationFunction;
	fn index(&self, s: &str) -> &ImageManipulationFunction {
		// note: manually implement indexing for each property
		match s {
			"enter" => &self.enter,
			"exit" => &self.exit,
			_ => &self.out_of_bounds_index,
		}
	}
}

pub const IMAGE_MANIPULATION: ImageManipulationFunctions = ImageManipulationFunctions {
	out_of_bounds_index,
	// note: manually place each function into this struct instance
	enter,
	exit,
};

pub fn out_of_bounds_index(
	_image: &str,
	_options: ImageManipulationFunctionOptions,
) -> ImageManipulationFunctionReturn {
	Box::pin(async move {
		Ok(HttpResponse::NotFound().json(MessageResponse {
			message: "the provided image type does not exist".to_owned(),
		}))
	})
}

pub fn enter(
	image: &str,
	options: ImageManipulationFunctionOptions,
) -> ImageManipulationFunctionReturn {
	let static_image = image.to_owned();
	Box::pin(async move {
		let avatar = load_avatar_from_url(static_image, options.flip).await?;
		let meme_image = resolve_asset_path("enter.png", options.large).await?;

		let avatar_x = smallify_large_number(35, options.large);
		let avatar_y = smallify_large_number(397, options.large);
		let avatar_dimensions = smallify_large_number(603, options.large);

		let mut avatar = avatar
			.resize_exact(avatar_dimensions, avatar_dimensions, FilterType::Triangle)
			.to_rgba8();

		round_image(&mut avatar);

		let mut meme_image = meme_image.to_rgba8();
		meme_image
			.copy_within_alpha_threshold(&avatar, avatar_x, avatar_y, 128)
			.proper_result()?;

		let meme_image = DynamicImage::ImageRgba8(meme_image);

		let png_assistant = PngAssistant::create_png(meme_image)?;

		Ok(HttpResponse::Ok()
			.content_type(ContentType::png())
			.body(png_assistant.encoding_bytes))
	})
}

pub fn exit(
	image: &str,
	options: ImageManipulationFunctionOptions,
) -> ImageManipulationFunctionReturn {
	let static_image = image.to_owned();
	Box::pin(async move {
		let avatar = load_avatar_from_url(static_image, options.flip).await?;
		let meme_image = resolve_asset_path("exit.png", options.large).await?;

		let avatar_x = smallify_large_number(35, options.large);
		let avatar_y = smallify_large_number(397, options.large);
		let avatar_dimensions = smallify_large_number(603, options.large);

		let mut avatar = avatar
			.resize_exact(avatar_dimensions, avatar_dimensions, FilterType::Triangle)
			.to_rgba8();

		round_image(&mut avatar);

		let mut meme_image = meme_image.to_rgba8();
		meme_image
			.copy_within_alpha_threshold(&avatar, avatar_x, avatar_y, 128)
			.proper_result()?;

		let meme_image = DynamicImage::ImageRgba8(meme_image);

		let png_assistant = PngAssistant::create_png(meme_image)?;

		Ok(HttpResponse::Ok()
			.content_type(ContentType::png())
			.body(png_assistant.encoding_bytes))
	})
}
