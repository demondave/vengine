use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

pub fn load_voxels<P: AsRef<Path>>(path: P) -> Vec<([i32; 3], [u8; 4])> {
    let mut reader = BufReader::new(File::open(path).unwrap());

    let mut magic_number = [0u8; 8];

    reader.read_exact(&mut magic_number).unwrap();

    if magic_number != "VOXELSRS".as_bytes() {
        panic!("magic number doesn't match")
    }

    // Read number of voxels
    let mut tmp = [0u8; size_of::<u64>()];
    reader.read_exact(&mut tmp).unwrap();

    let num_voxels = u64::from_le_bytes(tmp) as usize;

    let mut voxels = Vec::with_capacity(num_voxels);

    // 3 Positions and 1 Color
    let mut row = [0i32; 4];

    for _ in 0..num_voxels {
        reader
            .read_exact(unsafe {
                std::slice::from_raw_parts_mut(
                    row.as_mut_ptr() as *mut u8,
                    row.len() * size_of::<i32>(),
                )
            })
            .unwrap();

        voxels.push(([row[0], row[1], row[2]], row[3].to_le_bytes()));
    }

    voxels.iter_mut().for_each(|(_, c)| gamma_correction(c));

    assert_eq!(num_voxels, voxels.len());

    voxels
}

fn gamma_correction(pixel: &mut [u8; 4]) {
    *pixel = [
        (srgb_to_linear(pixel[0] as f32 / 255.0) * 255.0) as u8,
        (srgb_to_linear(pixel[1] as f32 / 255.0) * 255.0) as u8,
        (srgb_to_linear(pixel[2] as f32 / 255.0) * 255.0) as u8,
        pixel[3], // Alpha bleibt linear
    ];
}

fn srgb_to_linear(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}
