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
use image::{
	codecs::{gif, png::PngEncoder},
	error::{ParameterError, ParameterErrorKind},
	io::Reader as ImageReader,
	Delay, DynamicImage, Frame, GenericImage, GenericImageView, ImageBuffer, ImageEncoder,
	ImageError, ImageResult, Pixel, Rgba, RgbaImage,
};

use crate::handler::APIResponseError;

pub trait ProperResultConversion<T> {
	fn proper_result(self) -> Result<T, APIResponseError>;
}

// not sure why the implemented custom function isn't appearing, I'll figure out in a bit
impl<T> ProperResultConversion<T> for ImageResult<T> {
	fn proper_result(self) -> Result<T, APIResponseError> {
		match self {
			Ok(value) => Ok(value),
			Err(error) => Err(APIResponseError::new(error.to_string())),
		}
	}
}

pub trait AlphaImplementations: GenericImageView + GenericImage {
	/// Copies all of the pixels from another image into this image while taking an alpha threshold into account.
	///
	/// The other image is copied with the top-left corner of the
	/// other image placed at (x, y).
	///
	/// In order to copy only a piece of the other image, use [`GenericImageView::view`].
	///
	/// You can use [`FlatSamples`] to source pixels from an arbitrary regular raster of channel
	/// values, for example from a foreign interface or a fixed image.
	///
	/// # Returns
	/// Returns an error if the image is too large to be copied at the given position
	///
	/// [`GenericImageView::view`]: trait.GenericImageView.html#method.view
	/// [`FlatSamples`]: flat/struct.FlatSamples.html
	fn copy_within_alpha_threshold<O>(
		&mut self,
		other: &O,
		x: u32,
		y: u32,
		threshold: u8,
	) -> ImageResult<()>
	where
		O: GenericImageView<Pixel = Self::Pixel>;

	/// copies all pixels from another image into this image and blends based on alpha values.
	fn copy_with_blend<O>(&mut self, other: &O, x: u32, y: u32, threshold: u8) -> ImageResult<()>
	where
		O: GenericImageView<Pixel = Self::Pixel>;
}

impl AlphaImplementations for ImageBuffer<Rgba<u8>, Vec<u8>> {
	fn copy_within_alpha_threshold<O>(
		&mut self,
		other: &O,
		x: u32,
		y: u32,
		threshold: u8,
	) -> ImageResult<()>
	where
		O: GenericImageView<Pixel = Self::Pixel>,
	{
		// Do bounds checking here so we can use the non-bounds-checking
		// functions to copy pixels.
		if self.width() < other.width() + x || self.height() < other.height() + y {
			return Err(ImageError::Parameter(ParameterError::from_kind(
				ParameterErrorKind::DimensionMismatch,
			)));
		}

		for k in 0..other.height() {
			for i in 0..other.width() {
				let p = other.get_pixel(i, k);
				if p[3] > threshold {
					self.put_pixel(i + x, k + y, p);
				}
			}
		}
		Ok(())
	}

	fn copy_with_blend<O>(&mut self, other: &O, x: u32, y: u32, threshold: u8) -> ImageResult<()>
	where
		O: GenericImageView<Pixel = Self::Pixel>,
	{
		// Do bounds checking here so we can use the non-bounds-checking
		// functions to copy pixels.
		if self.width() < other.width() + x || self.height() < other.height() + y {
			return Err(ImageError::Parameter(ParameterError::from_kind(
				ParameterErrorKind::DimensionMismatch,
			)));
		}

		// blend transparent pixels with the background
		for k in 0..other.height() {
			for i in 0..other.width() {
				let p = other.get_pixel(i, k);
				if p[3] > threshold {
					self.put_pixel(i + x, k + y, p);
				} else {
					let bg = self.get_pixel_mut(i + x, k + y);
					bg.blend(&p);
				}
			}
		}

		Ok(())
	}
}

pub async fn image_request(image: &str) -> Result<Vec<u8>, APIResponseError> {
	let request = match reqwest::get(image).await {
		Ok(value) => value,
		Err(error) => return Err(APIResponseError::new(error.to_string())),
	};

	match request.bytes().await {
		Ok(value) => Ok(value.to_vec()),
		Err(error) => Err(APIResponseError::new(error.to_string())),
	}
}

