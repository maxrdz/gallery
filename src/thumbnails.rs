// This file is part of Memories.
//
// Copyright (c) 2024 Max Rodriguez
// All rights reserved.
//
// Memories is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Memories is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Memories.  If not, see <http://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Asynchronous function for generating thumbnails via FFmpeg.

use crate::application::MemoriesApplication;
use crate::globals::{CACHE_THUMBNAILS_SUBDIR, FFMPEG_BINARY};
use async_fs::File;
use async_process::{Command, Output};
use gtk::glib::{g_debug, g_warning};
use std::io;
use std::path::Path;

/// Returns a string path to a JPEG image generated by ffmpeg
/// as a cropped square thumbnail for an image or a video.
pub async fn generate_thumbnail_image(
    file_path: &Path,
    cached_file_name: &str,
    hwaccel: bool,
) -> io::Result<String> {
    // This is the absolute outfile path for the thumbnail.
    let absolute_out_path: String = format!(
        "{}/{}/{}.jpg",
        MemoriesApplication::get_app_cache_directory(),
        CACHE_THUMBNAILS_SUBDIR,
        cached_file_name
    );

    // Check if we have the thumbnail already cached, if so, return its path.
    match File::open(Path::new(&absolute_out_path)).await {
        Ok(_) => {
            return Ok(absolute_out_path);
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => g_debug!(
                "Thumbnails",
                "'{}' not found in app cache. Generating new thumbnail.",
                absolute_out_path,
            ),
            _ => todo!(), // TODO: Extend error handling for cache check
        },
    }

    let file_extension: &str = file_path
        .extension()
        .expect("Was given file path with no file extension!")
        .to_str()
        .unwrap();

    let extra_arguments: &[&str] = match file_extension.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "heic" | "heif" => {
            &["-vf", "crop='min(iw,ih):min(iw,ih)',scale=150:150"]
        }
        "mp4" | "webm" | "mkv" | "mov" | "avi" | "gif" => &[
            "-vf",
            "thumbnail,crop='min(iw,ih):min(iw,ih)',scale=150:150",
            "-frames:v",
            "1",
        ],
        _ => {
            g_warning!(
                "Thumbnails",
                "'{}': unsupported file format, or an unrecognized extension.",
                file_extension
            );
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file format.",
            ));
        }
    };
    let mut ffmpeg_command: &mut Command = &mut Command::new(FFMPEG_BINARY);

    if hwaccel {
        ffmpeg_command = ffmpeg_command.args(["-hwaccel", "auto"]);
    }

    let ffmpeg_output: Result<Output, io::Error> = ffmpeg_command
        .arg("-i")
        .arg(file_path)
        // For some reason, ffmpeg loves to print to stderr. Setting the log level
        // to **only** error messages fixes the issue of an error always being returned.
        .args(["-loglevel", "error"])
        .args(extra_arguments)
        .arg(&absolute_out_path)
        .output()
        .await;

    match ffmpeg_output {
        // An error should **never** occur here, since we check the existence
        // of the ffmpeg binary installation at the start of the library load.
        Err(e) => panic!("Failed to execute ffmpeg binary!\n\n{}", e),
        Ok(v) => {
            if !v.stderr.is_empty() {
                g_debug!("Thumbnails", "FFmpeg printed to stderr: {:?}", v);
                Err(io::Error::new(io::ErrorKind::Other, "FFmpeg printed to stderr."))
            } else {
                Ok(absolute_out_path)
            }
        }
    }
}
