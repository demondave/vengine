use colorgrad::Gradient;

pub fn gradient_to_palette(gradient: &impl Gradient) -> [[f32; 4]; 128] {
    let mut buffer = [[0f32; 4]; 128];

    let diff = 1f32 / 128f32;
    let mut n = 0f32;

    for b in &mut buffer {
        b.copy_from_slice(&gradient.at(n).to_linear_rgba());
        n += diff;
    }

    buffer
}