pub async fn resolve_asset_path(
	image: &str,
	large: bool,
) -> Result<DynamicImage, APIResponseError> {
	let image_readout = match ImageReader::open(format!(
		"assets/images/{}/{}",
		if large { 1000 } else { 250 },
		image
	)) {
		Ok(value) => value,
		Err(error) => return Err(APIResponseError::new(error.to_string())),
	};

	match image_readout.decode() {
		Ok(value) => Ok(value),
		Err(error) => Err(APIResponseError::new(error.to_string())),
	}
}

const CLEARPIXEL: image::Rgba<u8> = image::Rgba([0, 0, 0, 0]);

pub fn round_image(img: &mut image::RgbaImage) {
	let radius_squared = if img.width() < img.height() {
		(img.width() / 2).pow(2)
	} else {
		(img.height() / 2).pow(2)
	};

	/*
	 * NOTE: an overflow panic occurs here on a debug build but doesn't on a release build, I've tried casting them
	 * to i32 so then it wouldn't happen anymore but it just caused more problems so I'll not worry about it
	 */
	for w in 0..img.width() {
		let x = (w - img.width() / 2).pow(2);
		for h in 0..img.height() {
			let y = (h - img.height() / 2).pow(2);
			if x + y > radius_squared {
				img.put_pixel(w, h, CLEARPIXEL)
			}
		}
	}
}

// we may have a use for this function later on if we need to modify an image's alpha for some particular reason
#[allow(dead_code)]
pub fn apply_alpha_threshold(img: &mut image::RgbaImage, threshold: u8) {
	for pixel in img.pixels_mut() {
		if pixel[3] < threshold {
			*pixel = CLEARPIXEL;
		}
	}
}

/// If the large option is false, divide given number by 4.
pub fn smallify_large_number(num: u32, large: bool) -> u32 {
	if large {
		num
	} else {
		num / 4
	}
}

pub struct BoundaryCropOutput {
	pub image: DynamicImage,
	pub x_pos: u32,
	pub y_pos: u32,
}

/// a function that simplifies the process of cropping an image when it goes out of an image's boundaries.
///
/// good to use in cases where you're overlaying an image on another image with varying positions and sizes
pub fn out_of_bounds_crop(
	image: DynamicImage,
	x_pos: i32,
	y_pos: i32,
	max_width: u32,
	max_height: u32,
) -> BoundaryCropOutput {
	let mut output = BoundaryCropOutput {
		image,
		x_pos: 0,
		y_pos: 0,
	};

	// check the y-axis
	let avatar_height_signed: i32 = output.image.height().try_into().unwrap();
	let height_total = avatar_height_signed + y_pos;

	if height_total > max_height.try_into().unwrap() {
		let y_pos_unsigned: u32 = y_pos.try_into().unwrap();
		let height_total = output.image.height() + y_pos_unsigned;
		let new_height = output.image.height() - (height_total - max_height);

		output.image = output.image.crop(0, 0, output.image.width(), new_height);
		output.y_pos = y_pos.try_into().unwrap();
	} else if y_pos < 0 {
		output.image = output.image.crop(
			0,
			(-y_pos).try_into().unwrap(),
			output.image.width(),
			height_total.try_into().unwrap(),
		);
	} else {
		output.y_pos = y_pos.try_into().unwrap();
	};

	// check the x-axis
	let avatar_width_signed: i32 = output.image.width().try_into().unwrap();
	let width_total = avatar_width_signed + x_pos;

	if width_total > max_width.try_into().unwrap() {
		let x_pos_unsigned: u32 = x_pos.try_into().unwrap();
		let width_total = output.image.width() + x_pos_unsigned;
		let new_width = output.image.width() - (width_total - max_width);

		output.image = output.image.crop(0, 0, new_width, output.image.height());
		output.x_pos = x_pos.try_into().unwrap();
	} else if x_pos < 0 {
		output.image = output.image.crop(
			0,
			(-x_pos).try_into().unwrap(),
			width_total.try_into().unwrap(),
			output.image.height(),
		);
	} else {
		output.x_pos = x_pos.try_into().unwrap();
	};

	output
}

/// shortens the process for loading a user's avatar, converting to a DynamicImage, then flipping if necessary
pub async fn load_avatar_from_url(
	url: String,
	flip: bool,
) -> Result<DynamicImage, APIResponseError> {
	let avatar =
		image::load_from_memory_with_format(&image_request(&url).await?, image::ImageFormat::Png)
			.proper_result()?;
	if flip {
		return Ok(avatar.fliph());
	}
	Ok(avatar)
}

