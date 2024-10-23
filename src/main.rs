use std::{path::Path, time::Instant};

use deduper::{image::Image, process_images};

fn main() -> Result<(), fjall::Error> {
    let path = Path::new("test-data/01/house.jpg");
    process_images(path);
    let image = Image::from_path(&path.to_owned()).unwrap();
    let time = Instant::now();
    let hash = image.hash().unwrap();
    let total = Instant::now();
    println!(
        "hashed {} in {}",
        hash,
        total.duration_since(time).as_millis()
    );

    Ok(())
}
