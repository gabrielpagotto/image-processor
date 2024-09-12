use image::imageops::FilterType;
use image::GenericImageView;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use std::{fs, process};
use tabled::{Table, Tabled};

const IMAGES_DIR: &str = "images";
const WITH_OUT_THREAD_IMAGES_DIR: &str = "results/with_out_thread_images";
const WITH_THREAD_IMAGES_DIR: &str = "results/with_thread_images";
const RESIZED_WIDTH: u32 = 800;
const RESIZED_HEIGHT: u32 = 600;
const EXTENSIONS: [&str; 4] = ["jpeg", "jpg", "png", "webp"];

fn main() {
    match get_image_paths() {
        Ok(paths) => {
            println!("processando...\n");
            process_with_out_threads(&paths);
            process_with_threads(&paths, 2);
            process_with_threads(&paths, 4);
            process_with_threads(&paths, 7);
            process_with_threads(&paths, 10);
        }
        Err(_) => {
            eprintln!(
                "error on find images, create a folder called 'images' and add the images inside."
            );
            process::exit(0)
        }
    }
}

#[derive(Tabled)]
struct ResizeAndApplyFilterResult {
    path: String,
    duration: String,
    size: String,
    dimensions: String,
    new_size: String,
    new_dimensions: String,
}

fn process_with_out_threads(paths: &Vec<String>) {
    fs::create_dir_all(WITH_OUT_THREAD_IMAGES_DIR).expect("Error on create no thread images.");
    let start = Instant::now();
    let mut results: Vec<ResizeAndApplyFilterResult> = Vec::new();

    for path in paths {
        results.push(resize_and_apply_filter(
            path.clone(),
            WITH_OUT_THREAD_IMAGES_DIR.to_string(),
        ));
    }

    let duration = start.elapsed();
    println!(
        "\nO processo das images levou {:?} segundos sem utilizar threads.",
        duration.as_secs_f64()
    );
    println!("MÉTRICAS POR IMAGEM");
    println!("{}", Table::new(results));
}

fn process_with_threads(paths: &Vec<String>, threads_number: usize) {
    fs::create_dir_all(WITH_THREAD_IMAGES_DIR).expect("Error on create with thread images.");
    let start = Instant::now();
    let chunk_size = paths.len() / threads_number;
    let mut handles = vec![];
    let results = Arc::new(Mutex::new(Vec::new()));

    for chunk in paths.chunks(chunk_size) {
        let paths_chunk = chunk.to_vec();
        let results_clone = Arc::clone(&results);
        let handle = thread::spawn(move || {
            for path in paths_chunk {
                let result = resize_and_apply_filter(path, WITH_THREAD_IMAGES_DIR.to_string());
                let mut results = results_clone.lock().unwrap();
                results.push(result);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!(
        "\nO processo levou {:?} segundos para processar as images utilizando {:?} threads.",
        duration.as_secs_f64(),
        threads_number,
    );
    println!("MÉTRICAS POR IMAGEM");
    let results = results.lock().unwrap();
    println!("{}", Table::new(results.iter().map(|result| result)));
}

fn get_image_paths() -> Result<Vec<String>, std::io::Error> {
    let mut paths: Vec<String> = Vec::new();
    let entries = fs::read_dir(IMAGES_DIR)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                if let Some(path_str) = path.to_str() {
                    paths.push(path_str.to_string());
                }
            }
        }
    }
    Ok(paths)
}

fn resize_and_apply_filter(path: String, to_save_dir: String) -> ResizeAndApplyFilterResult {
    let resized_path = path.replace(&format!("{}/", IMAGES_DIR), &format!("{}/", to_save_dir));

    match image::open(&path) {
        Ok(img) => {
            let start = Instant::now();
            let (width, height) = img.dimensions();
            let gray_img = img.grayscale();
            let contrast_img = gray_img.adjust_contrast(30.0);
            let resized_img =
                contrast_img.resize(RESIZED_WIDTH, RESIZED_HEIGHT, FilterType::Lanczos3);

            if let Err(e) = resized_img.save(&resized_path) {
                eprintln!("cannot save image {}: {}", path, e);
                process::exit(0)
            } else {
                let duration_in_secs = start.elapsed().as_secs_f64();
                let (new_width, new_height) = resized_img.dimensions();
                return ResizeAndApplyFilterResult {
                    path: path.clone(),
                    size: format!("{:.2}MB", get_file_size_in_mega_bytes(path)),
                    duration: format!("{:.2}s", duration_in_secs),
                    dimensions: format!("{:?}x{:?}", width, height),
                    new_size: format!("{:.2}MB", get_file_size_in_mega_bytes(resized_path)),
                    new_dimensions: format!("{:?}x{:?}", new_width, new_height),
                };
            }
        }
        Err(e) => {
            eprintln!("cannot open the image {}: {}", path, e);
            process::exit(0)
        }
    }
}

fn get_file_size_in_mega_bytes(path: String) -> f64 {
    let metadata = fs::metadata(path).expect("cannot obtain file metadata.");
    let file_size_in_bytes = metadata.len();
    file_size_in_bytes as f64 / (1024.0 * 1024.0)
}