pub struct GifAssistant {
	pub encoding_bytes: Vec<u8>,
}

impl GifAssistant {
	/// This function greatly reduces the amount of work needed to create and encode a GIF
	///
	/// access `encoding_bytes` on the struct when you're ready to send the GIF through a response
	pub fn create_gif<Overlayed: Clone, Overlaying: Clone>(
		overlayed_image: Overlayed,
		overlaying_image: Overlaying,
		ms: u32,
		iterations: u32,
		encoding_function: fn(u32, Overlayed, Overlaying) -> Result<RgbaImage, APIResponseError>,
	) -> Result<GifAssistant, APIResponseError> {
		let mut gif_assistant = GifAssistant {
			encoding_bytes: Vec::new(),
		};
		{
			let mut encoder = gif::GifEncoder::new(&mut gif_assistant.encoding_bytes);

			encoder.set_repeat(gif::Repeat::Infinite).proper_result()?;

			let mut frames: Vec<Frame> = Vec::new();

			for i in 0..iterations {
				let output =
					encoding_function(i, overlayed_image.to_owned(), overlaying_image.to_owned())?;

				let frame = Frame::from_parts(output, 0, 0, Delay::from_numer_denom_ms(ms, 1));

				frames.push(frame);
			}

			encoder.encode_frames(frames).proper_result()?;
		}

		Ok(gif_assistant)
	}
}

pub struct PngAssistant {
	pub encoding_bytes: Vec<u8>,
}

impl PngAssistant {
	pub fn create_png(image: DynamicImage) -> Result<PngAssistant, APIResponseError> {
		let mut png_assistant = PngAssistant {
			encoding_bytes: Vec::new(),
		};
		{
			let encoder = PngEncoder::new(&mut png_assistant.encoding_bytes);

			encoder
				.write_image(
					image.as_bytes(),
					image.width(),
					image.height(),
					image.color(),
				)
				.proper_result()?;
		}

		Ok(png_assistant)
	}
}

pub trait CustomRotation {
	/// Rotate an image by a specified amount of radians counter-clockwise and put the result into the destination [`ImageBuffer`].
	///
	/// This function is really basic and only works properly for images that have same width and height.
	fn rotate(self, degrees: i32) -> DynamicImage;
}
impl CustomRotation for DynamicImage {
	fn rotate(self, degrees: i32) -> DynamicImage {
		let (width, height) = self.dimensions();
		let radians = (degrees as f64) * std::f64::consts::PI / (180_f64);
		let radius: f64 = (self.width() as f64) / (2_f64);

		let mut new_image = RgbaImage::new(width, height);

		for x in 0..width {
			for y in 0..height {
				let x_new =
					(x as f64 - radius) * radians.cos() - (y as f64 - radius) * radians.sin();
				let y_new =
					(x as f64 - radius) * radians.sin() + (y as f64 - radius) * radians.cos();

				let mut x_new = (x_new + radius) as u32;
				let mut y_new = (y_new + radius) as u32;

				if x_new >= width {
					x_new = width - 1;
				}
				if y_new >= height {
					y_new = height - 1;
				}

				let pixel = self.get_pixel(x, y);

				new_image.put_pixel(x_new, y_new, pixel);
			}
		}

		// anti-aliasing via a convolution matrix
		let kernel = 1.0 / 9.0;
		let mut kernel_sum = 0.0;
		for _i in 0..3 {
			for _j in 0..3 {
				kernel_sum += kernel;
			}
		}
		for x in 0..width {
			for y in 0..height {
				let mut r = 0.0;
				let mut g = 0.0;
				let mut b = 0.0;
				for i in 0..3 {
					for j in 0..3 {
						let x_new = x as i32 + i - 1;
						let y_new = y as i32 + j - 1;
						if x_new >= 0
							&& x_new < width.try_into().unwrap()
							&& y_new >= 0 && y_new < height.try_into().unwrap()
						{
							let pixel = new_image
								.get_pixel(x_new.try_into().unwrap(), y_new.try_into().unwrap());
							r += pixel[0] as f64 * kernel;
							g += pixel[1] as f64 * kernel;
							b += pixel[2] as f64 * kernel;
						}
					}
				}
				r /= kernel_sum;
				g /= kernel_sum;
				b /= kernel_sum;
				new_image.put_pixel(x, y, Rgba([r as u8, g as u8, b as u8, 255]));
			}
		}

		DynamicImage::ImageRgba8(new_image)
	}
}
