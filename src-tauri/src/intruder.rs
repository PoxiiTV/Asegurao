// Captura opcional de la webcam tras varios fallos. Devuelve el nombre del
// archivo guardado en la carpeta de fotos, o None si no se pudo capturar.

use crate::history::photos_dir;
use crate::history::now;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

pub fn capture() -> Option<String> {
    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut cam = Camera::new(CameraIndex::Index(0), format).ok()?;
    cam.open_stream().ok()?;
    // Descarta el primer par de fotogramas (sensor calibrando la exposición).
    let _ = cam.frame();
    let _ = cam.frame();
    let frame = cam.frame().ok()?;
    let image = frame.decode_image::<RgbFormat>().ok()?;

    let name = format!("intruso_{}.png", now());
    let path = photos_dir().join(&name);
    image.save_with_format(&path, image::ImageFormat::Png).ok()?;
    Some(name)
}
